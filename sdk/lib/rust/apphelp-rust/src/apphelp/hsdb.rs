// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_731D7EC48EEF: [u16; 3] = [0x0020u16, 0x0009, 0x0000];
pub static RUSTCONV_WSTR_ABF250C9AB1D: [u16; 15] = [
    0x005Fu16, 0x005F, 0x0043, 0x004F, 0x004D, 0x0050, 0x0041, 0x0054, 0x005F, 0x004C,
    0x0041, 0x0059, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_B28803D501F2: [u16; 14] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x0056, 0x0045, 0x0052, 0x0053, 0x0049, 0x004F,
    0x004E, 0x004E, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_D86033D68ADE: [u16; 22] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C,
    0x0061, 0x0070, 0x0070, 0x0068, 0x0065, 0x006C, 0x0070, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_E8A4B2EE7EDE: [u16; 2] = [0x002Au16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_ABF250C9AB1D: [u16; 15] = [
    0x005Fu16, 0x005F, 0x0043, 0x004F, 0x004D, 0x0050, 0x0041, 0x0054, 0x005F, 0x004C,
    0x0041, 0x0059, 0x0045, 0x0052, 0x0000,
];

static mut EnvKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_ABF250C9AB1D.as_ptr() as PWSTR,
    };

pub const MAX_LAYER_LENGTH: usize = 256;

pub const GPLK_USER: i32 = 1;

pub const GPLK_MACHINE: i32 = 2;

#[repr(C)]
pub struct _ShimData {
    pub szModule: [WCHAR; (MAX_PATH) as _],
    pub dwSize: DWORD,
    pub dwMagic: DWORD,
    pub Query: SDBQUERYRESULT,
    pub szLayer: [WCHAR; MAX_LAYER_LENGTH],
    // Start ReactOS specific
    pub dwRosProcessCompatVersion: DWORD,
    #[cfg(target_arch = "x86_64")]
    pub RtlGetCurrentDirectory_U_RtlpMsysDecoy: PVOID,
}

pub type ShimData = _ShimData;
pub type PShimData = *mut _ShimData;

pub const SHIMDATA_MAGIC: u32 = 0xAC0DEDAB;


// C_ASSERT(SHIMDATA_MAGIC == REACTOS_SHIMDATA_MAGIC);
// C_ASSERT(sizeof(ShimData) == sizeof(ReactOS_ShimData));
// C_ASSERT(offsetof(ShimData, dwMagic) == offsetof(ReactOS_ShimData, dwMagic));
// C_ASSERT(offsetof(ShimData, dwRosProcessCompatVersion) == offsetof(ReactOS_ShimData, dwRosProcessCompatVersion));
// C_ASSERT(offsetof(ShimData, RtlGetCurrentDirectory_U_RtlpMsysDecoy) == offsetof(ReactOS_ShimData, RtlGetCurrentDirectory_U_RtlpMsysDecoy)); // _M_AMD64 only

unsafe extern "system" fn SdbpFileExists(path: LPCWSTR) -> BOOL {
    let attr: DWORD = GetFileAttributesW(path);
    return (attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_DIRECTORY) == 0) as BOOL;
}

unsafe extern "C" fn SdbpMatchFileAttributes(pdb: PDB, matching_file: TAGID, attribs: PATTRINFO, attr_count: DWORD) -> BOOL {
    let mut child: TAGID;
    child = SdbGetFirstChild(pdb, matching_file);
    while child != TAGID_NULL {
        'next: {
            let tag: TAG = SdbGetTagFromTagID(pdb, child);
            let mut n: DWORD;

            /* Already handled! */
            if tag as u32 == TAG_NAME {
                break 'next;
            }

            if tag as u32 == TAG_UPTO_BIN_FILE_VERSION ||
                tag as u32 == TAG_UPTO_BIN_PRODUCT_VERSION ||
                tag as u32 == TAG_UPTO_LINK_DATE
            {
                SHIM_WARN!(b"Unimplemented TAG_UPTO_XXXXX\n\0".as_ptr() as PCSTR);
                break 'next;
            }

            n = 0;
            while n < attr_count {
                let attr: PATTRINFO = attribs.offset(n as isize);
                if (*attr).flags == ATTRIBUTE_AVAILABLE && (*attr).type_ == tag {
                    let mut dwval: DWORD;
                    let mut lpval: *mut WCHAR;
                    let mut qwval: QWORD;
                    match (tag as u32) & TAG_TYPE_MASK {
                        TAG_TYPE_DWORD => {
                            dwval = SdbReadDWORDTag(pdb, child, 0);
                            if dwval != (*attr).__bindgen_anon_1.dwattr {
                                return FALSE as BOOL;
                            }
                        }
                        TAG_TYPE_STRINGREF => {
                            lpval = SdbGetStringTagPtr(pdb, child) as *mut WCHAR;
                            if lpval.is_null() || _wcsicmp((*attr).__bindgen_anon_1.lpattr, lpval) != 0 {
                                return FALSE as BOOL;
                            }
                        }
                        TAG_TYPE_QWORD => {
                            qwval = SdbReadQWORDTag(pdb, child, 0);
                            if qwval != (*attr).__bindgen_anon_1.qwattr {
                                return FALSE as BOOL;
                            }
                        }
                        _ => {
                            SHIM_WARN!(b"Unhandled type 0x%x MATCHING_FILE\n\0".as_ptr() as PCSTR, ((tag as u32) & TAG_TYPE_MASK));
                            return FALSE as BOOL;
                        }
                    }
                }
                n = n.wrapping_add(1);
            }
            if n == attr_count {
                SHIM_WARN!(b"Unhandled tag %ws in MATCHING_FILE\n\0".as_ptr() as PCSTR, SdbTagToString(tag));
            }
        }
        child = SdbGetNextChild(pdb, matching_file, child);
    }
    return (TRUE) as BOOL;
}
unsafe extern "system" fn SdbpMatchExe(
    pdb: PDB,
    exe: TAGID,
    dir: *const WCHAR,
    main_attribs: PATTRINFO,
    main_attr_count: DWORD,
) -> BOOL {
    let mut FullPathName: RTL_UNICODE_STRING_BUFFER = core::mem::zeroed();
    let _ = 0;
    let mut FullPathBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut UnicodeDir: _UNICODE_STRING = core::mem::zeroed();
    let mut matching_file: TAGID = 0;
    let mut attribs: PATTRINFO = core::ptr::null_mut();
    let mut attr_count: DWORD = 0;
    let mut IsMatch: BOOL = FALSE as BOOL;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeDir), dir);
    RtlInitBuffer(
        core::ptr::addr_of_mut!(FullPathName.ByteBuffer),
        core::ptr::addr_of_mut!(FullPathBuffer) as PUCHAR,
        core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as ULONG,
    );

    'cleanup: {
        matching_file = SdbFindFirstTag(pdb, exe, TAG_MATCHING_FILE as TAG);
        loop {
            if !(matching_file != TAGID_NULL as TAGID) {
                break;
            }
            'next: {
                let mut tagName: TAGID =
                    SdbFindFirstTag(pdb, matching_file, TAG_NAME as TAG);
                let mut Name: _UNICODE_STRING = core::mem::zeroed();
                let mut Len: USHORT = 0;

                RtlInitUnicodeString(
                    core::ptr::addr_of_mut!(Name),
                    SdbGetStringTagPtr(pdb, tagName) as PCWSTR,
                );

                if Name.Buffer.is_null() {
                    break 'cleanup;
                }

                /* An '*' here means use the main executable' */
                if wcscmp(Name.Buffer, RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr()) == 0 {
                    /* We already have these attributes, so we do not need to retrieve them */
                    if SdbpMatchFileAttributes(pdb, matching_file, main_attribs, main_attr_count) == 0 {
                        break 'cleanup;
                    }
                    break 'next;
                }

                /* Technically, one UNICODE_NULL and one path separator. */
                Len = (UnicodeDir.Length as usize
                    + Name.Length as usize
                    + core::mem::size_of::<WCHAR>()
                    + core::mem::size_of_val(&UNICODE_NULL) as usize) as USHORT;
                if !NT_SUCCESS!(RtlEnsureBufferSize(
                    RTL_SKIP_BUFFER_COPY as ULONG,
                    core::ptr::addr_of_mut!(FullPathName.ByteBuffer),
                    Len as ULONG,
                )) {
                    break 'cleanup;
                }

                if (Len as SIZE_T) > FullPathName.ByteBuffer.Size {
                    break 'cleanup;
                }

                RtlInitEmptyUnicodeString(
                    core::ptr::addr_of_mut!(FullPathName.String),
                    FullPathName.ByteBuffer.Buffer as PWCHAR,
                    FullPathName.ByteBuffer.Size as USHORT,
                );

                RtlCopyUnicodeString(
                    core::ptr::addr_of_mut!(FullPathName.String),
                    core::ptr::addr_of_mut!(UnicodeDir),
                );
                RtlAppendUnicodeToString(
                    core::ptr::addr_of_mut!(FullPathName.String),
                    RUSTCONV_WSTR_B7FC02907B1F.as_ptr(),
                );
                RtlAppendUnicodeStringToString(
                    core::ptr::addr_of_mut!(FullPathName.String),
                    core::ptr::addr_of_mut!(Name),
                );

                /* If the file does not exist, do not bother trying to read it's attributes */
                if SdbpFileExists(FullPathName.String.Buffer as LPCWSTR) == 0 {
                    break 'cleanup;
                }

                /* Do we have some attributes from the previous iteration? */
                if !attribs.is_null() {
                    SdbFreeFileAttributes(attribs);
                }

                if SdbGetFileAttributes(
                    FullPathName.String.Buffer as LPCWSTR,
                    core::ptr::addr_of_mut!(attribs),
                    core::ptr::addr_of_mut!(attr_count),
                ) == 0
                {
                    break 'cleanup;
                }

                if SdbpMatchFileAttributes(pdb, matching_file, attribs, attr_count) == 0 {
                    break 'cleanup;
                }
            }
            matching_file = SdbFindNextTag(pdb, exe, matching_file);
        }

        IsMatch = TRUE as BOOL;
    }

    /* Cleanup: */
    RtlFreeBuffer(core::ptr::addr_of_mut!(FullPathName.ByteBuffer));
    if !attribs.is_null() {
        SdbFreeFileAttributes(attribs);
    }

    return IsMatch;
}

unsafe extern "C" fn SdbpAddDatabaseGuid(pdb: PDB, result: PSDBQUERYRESULT) {
    let mut n: usize;

    n = 0 as usize;
    while n < _countof!((*result).rgGuidDB) as usize {
        if memcmp(
            (core::ptr::addr_of_mut!((*result).rgGuidDB) as *mut GUID).add(n) as *const core::ffi::c_void,
            core::ptr::addr_of!((*pdb).database_id) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        ) == 0
        {
            return;
        }

        if (*result).dwCustomSDBMap & ((1 as DWORD) << n) != 0 {
            n += 1;
            continue;
        }

        memcpy(
            (core::ptr::addr_of_mut!((*result).rgGuidDB) as *mut GUID).add(n) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pdb).database_id) as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        );
        (*result).dwCustomSDBMap |= (1 as DWORD) << n;
        return;
    }
}

unsafe extern "C" fn SdbpAddSingleLayerMatch(layer: TAGREF, result: PSDBQUERYRESULT) -> BOOL {
    let mut n: size_t = 0;
    while n < (*result).dwLayerCount as size_t {
        if (*result).atrLayers[n as usize] == layer {
            return FALSE as BOOL;
        }
        n = n.wrapping_add(1);
    }

    if n as usize >= (_countof!((*result).atrLayers) as usize) {
        return FALSE as BOOL;
    }

    (*result).atrLayers[n as usize] = layer;
    (*result).dwLayerCount = (*result).dwLayerCount.wrapping_add(1);

    return TRUE as BOOL;
}

unsafe extern "C" fn SdbpAddNamedLayerMatch(hsdb: HSDB, layerName: PCWSTR, result: PSDBQUERYRESULT) -> BOOL {
    let mut database: TAGID;
    let mut layer: TAGID;
    let mut tr: TAGREF;
    let pdb: PDB = (*hsdb).pdb;

    database = SdbFindFirstTag(pdb, TAGID_ROOT, TAG_DATABASE as TAG);
    if database == TAGID_NULL {
        return FALSE as BOOL;
    }

    layer = SdbFindFirstNamedTag(pdb, database, TAG_LAYER, TAG_NAME, layerName as LPCWSTR);
    if layer == TAGID_NULL {
        return FALSE as BOOL;
    }

    tr = 0;
    if SdbTagIDToTagRef(hsdb, pdb, layer, core::ptr::addr_of_mut!(tr)) == 0 {
        return (FALSE) as BOOL;
    }

    if SdbpAddSingleLayerMatch(tr, result) == 0 {
        return (FALSE) as BOOL;
    }

    SdbpAddDatabaseGuid(pdb, result);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn SdbpAddExeLayers(hsdb: HSDB, pdb: PDB, tagExe: TAGID, result: PSDBQUERYRESULT) {
    let mut layer: TAGID = SdbFindFirstTag(pdb, tagExe, TAG_LAYER as TAG);

    while layer != TAGID_NULL as TAGID {
        let mut tr: TAGREF = 0;
        let mut layerIdTag: TAGID = SdbFindFirstTag(pdb, layer, TAG_LAYER_TAGID as TAG);
        let mut tagId: DWORD = SdbReadDWORDTag(pdb, layerIdTag, TAGID_NULL as DWORD);

        if layerIdTag != TAGID_NULL as TAGID
            && tagId != TAGID_NULL as DWORD
            && SdbTagIDToTagRef(hsdb, pdb, tagId as TAGID, core::ptr::addr_of_mut!(tr)) != 0
        {
            SdbpAddSingleLayerMatch(tr, result);
        } else {
            /* Try a name lookup */
            let mut layerTag: TAGID = SdbFindFirstTag(pdb, layer, TAG_NAME as TAG);
            if layerTag != TAGID_NULL as TAGID {
                let mut layerName: LPCWSTR = SdbGetStringTagPtr(pdb, layerTag) as LPCWSTR;
                if !layerName.is_null() {
                    SdbpAddNamedLayerMatch(hsdb, layerName as PCWSTR, result);
                }
            }
        }

        layer = SdbFindNextTag(pdb, tagExe, layer);
    }
}

unsafe extern "C" fn SdbpAddExeMatch(hsdb: HSDB, pdb: PDB, tagExe: TAGID, result: PSDBQUERYRESULT) {
    let mut n: size_t = 0;
    let mut tr: TAGREF = 0;

    if SdbTagIDToTagRef(hsdb, pdb, tagExe, core::ptr::addr_of_mut!(tr)) == 0 {
        return;
    }

    n = 0;
    while n < (*result).dwExeCount as size_t {
        if (*result).atrExes[n as usize] == tr {
            return;
        }
        n = n.wrapping_add(1);
    }

    if n >= _countof!((*result).atrExes) as size_t {
        return;
    }

    (*result).atrExes[n as usize] = tr;
    (*result).dwExeCount = (*result).dwExeCount.wrapping_add(1);

    SdbpAddExeLayers(hsdb, pdb, tagExe, result);

    SdbpAddDatabaseGuid(pdb, result);
}

unsafe extern "C" fn SdbpAddLayerMatches(
    hsdb: HSDB,
    pwszLayers: PWSTR,
    pdwBytes: DWORD,
    result: PSDBQUERYRESULT,
) -> ULONG {
    let mut start: PWSTR = pwszLayers;
    let mut p: PWSTR = core::ptr::null_mut();
    let mut Added: ULONG = 0;

    let end: PWSTR = pwszLayers.offset((pdwBytes as usize / core::mem::size_of::<WCHAR>()) as isize);
    while start < end
        && (*start == 0x0021u16
            || *start == 0x0023u16
            || *start == 0x0020u16
            || *start == 0x0009u16)
    {
        start = start.offset(1);
    }

    if start == end {
        return 0;
    }

    loop {
        while *start == 0x0020u16 || *start == 0x0009u16 {
            start = start.offset(1);
        }

        if *start == UNICODE_NULL {
            break;
        }
        p = wcspbrk(start as *const u16, RUSTCONV_WSTR_731D7EC48EEF.as_ptr()) as PWSTR;

        if !p.is_null() {
            *p = UNICODE_NULL;
        }

        if SdbpAddNamedLayerMatch(hsdb, start as PCWSTR, result) != 0 {
            Added = Added.wrapping_add(1);
        }

        start = p.offset(1 as isize);
        if !(start < end && !p.is_null()) {
            break;
        }
    }

    return Added;
}

unsafe extern "C" fn SdbpPropagateEnvLayers(hsdb: HSDB, Environment: LPWSTR, Result: PSDBQUERYRESULT) -> BOOL {
    let mut EnvValue: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut Buffer: [WCHAR; MAX_LAYER_LENGTH] = [0; MAX_LAYER_LENGTH];

    RtlInitEmptyUnicodeString(
        &raw mut EnvValue,
        &raw mut Buffer as PWCHAR,
        core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH]>() as USHORT,
    );

    Status = RtlQueryEnvironmentVariable_U(Environment, &raw const EnvKey, &raw mut EnvValue);

    if !NT_SUCCESS!(Status) {
        return (FALSE) as BOOL;
    }

    return (SdbpAddLayerMatches(
        hsdb,
        &raw mut Buffer as PWSTR,
        EnvValue.Length as DWORD,
        Result,
    ) > 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbInitDatabase(mut flags: DWORD, path: LPCWSTR) -> HSDB {
    static shim: [WCHAR; 13] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x006D, 0x0061, 0x0069, 0x006E, 0x002E, 0x0073,
    0x0064, 0x0062, 0x0000,
];
    static msi: [WCHAR; 13] = [
    0x005Cu16, 0x006D, 0x0073, 0x0069, 0x006D, 0x0061, 0x0069, 0x006E, 0x002E, 0x0073,
    0x0064, 0x0062, 0x0000,
];
    static drivers: [WCHAR; 13] = [
    0x005Cu16, 0x0064, 0x0072, 0x0076, 0x006D, 0x0061, 0x0069, 0x006E, 0x002E, 0x0073,
    0x0064, 0x0062, 0x0000,
];
    let mut name: LPCWSTR = core::ptr::null();
    let mut buffer: [WCHAR; 128] = [0; 128];
    let mut hsdb: HSDB;

    hsdb = SdbAlloc!(core::mem::size_of::<SDB>() as SIZE_T) as HSDB;
    if hsdb.is_null() {
        return core::ptr::null_mut();
    }
    (*hsdb).auto_loaded = 0;

    /* Check for predefined databases */
    if ((flags & HID_DATABASE_TYPE_MASK) != 0) && path.is_null() {
        let sw: DWORD = flags & HID_DATABASE_TYPE_MASK;
        if sw == SDB_DATABASE_MAIN_SHIM {
            name = shim.as_ptr();
        } else if sw == SDB_DATABASE_MAIN_MSI {
            name = msi.as_ptr();
        } else if sw == SDB_DATABASE_MAIN_DRIVERS {
            name = drivers.as_ptr();
        } else {
            SdbReleaseDatabase(hsdb);
            return core::ptr::null_mut();
        }
        SdbGetAppPatchDir(
            core::ptr::null_mut(),
            buffer.as_mut_ptr(),
            _countof!(buffer) as DWORD,
        );
        StringCchCatW(buffer.as_mut_ptr(), _countof!(buffer) as usize, name);
        flags = HID_DOS_PATHS;
    }

    (*hsdb).pdb = SdbOpenDatabase(
        if !path.is_null() {
            path
        } else {
            buffer.as_mut_ptr() as LPCWSTR
        },
        (flags & 0xF).wrapping_sub(1) as PATH_TYPE,
    );

    /* If database could not be loaded, a handle doesn't make sense either */
    if (*hsdb).pdb.is_null() {
        SdbReleaseDatabase(hsdb);
        return core::ptr::null_mut();
    }

    return hsdb;
}

#[no_mangle]
pub unsafe extern "system" fn SdbReleaseDatabase(hsdb: HSDB) {
    if !hsdb.is_null() {
        SdbCloseDatabase((*hsdb).pdb);
        SdbFree!(hsdb as LPVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetMatchingExe(mut hsdb: HSDB, path: LPCWSTR, module_name: LPCWSTR, env: LPCWSTR, flags: DWORD, result: PSDBQUERYRESULT) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut database: TAGID;
    let mut iter: TAGID;
    let mut name: TAGID;
    let mut attribs: PATTRINFO = core::ptr::null_mut();
    let mut attr_count: DWORD = 0;
    let mut DosApplicationName: RTL_UNICODE_STRING_BUFFER = core::mem::zeroed();
    DosApplicationName.String.Length = 0 as USHORT;
    let mut DosPathBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut PathType: ULONG = 0 as ULONG;
    let mut file_name: LPWSTR = core::ptr::null_mut();
    let mut wszLayers: [WCHAR; MAX_LAYER_LENGTH] = [0; MAX_LAYER_LENGTH];
    let mut dwSize: DWORD;
    let mut pdb: PDB;

    /* Load default database if one is not specified */
    if hsdb.is_null() {
        /* To reproduce windows behaviour HID_DOS_PATHS needs
         * to be specified when loading default database */
        hsdb = SdbInitDatabase(
            (HID_DOS_PATHS as DWORD | SDB_DATABASE_MAIN_SHIM as DWORD),
            core::ptr::null(),
        );
        if !hsdb.is_null() {
            (*hsdb).auto_loaded = TRUE as BOOL;
        }
    }

    ZeroMemory!(
        result as *mut core::ffi::c_void,
        core::mem::size_of::<SDBQUERYRESULT>() as usize,
    );

    /* No database could be loaded */
    if hsdb.is_null() || path.is_null() {
        return FALSE as BOOL;
    }

    /* We do not support multiple db's yet! */
    pdb = (*hsdb).pdb;

    'cleanup: {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(DosApplicationName.String), path as PCWSTR);
        RtlInitBuffer(
            core::ptr::addr_of_mut!(DosApplicationName.ByteBuffer),
            core::ptr::addr_of_mut!(DosPathBuffer) as *mut WCHAR as PUCHAR,
            core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as ULONG,
        );
        if !NT_SUCCESS!(RtlEnsureBufferSize(
            RTL_SKIP_BUFFER_COPY as ULONG,
            core::ptr::addr_of_mut!(DosApplicationName.ByteBuffer),
            DosApplicationName.String.MaximumLength as ULONG,
        )) {
            SHIM_ERR!(b"Failed to convert allocate buffer.\n\0".as_ptr() as *const i8);
            break 'cleanup;
        }
        /* Update the internal buffer to contain the string */
        memcpy(
            DosApplicationName.ByteBuffer.Buffer as *mut core::ffi::c_void,
            path as *const core::ffi::c_void,
            DosApplicationName.String.MaximumLength as usize,
        );
        /* Make sure the string uses our internal buffer (we want to modify the buffer,
            and RtlNtPathNameToDosPathName does not always modify the String to point to the Buffer)! */
        DosApplicationName.String.Buffer = DosApplicationName.ByteBuffer.Buffer as PWSTR;

        if !NT_SUCCESS!(RtlNtPathNameToDosPathName(
            0 as ULONG,
            core::ptr::addr_of_mut!(DosApplicationName),
            core::ptr::addr_of_mut!(PathType),
            core::ptr::null_mut(),
        )) {
            SHIM_ERR!(b"Failed to convert %S to DOS Path.\n\0".as_ptr() as *const i8, path);
            break 'cleanup;
        }

        /* Extract file name */
        file_name = wcsrchr(
            DosApplicationName.String.Buffer as *const wchar_t,
            '\\' as i32 as wchar_t,
        ) as LPWSTR;
        if file_name.is_null() {
            SHIM_ERR!(
                b"Failed to find Exe name in %wZ.\n\0".as_ptr() as *const i8,
                core::ptr::addr_of_mut!(DosApplicationName.String),
            );
            break 'cleanup;
        }

        /* We will use the buffer for exe name and directory. */
        let fresh31 = file_name;
        file_name = file_name.offset(1);
        *fresh31 = UNICODE_NULL as WCHAR;

        /* DATABASE is list TAG which contains all executables */
        database = SdbFindFirstTag(
            pdb,
            TAGID_ROOT as TAGID,
            TAG_DATABASE as TAG,
        );
        if database == TAGID_NULL as TAGID {
            break 'cleanup;
        }

        /*
         * Instead of brute-force searching the 'entire' DB, we should:
         * 1. Iterate over the Index TAG_EXE.TAG_WILDCARD_NAME
         * 2. Search the index TAG_EXE.TAG_NAME (fallback to SdbFindFirstNamedTag)
         * 3. Search the index TAG_EXE.TAG_16BIT_MODULE_NAME (fallback to SdbFindFirstNamedTag)
         */

        /* EXE is list TAG which contains data required to match executable */
        iter = SdbFindFirstTag(pdb, database, TAG_EXE as TAG);
        loop {
            if !(iter != TAGID_NULL as TAGID) {
                break;
            }
            'body: {
                name = SdbFindFirstTag(pdb, iter, TAG_NAME as TAG);
                let mut foundName: LPWSTR = SdbGetStringTagPtr(pdb, name);
                /* If this is a malformed DB, (no TAG_NAME), we should not crash. */
                if foundName.is_null() {
                    break 'body;
                }

                /* We only support full wildcard names for now */
                let mut isWildcard: BOOL = (*foundName.offset(0 as isize) as i32
                    == '*' as i32
                    && *foundName.offset(1 as isize) as i32
                        == UNICODE_NULL as i32) as BOOL;

                /* Check if exe name matches */
                if isWildcard != 0
                    || _wcsicmp(
                        foundName as *const wchar_t,
                        file_name as *const wchar_t,
                    ) == 0
                {
                    /* Get information about executable required to match it with database entry */
                    if attribs.is_null() {
                        if SdbGetFileAttributes(
                            path,
                            core::ptr::addr_of_mut!(attribs),
                            core::ptr::addr_of_mut!(attr_count),
                        ) == 0
                        {
                            break 'cleanup;
                        }
                    }

                    /* We have a null terminator before the application name, so DosApplicationName only contains the path. */
                    if SdbpMatchExe(
                        pdb,
                        iter,
                        DosApplicationName.String.Buffer as *const WCHAR,
                        attribs,
                        attr_count,
                    ) != 0
                    {
                        ret = TRUE as BOOL;
                        SdbpAddExeMatch(hsdb, pdb, iter, result);
                    }
                }
            }
            iter = SdbFindNextTag(pdb, database, iter);
        }

        /* Restore the full path. */
        file_name = file_name.offset(-1);
        *file_name = 0x005Cu16;

        dwSize = core::mem::size_of::<[WCHAR; MAX_LAYER_LENGTH]>() as DWORD;
        if SdbGetPermLayerKeys(
            DosApplicationName.String.Buffer as PCWSTR,
            core::ptr::addr_of_mut!(wszLayers) as *mut WCHAR as PWSTR,
            core::ptr::addr_of_mut!(dwSize),
            (GPLK_MACHINE as DWORD | GPLK_USER as DWORD),
        ) != 0
        {
            SdbpAddLayerMatches(
                hsdb,
                core::ptr::addr_of_mut!(wszLayers) as *mut WCHAR as PWSTR,
                dwSize,
                result,
            );
            ret = TRUE as BOOL;
        }

        if flags & (SDBGMEF_IGNORE_ENVIRONMENT as DWORD) == 0 {
            if SdbpPropagateEnvLayers(hsdb, env as LPWSTR, result) != 0 {
                ret = TRUE as BOOL;
                (*result).dwFlags |= SHIMREG_HAS_ENVIRONMENT as DWORD;
            }
        }
    }

    RtlFreeBuffer(core::ptr::addr_of_mut!(DosApplicationName.ByteBuffer));
    if !attribs.is_null() {
        SdbFreeFileAttributes(attribs);
    }
    if (*hsdb).auto_loaded != 0 {
        SdbReleaseDatabase(hsdb);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetAppPatchDir(hsdb: HSDB, path: LPWSTR, size: DWORD) -> HRESULT {
    static mut default_dir: *mut WCHAR = core::ptr::null_mut();
    static szAppPatch: [u16; 10] = [0x005Cu16, 0x0041, 0x0070, 0x0070, 0x0050, 0x0061, 0x0074, 0x0063, 0x0068, 0x0000];

    /* In case function fails, path holds empty string */
    if size > 0 {
        *path = 0;
    }

    if default_dir.is_null() {
        let mut tmp: *mut WCHAR = core::ptr::null_mut();
        let mut hr: HRESULT = E_FAIL;
        let mut len: UINT = (GetSystemWindowsDirectoryW(core::ptr::null_mut(), 0) as DWORD)
            .wrapping_add(SdbpStrlen(core::ptr::addr_of!(szAppPatch) as PCWSTR)) as UINT;
        tmp = SdbAlloc!(
            ((len.wrapping_add(1)) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut WCHAR;
        if !tmp.is_null() {
            let mut r: UINT = GetSystemWindowsDirectoryW(tmp, len.wrapping_add(1));
            if r != 0 && r < len {
                hr = StringCchCatW(
                    tmp,
                    (len.wrapping_add(1)) as _,
                    core::ptr::addr_of!(szAppPatch) as *const WCHAR,
                );
                if hr >= 0 {
                    if InterlockedCompareExchangePointer(
                        core::ptr::addr_of_mut!(default_dir) as *mut *mut core::ffi::c_void,
                        tmp as *mut core::ffi::c_void,
                        core::ptr::null_mut(),
                    )
                    .is_null()
                    {
                        tmp = core::ptr::null_mut();
                    }
                }
            }
            if !tmp.is_null() {
                SdbFree!(tmp);
            }
        }
        if default_dir.is_null() {
            SHIM_ERR!(
                b"Unable to obtain default AppPatch directory (0x%x)\n\0".as_ptr() as *const i8,
                hr,
            );
            return hr;
        }
    }

    if hsdb.is_null() {
        return StringCchCopyW(path, (size) as _, default_dir);
    } else {
        SHIM_ERR!(b"Unimplemented for hsdb != NULL\n\0".as_ptr() as *const i8);
        return E_NOTIMPL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SdbTagRefToTagID(hsdb: HSDB, trWhich: TAGREF, ppdb: *mut PDB, ptiWhich: *mut TAGID) -> BOOL {
    if (trWhich & (0xf0000000 as TAGREF)) != 0 {
        SHIM_ERR!(b"Multiple shim databases not yet implemented!\n\0".as_ptr() as *const i8);
        if !ppdb.is_null() {
            *ppdb = core::ptr::null_mut();
        }
        if !ptiWhich.is_null() {
            *ptiWhich = TAG_NULL as TAGID;
        }
        return FALSE as BOOL;
    }

    /* There seems to be no range checking on trWhich.. */
    if !ppdb.is_null() {
        *ppdb = (*hsdb).pdb;
    }
    if !ptiWhich.is_null() {
        *ptiWhich = (trWhich & (0x0fffffff as TAGREF)) as TAGID;
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbTagIDToTagRef(hsdb: HSDB, pdb: PDB, tiWhich: TAGID, ptrWhich: *mut TAGREF) -> BOOL {
    if pdb != (*hsdb).pdb {
        SHIM_ERR!(b"Multiple shim databases not yet implemented!\n\0".as_ptr() as *const i8);
        if !ptrWhich.is_null() {
            *ptrWhich = TAGREF_NULL;
        }
        return FALSE as BOOL;
    }

    if !ptrWhich.is_null() {
        *ptrWhich = tiWhich & 0x0fffffff;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbPackAppCompatData(mut hsdb: HSDB, pQueryResult: PSDBQUERYRESULT, ppData: *mut PVOID, pdwSize: *mut DWORD) -> BOOL {
    let mut pData: *mut ShimData = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut n: DWORD;
    let mut bCloseDatabase: BOOL = FALSE as BOOL;

    if pQueryResult.is_null() || ppData.is_null() || pdwSize.is_null() {
        SHIM_WARN!(b"Invalid params: %p, %p, %p\n\0".as_ptr() as *const i8, pQueryResult, ppData, pdwSize);
        return FALSE as BOOL;
    }

    pData = RtlAllocateHeap(RtlGetProcessHeap!(), HEAP_ZERO_MEMORY as ULONG, core::mem::size_of::<ShimData>() as SIZE_T) as *mut ShimData;
    if pData.is_null() {
        SHIM_WARN!(b"Unable to allocate %d bytes\n\0".as_ptr() as *const i8, core::mem::size_of::<ShimData>());
        return FALSE as BOOL;
    }

    GetSystemWindowsDirectoryW(
        core::ptr::addr_of_mut!((*pData).szModule) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as UINT,
    );
    hr = StringCchCatW(
        core::ptr::addr_of_mut!((*pData).szModule) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as size_t,
        RUSTCONV_WSTR_D86033D68ADE.as_ptr(),
    );
    if !(hr >= 0 as HRESULT) {
        SHIM_ERR!(b"Unable to append module name (0x%x)\n\0".as_ptr() as *const i8, hr);
        RtlFreeHeap(RtlGetProcessHeap!(), 0 as ULONG, pData as PVOID);
        return FALSE as BOOL;
    }

    (*pData).dwSize = core::mem::size_of::<ShimData>() as DWORD;
    (*pData).dwMagic = SHIMDATA_MAGIC as DWORD;
    (*pData).Query = *pQueryResult;
    (*pData).dwRosProcessCompatVersion = 0 as DWORD;
    (*pData).szLayer[0 as usize] = UNICODE_NULL as WCHAR; /* TODO */

    SHIM_INFO!(
        b"\ndwFlags    0x%x\ndwMagic    0x%x\ntrExe      0x%x\ntrLayer    0x%x\n\0".as_ptr() as *const i8,
        (*pData).Query.dwFlags,
        (*pData).dwMagic,
        (*pData).Query.atrExes[0 as usize],
        (*pData).Query.atrLayers[0 as usize],
    );

    /* Database List */
    /* 0x0 {GUID} NAME: Use to open HSDB */
    if hsdb.is_null() {
        hsdb = SdbInitDatabase(
            (HID_DOS_PATHS | SDB_DATABASE_MAIN_SHIM) as DWORD,
            core::ptr::null(),
        );
        bCloseDatabase = TRUE as BOOL;
    }

    n = 0 as DWORD;
    while n < (*pQueryResult).dwLayerCount {
        let mut dwValue: DWORD = 0 as DWORD;
        let mut dwType: DWORD = 0;
        let mut dwValueSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        SHIM_INFO!(
            b"Layer 0x%x\n\0".as_ptr() as *const i8,
            (*pQueryResult).atrLayers[n as usize],
        );

        if SdbQueryData(
            hsdb,
            (*pQueryResult).atrLayers[n as usize],
            RUSTCONV_WSTR_B28803D501F2.as_ptr(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(dwValue) as LPVOID,
            core::ptr::addr_of_mut!(dwValueSize),
        ) == ERROR_SUCCESS as DWORD
            && dwType == REG_DWORD as DWORD
            && dwValueSize == core::mem::size_of::<DWORD>() as DWORD
        {
            if dwValue != REACTOS_COMPATVERSION_IGNOREMANIFEST as DWORD {
                dwValue = (dwValue % 100 as DWORD)
                    | ((dwValue / 100 as DWORD) << 8);
            }
            if dwValue > (*pData).dwRosProcessCompatVersion {
                (*pData).dwRosProcessCompatVersion = dwValue;
            }
        }
        n = n.wrapping_add(1);
    }

    if (*pData).dwRosProcessCompatVersion != 0 {
        SHIM_INFO!(
            b"Setting ProcessCompatVersion 0x%x\n\0".as_ptr() as *const i8,
            (*pData).dwRosProcessCompatVersion,
        );
    }

    if bCloseDatabase != 0 {
        SdbReleaseDatabase(hsdb);
    }

    *ppData = pData as PVOID;
    *pdwSize = (*pData).dwSize;

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbUnpackAppCompatData(hsdb: HSDB, pszImageName: LPCWSTR, pData: PVOID, pQueryResult: PSDBQUERYRESULT) -> BOOL {
    let mut pShimData: *mut ShimData = pData as *mut ShimData;

    if pShimData.is_null()
        || (*pShimData).dwMagic != SHIMDATA_MAGIC as DWORD
        || ((*pShimData).dwSize as usize) < core::mem::size_of::<ShimData>()
    {
        return FALSE as BOOL;
    }

    if pQueryResult.is_null() {
        return FALSE as BOOL;
    }

    /* szLayer? */

    *pQueryResult = (*pShimData).Query;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SdbGetAppCompatDataSize(pData: *mut ShimData) -> DWORD {
    if pData.is_null() || (*pData).dwMagic != SHIMDATA_MAGIC {
        return 0;
    }

    return (*pData).dwSize;
}

#[no_mangle]
pub unsafe extern "system" fn SdbQueryDataEx(
    hsdb: HSDB,
    trWhich: TAGREF,
    lpszDataName: LPCWSTR,
    lpdwDataType: LPDWORD,
    lpBuffer: LPVOID,
    lpcbBufferSize: LPDWORD,
    ptrData: *mut TAGREF,
) -> DWORD {
    let mut pdb: PDB = core::ptr::null_mut();
    let mut tiWhich: TAGID = 0;
    let mut tiData: TAGID = 0;
    let mut dwResult: DWORD = 0;

    if SdbTagRefToTagID(
        hsdb,
        trWhich,
        core::ptr::addr_of_mut!(pdb),
        core::ptr::addr_of_mut!(tiWhich),
    ) == 0
    {
        SHIM_WARN!(b"Unable to translate trWhich=0x%x\n\0".as_ptr() as *const i8, trWhich);
        return ERROR_NOT_FOUND as DWORD;
    }

    dwResult = SdbQueryDataExTagID(
        pdb,
        tiWhich,
        lpszDataName,
        lpdwDataType,
        lpBuffer,
        lpcbBufferSize,
        core::ptr::addr_of_mut!(tiData),
    );

    if dwResult == ERROR_SUCCESS as DWORD && !ptrData.is_null() {
        SdbTagIDToTagRef(hsdb, pdb, tiData, ptrData);
    }

    return dwResult;
}

#[no_mangle]
pub unsafe extern "system" fn SdbQueryData(
    hsdb: HSDB,
    trWhich: TAGREF,
    lpszDataName: LPCWSTR,
    lpdwDataType: LPDWORD,
    lpBuffer: LPVOID,
    lpcbBufferSize: LPDWORD,
) -> DWORD {
    SdbQueryDataEx(hsdb, trWhich, lpszDataName, lpdwDataType, lpBuffer, lpcbBufferSize, core::ptr::null_mut())
}
