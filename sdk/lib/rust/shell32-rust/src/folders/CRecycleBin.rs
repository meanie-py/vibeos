// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_49A455CD42F8: [WCHAR; 57] = [0x0041, 0x0070, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x005C, 0x0053, 0x0063, 0x0068, 0x0065, 0x006D, 0x0065, 0x0073, 0x005C, 0x0041, 0x0070, 0x0070, 0x0073, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0045, 0x006D, 0x0070, 0x0074, 0x0079, 0x0052, 0x0065, 0x0063, 0x0079, 0x0063, 0x006C, 0x0065, 0x0042, 0x0069, 0x006E, 0x005C, 0x002E, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_604114ADC8B4: [WCHAR; 13] = [0x004E, 0x0075, 0x006B, 0x0065, 0x004F, 0x006E, 0x0044, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_69A283E639E4: [WCHAR; 3] = [0x0025, 0x0075, 0x0000];
pub static CPP_COM_WSTR_9618E75FEC6D: [WCHAR; 10] = [0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];
pub static CPP_COM_WSTR_9855CDA098DC: [WCHAR; 2] = [0x0058, 0x0000];
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_BCF17C907FBF: [WCHAR; 69] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0042, 0x0069, 0x0074, 0x0042, 0x0075, 0x0063, 0x006B, 0x0065, 0x0074, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x005C, 0x0000];
pub static CPP_COM_WSTR_C7A5956868FE: [WCHAR; 12] = [0x004D, 0x0061, 0x0078, 0x0043, 0x0061, 0x0070, 0x0061, 0x0063, 0x0069, 0x0074, 0x0079, 0x0000];
pub static CPP_COM_WSTR_F51AA22D711D: [WCHAR; 5] = [0x0043, 0x003A, 0x005C, 0x0058, 0x0000];

// The island/bindgen substrate (via `prelude`) is the ONLY substrate source for this TU. The
// hand-written `types`/`win32`/`shellcom` globs the assembler emits alongside it are dropped
// rather than joined: carrying both makes every shared name ambiguous (E0659) at 70+ sites.
// See prelude.rs, and CNetFolder.rs for the same note.
use core::ffi::c_void;
use crate::prelude::*;
use crate::recycle_bin::{IDC_PROPERTIES};

// ── PIDL static-inline helpers (wine/pidl.h) ──
// Defined here rather than imported from `cfs_folder`: that module predates the island and its
// LPCITEMIDLIST is the hand-written `shellcom::ITEMIDLIST`, so importing across the substrate
// boundary makes every call site a nominal mismatch. Two lines each; a local copy is cheaper
// and cannot drift, because the C is a header inline with no symbol either way.
#[inline] unsafe fn _ILGetType(pidl: LPCITEMIDLIST) -> BYTE {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 { *(pidl as *const u8).add(2) } else { 0 }
}
#[inline] unsafe fn _ILGetFSType(pidl: LPCITEMIDLIST) -> BYTE {
    let t = _ILGetType(pidl);
    if (t as u32 & PT_FOLDERTYPEMASK) == PT_FS { t } else { 0 }
}
use crate::shell32::sdk_consts::{PID_STG_SIZE, SHCONTF_INCLUDEHIDDEN};
use crate::drives_folder::{PID_STG_NAME, PID_STG_STORAGETYPE};
use crate::cfs_folder::PID_STG_WRITETIME;
use crate::file_sys_enum::{SHCONTF_FOLDERS, SHCONTF_NONFOLDERS};
use crate::move_to_menu::SHGDN_INFOLDER;
use crate::recycle_bin::SND_FILENAME;
use crate::user_notification::SND_NODEFAULT;
use crate::shellcom::STRRET_WSTR;
use crate::wine::pidl::{_ILCopyaPidl, _ILFreeaPidl};
use crate::shell32::shlfileop::SHELL32_FileOperation;

// mmsystem.h SND_* — SND_ASYNC has no home in the crate yet (SND_FILENAME and SND_NODEFAULT
// each do, in the module that first needed them).
pub const SND_ASYNC: DWORD = 0x0001;

unsafe extern "system" {
    /// winmm.dll. shell32 links winmm for the "empty recycle bin" sound.
    fn PlaySoundW(pszSound: LPCWSTR, hmod: HMODULE, fdwSound: DWORD) -> BOOL;
    /// shlwapi.dll.
    fn StrRetToBufW(pstr: *mut STRRET, pidl: LPCITEMIDLIST, pszBuf: LPWSTR, cchBuf: UINT) -> HRESULT;
}

unsafe extern "system" {
    // shellrecyclebin/recyclebin.h:94-97. Declared inside that header's `extern "C"` block, so
    // the symbol is unmangled — but it carries WINAPI, so it is STDCALL, not cdecl. On i386
    // that is the difference between `_DeleteFileToRecycleBinW@4` and `_DeleteFileToRecycleBinW`,
    // and it is invisible on amd64/arm64.
    fn DeleteFileToRecycleBinW(FileName: LPCWSTR) -> BOOL;
}

/// `strsafe.h` `StringCbCopyW` — an INLINE, so there is no symbol to import. Byte count in,
/// character count out; the truncation contract is the same as StringCchCopyW's.
#[inline]
unsafe fn StringCbCopyW(dst: LPWSTR, cbDst: SIZE_T, src: LPCWSTR) -> HRESULT {
    StringCchCopyW(dst, (cbDst / core::mem::size_of::<WCHAR>() as SIZE_T) as usize, src)
}

/// `recyclebin.h:50` `#define IRecycleBinFileFromHDELFILE(hDF) ((IRecycleBinFile*)(hDF))` — a
/// pure reinterpret of the opaque handle.
#[inline]
unsafe fn IRecycleBinFileFromHDELFILE(hDF: HDELFILE) -> *mut core::ffi::c_void { hDF as *mut core::ffi::c_void }

use crate::win32::{PathFindFileNameW, SHELL_ErrorBox};
use crate::shell32::shlfileop::SHELL_ConfirmYesNoW;
use crate::shell32::sdk_consts::{CMIC_MASK_FLAG_NO_UI, SHCOLSTATE_TYPE_DATE, SHCOLSTATE_TYPE_INT};
use crate::shell32::sdk_consts::SHCOLSTATE_TYPE_STR;

// Missing from `sdk_consts`, as in every other flipped folder module in this crate
// (drives_folder.rs, cfs_folder.rs, control_panel_folder.rs, ...) — there is nowhere to
// import it from.
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // shtypes.idl:125

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct columninfo {
    pub column_name_id: i32,
    pub fmtId: *const GUID,
    pub pid: DWORD,
    pub pcsFlags: i32,
    pub fmt: i32,
    pub cxChars: i32,
}

// `columninfo` holds a `*const GUID` into static data. Same shape, same reason, as
// recycle_bin.rs:1369.
unsafe impl Sync for columninfo {}
pub static RecycleBinColumns: [columninfo; 6] = [
    columninfo {
        column_name_id: IDS_SHV_COLUMN_NAME as i32,
        fmtId: core::ptr::addr_of!(FMTID_Storage).cast(),
        pid: PID_STG_NAME,
        pcsFlags: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_LEFT as i32,
        cxChars: 25,
    },
    columninfo {
        column_name_id: IDS_SHV_COLUMN_DELFROM as i32,
        fmtId: core::ptr::addr_of!(FMTID_Displaced).cast(),
        pid: PID_DISPLACED_FROM,
        pcsFlags: (SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_LEFT as i32,
        cxChars: 35,
    },
    columninfo {
        column_name_id: IDS_SHV_COLUMN_DELDATE as i32,
        fmtId: core::ptr::addr_of!(FMTID_Displaced).cast(),
        pid: PID_DISPLACED_DATE,
        pcsFlags: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_LEFT as i32,
        cxChars: 15,
    },
    columninfo {
        column_name_id: IDS_SHV_COLUMN_SIZE as i32,
        fmtId: core::ptr::addr_of!(FMTID_Storage).cast(),
        pid: PID_STG_SIZE,
        pcsFlags: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_RIGHT as i32,
        cxChars: 10,
    },
    columninfo {
        column_name_id: IDS_SHV_COLUMN_TYPE as i32,
        fmtId: core::ptr::addr_of!(FMTID_Storage).cast(),
        pid: PID_STG_STORAGETYPE,
        pcsFlags: (SHCOLSTATE_TYPE_INT | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_LEFT as i32,
        cxChars: 15,
    },
    columninfo {
        column_name_id: IDS_SHV_COLUMN_MODIFIED as i32,
        fmtId: core::ptr::addr_of!(FMTID_Storage).cast(),
        pid: PID_STG_WRITETIME,
        pcsFlags: (SHCOLSTATE_TYPE_DATE | SHCOLSTATE_ONBYDEFAULT) as i32,
        fmt: LVCFMT_LEFT as i32,
        cxChars: 15,
    },
];

pub const COLUMN_NAME: u32 = 0;
pub const COLUMN_DELFROM: u32 = 1;
pub const COLUMN_DATEDEL: u32 = 2;
pub const COLUMN_SIZE: u32 = 3;
pub const COLUMN_TYPE: u32 = 4;
pub const COLUMN_MTIME: u32 = 5;
pub const COLUMNS_COUNT: u32 = 6;

pub const BBITEMFILETYPE: u32 = PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FILE_FLAG;
pub const BBITEMFOLDERTYPE: u32 = PT_FS | PT_FS_UNICODE_FLAG | PT_FS_FOLDER_FLAG;

#[repr(C)]
pub struct BBITEMDATA {
    pub DeletionTime: FILETIME,
    pub RecycledPathOffset: WORD,
    pub OriginalLocation: [WCHAR; 1],
}

#[repr(C)]
pub struct BBITEMFOOTER {
    pub DataSize: WORD,
    pub EndSignature: WORD,
}

impl BBITEMFOOTER {
    /// C: `enum { ENDSIG = MAKEWORD('K', 'I') }` — an in-struct enum constant, which has no
    /// Rust field to land in. "Killed item". The low bit MUST be set so `_ILGetFileStructW`
    /// returns NULL for these PIDLs; 'K' is 0x4B, so it is.
    pub const ENDSIG: WORD = ((b'I' as WORD) << 8) | (b'K' as WORD);
}

pub const IDC_BB_RESTORE: u32 = 1;
pub const IDC_BB_CUT: u32 = 2;
pub const IDC_BB_DELETE: u32 = 3;
pub const IDC_BB_PROPERTIES: u32 = 4;

pub static mut g_BBItemVerbMap: [CMVERBMAP; 5] = [
    CMVERBMAP {
        Verb: b"undelete\0".as_ptr() as LPCSTR,
        CmdId: IDC_BB_RESTORE as u16,
    },
    CMVERBMAP {
        Verb: b"cut\0".as_ptr() as LPCSTR,
        CmdId: IDC_BB_CUT as u16,
    },
    CMVERBMAP {
        Verb: b"delete\0".as_ptr() as LPCSTR,
        CmdId: IDC_BB_DELETE as u16,
    },
    CMVERBMAP {
        Verb: b"properties\0".as_ptr() as LPCSTR,
        CmdId: IDC_BB_PROPERTIES as u16,
    },
    CMVERBMAP {
        Verb: core::ptr::null(),
        CmdId: 0,
    },
];

pub const IDC_EMPTYRECYCLEBIN: u32 = 1;

pub static mut g_BBFolderVerbMap: [CMVERBMAP; 3] = [
    CMVERBMAP {
        Verb: b"empty\0".as_ptr() as LPCSTR,
        CmdId: IDC_EMPTYRECYCLEBIN as u16,
    },
    CMVERBMAP {
        Verb: b"properties\0".as_ptr() as LPCSTR,
        CmdId: IDC_PROPERTIES as u16,
    },
    CMVERBMAP {
        Verb: core::ptr::null(),
        CmdId: 0,
    },
];

#[repr(C)]
pub struct FILEOPDATA {
    pub apidl: PCUITEMID_CHILD_ARRAY,
    pub cidl: UINT,
}
/// C writes `FIELD_OFFSET(PIDLDATA, u.file.szNames)`. Our `FIELD_OFFSET!` takes a single ident,
/// and widening it would not help: bindgen renders `PIDLDATA::u` as a struct of
/// `__BindgenUnionField<T>` wrappers, which are zero-sized, so `offset_of!(PIDLDATA, u.file.…)`
/// has no `szNames` to reach. Every arm of a union starts at the same address, so the offset is
/// the union's own offset plus the field's offset inside the arm.
#[inline]
pub fn PIDLDATA_FILE_SZNAMES_OFFSET() -> usize {
    core::mem::offset_of!(PIDLDATA, u) + core::mem::offset_of!(FileStruct, szNames)
}

unsafe extern "C" fn IsFolder(pidl: LPCITEMIDLIST) -> u8 {
    if (_ILGetFSType(pidl) as u32) & PT_FS_FOLDER_FLAG != 0 { 1u8 } else { 0u8 }
}
unsafe extern "C" fn ValidateItem(pidl: LPCITEMIDLIST) -> *mut BBITEMDATA {
    // C: `sizeof(L"X") + sizeof("")` == 4 + 1. Sealing turns each literal into a named static,
    // and `sizeof` of a literal is the ARRAY's byte size — so the lowering is
    // `size_of_val(&ARRAY)`. Adding `.as_ptr()` measures a POINTER instead (4 or 8 by arch),
    // which compiles and is silently wrong; `size_of_val(&"")` is likewise a `&str` (16), not 1.
    // minsize gates PIDL validation, so getting it wrong either accepts malformed items and
    // reads out of bounds, or rejects valid ones and shows an empty recycle bin.
    let minstringsize = core::mem::size_of_val(&CPP_COM_WSTR_9855CDA098DC) + 1;
    let minfs = core::mem::size_of::<WORD>() + PIDLDATA_FILE_SZNAMES_OFFSET() + minstringsize;
    // `sizeof(L"C:\\X") * 2` == 10 * 2; same array-not-pointer rule as above.
    let mindatasize = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) as usize + (core::mem::size_of_val(&CPP_COM_WSTR_F51AA22D711D) * 2);
    let minsize = minfs + mindatasize + core::mem::size_of::<BBITEMFOOTER>();
    let type_ = _ILGetType(pidl);
    if (type_ == BBITEMFILETYPE as u8 || type_ == BBITEMFOLDERTYPE as u8) && (*pidl).mkid.cb >= minsize as u16 {
        let pEnd = (pidl as *const u8).add((*pidl).mkid.cb as usize - core::mem::size_of::<BBITEMFOOTER>()) as *mut BBITEMFOOTER;
        if (*pEnd).EndSignature == BBITEMFOOTER::ENDSIG && (*pEnd).DataSize >= mindatasize as WORD {
            return (pEnd as *mut u8).sub((*pEnd).DataSize as usize) as *mut BBITEMDATA;
        }
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn CreateItem(pszTrash: *const u16, pszOrig: *const u16, Details: *const DELETED_FILE_INFO) -> LPITEMIDLIST {
    let folder = ((*Details).Attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
    let pszName = PathFindFileNameW(pszTrash);
    let ofsName = pszName.offset_from(pszTrash) as usize;
    let cchName = wcslen(pszName) + 1;
    let cbName = cchName * core::mem::size_of::<u16>();
    let cbFSNames = cbName + core::mem::size_of::<[u8; 1]>() + 1;
    let cbFS = core::mem::size_of::<u16>() + PIDLDATA_FILE_SZNAMES_OFFSET() + cbFSNames;
    let cchTrash = ofsName + cchName;
    let cbTrash = cchTrash * core::mem::size_of::<u16>();
    let cchOrig = wcslen(pszOrig) + 1;
    let cbOrig = cchOrig * core::mem::size_of::<u16>();
    let cbData = FIELD_OFFSET!(BBITEMDATA, OriginalLocation) as usize + cbOrig + cbTrash;
    let cb = cbFS + cbData + core::mem::size_of::<BBITEMFOOTER>();
    if cb > 0xffff {
        return core::ptr::null_mut();
    }
    let pidl = SHAlloc((cb + core::mem::size_of::<WORD>()) as SIZE_T) as LPITEMIDLIST;
    if pidl.is_null() {
        return pidl;
    }

    (*pidl).mkid.cb = cb as u16;
    (*pidl).mkid.abID[0] = if folder { BBITEMFOLDERTYPE as u8 } else { BBITEMFILETYPE as u8 };
    (*ILGetNext(pidl)).mkid.cb = 0;

    // bindgen renders PIDLDATA's union as a struct of `__BindgenUnionField<T>` wrappers, so the
    // arm has to be selected once, here, rather than at every field use.
    let fsitem = (*(((*pidl).mkid.abID).as_mut_ptr() as *mut PIDLDATA)).u.file.as_mut() as *mut FileStruct;
    (*fsitem).dummy = 0;
    const _: () = assert!(core::mem::size_of::<RECYCLEBINFILESIZETYPE>() <= core::mem::size_of::<DWORD>());
    (*fsitem).dwFileSize = (*Details).FileSize;
    (*fsitem).uFileAttribs = LOWORD((*Details).Attributes);
    FileTimeToDosDateTime(core::ptr::addr_of!((*Details).LastModification), core::ptr::addr_of_mut!((*fsitem).uFileDate), core::ptr::addr_of_mut!((*fsitem).uFileTime));
    CopyMemory((*fsitem).szNames.as_mut_ptr() as *mut core::ffi::c_void, pszName as *const core::ffi::c_void, cbName);
    let pszShort = ((*fsitem).szNames.as_mut_ptr() as *mut u8).add(cbName) as *mut i8;
    *pszShort.add(0) = b'\0' as i8;
    *pszShort.add(1) = b'\0' as i8;

    let footer = (pidl as *mut u8).add(cb).sub(core::mem::size_of::<BBITEMFOOTER>()) as *mut BBITEMFOOTER;
    (*footer).DataSize = cbData as u16;
    (*footer).EndSignature = BBITEMFOOTER::ENDSIG;

    let data = (footer as *mut u8).sub((*footer).DataSize as usize) as *mut BBITEMDATA;
    (*data).DeletionTime = (*Details).DeletionTime;
    (*data).RecycledPathOffset = (FIELD_OFFSET!(BBITEMDATA, OriginalLocation) + cbOrig as isize) as u16;
    CopyMemory((*data).OriginalLocation.as_mut_ptr() as *mut core::ffi::c_void, pszOrig as *const core::ffi::c_void, cbOrig);
    CopyMemory((data as *mut u8).add((*data).RecycledPathOffset as usize) as *mut core::ffi::c_void, pszTrash as *const core::ffi::c_void, cbTrash);

    assert!(((core::ptr::addr_of!((*fsitem).szNames) as usize) & 1) == 0);
    const _: () = assert!(core::mem::offset_of!(BBITEMDATA, OriginalLocation) % 2 == 0);
    assert!(((data as usize) & 1) == 0);
    assert!(_ILGetFSType(pidl) != 0);
    assert!(_ILIsPidlSimple(pidl) != 0);
    assert!((*(((pidl as *mut u8).add((*pidl).mkid.cb as usize).sub(core::mem::size_of::<WORD>())) as *mut u16) & 1) != 0);
    assert!(_ILGetFileStructW(pidl).is_null());
    assert!(ValidateItem(pidl) == data);
    pidl
}
unsafe extern "C" fn GetItemFileSize(pidl: LPCITEMIDLIST) -> UINT {
    if _ILGetFSType(pidl) != 0 {
        (*(pidl as *const PIDLDATA)).u.file.as_ref().dwFileSize
    } else {
        0
    }
}
unsafe extern "C" fn GetItemOriginalFullPath(Data: *const BBITEMDATA) -> LPCWSTR {
    core::ptr::addr_of!((*Data).OriginalLocation).cast()
}
unsafe extern "C" fn GetItemOriginalFolder(Data: *const BBITEMDATA, Out: *mut LPWSTR) -> HRESULT {
    let hr = SHStrDupW(GetItemOriginalFullPath(Data), Out);
    if ((hr) >= 0) {
        PathRemoveFileSpecW(*Out);
    }
    hr
}
unsafe extern "C" fn GetItemOriginalFileName(Data: *const BBITEMDATA) -> LPCWSTR {
    PathFindFileNameW(GetItemOriginalFullPath(Data))
}
unsafe extern "C" fn GetItemRecycledFullPath(Data: *const BBITEMDATA) -> LPCWSTR {
    ((Data as *const u8).add((*Data).RecycledPathOffset as usize)) as LPCWSTR
}
unsafe extern "C" fn GetItemDriveNumber(pidl: LPCITEMIDLIST) -> i32 {
    let pData = ValidateItem(pidl);
    if !pData.is_null() {
        return PathGetDriveNumberW(GetItemRecycledFullPath(pData));
    }
    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if _ILSimpleGetTextW(pidl, buf.as_mut_ptr(), _countof!(buf)) != 0 {
        PathGetDriveNumberW(buf.as_ptr())
    } else {
        -1
    }
}
unsafe extern "C" fn GetItemTypeName(pidl: PCUITEMID_CHILD, Data: *const BBITEMDATA, shfi: *mut SHFILEINFOW) -> HRESULT {
    let path = GetItemRecycledFullPath(Data);
    let attribs = (*((*pidl).mkid.abID.as_ptr() as *const PIDLDATA)).u.file.as_ref().uFileAttribs;
    if SHGetFileInfoW(path, attribs as u32, shfi, core::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES) != 0 {
        return S_OK;
    }
    (*shfi).szTypeName[0] = UNICODE_NULL;
    E_FAIL
}
unsafe extern "C" fn GetDefaultRecycleDriveNumber() -> UINT {
    let mut drive: c_int = 0;
    let mut buf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];   // C: `WCHAR buf[MAX_PATH]`
    if GetWindowsDirectoryW(buf.as_mut_ptr(), _countof!(buf) as UINT) != 0 {
        drive = PathGetDriveNumberW(buf.as_ptr());
    }
    core::cmp::max(0, drive) as UINT
}

unsafe extern "C" fn GetGlobalRecycleBinPath() -> LPCWSTR {
    (NULL).cast()
}
pub unsafe extern "C" fn IsRecycleBinEmpty(pSF: *mut IShellFolder) -> u8 {
    let mut spEnumFiles: *mut IEnumIDList = core::ptr::null_mut();
    let hr: i32 = ((*(*pSF).lpVtbl).EnumObjects)(
        pSF,
        core::ptr::null_mut(),
        SHCONTF_FOLDERS | SHCONTF_NONFOLDERS,
        &mut spEnumFiles,
    );
    let mut spPidl: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut itemcount: u32 = 0;
    let result = if (hr) < 0 {
        1u8
    } else {
        let next_hr = ((*(*spEnumFiles).lpVtbl).Next)(spEnumFiles, 1, core::ptr::addr_of_mut!(spPidl).cast(), &mut itemcount);
        if next_hr != S_OK {
            1u8
        } else {
            0u8
        }
    };
    if !spPidl.is_null() {
        CoTaskMemFree(spPidl as *mut core::ffi::c_void);
    }
    if !spEnumFiles.is_null() {
        ((*(*spEnumFiles).lpVtbl).Release)(spEnumFiles);
    }
    result
}
unsafe extern "C" fn CRecycleBin_ChangeNotifyBBItem(Event: LONG, BBItem: LPCITEMIDLIST) {
    let pidlFolder = SHCloneSpecialIDList(core::ptr::null_mut(), CSIDL_BITBUCKET as i32, FALSE);
    if pidlFolder.is_null() {
        return;
    }
    if !BBItem.is_null() {
        assert!(!ValidateItem(BBItem).is_null());
        let pidlFull = ILCombine(pidlFolder, BBItem);
        if !pidlFull.is_null() {
            SHChangeNotify(Event, SHCNF_IDLIST, (pidlFull).cast(), core::ptr::null_mut());
            ILFree(pidlFull);
        }
    } else {
        SHChangeNotify(Event, SHCNF_IDLIST, (pidlFolder).cast(), core::ptr::null_mut());
    }
    ILFree(pidlFolder);
}
#[no_mangle]
pub unsafe extern "C" fn CRecycleBin_NotifyRecycled(
    OrigPath: *const u16,
    pFind: *const WIN32_FIND_DATAW,
    pFI: *const RECYCLEBINFILEIDENTITY,
) {
    let mut info: DELETED_FILE_INFO = core::mem::zeroed();
    info.LastModification = (*pFind).ftLastWriteTime;
    info.DeletionTime = (*pFI).DeletionTime;
    info.FileSize = (*pFind).nFileSizeLow;
    info.Attributes = (*pFind).dwFileAttributes;
    let pidl: LPITEMIDLIST = CreateItem((*pFI).RecycledFullPath, OrigPath, &info as *const _);
    if !pidl.is_null() {
        CRecycleBin_ChangeNotifyBBItem(
            if IsFolder(pidl) != 0 {
                SHCNE_MKDIR as i32
            } else {
                SHCNE_CREATE as i32
            },
            pidl,
        );
        ILFree(pidl);
    }
}
pub unsafe extern "C" fn CRecycleBin_NotifyRemovedFromRecycleBin(BBItem: LPCITEMIDLIST) {
    CRecycleBin_ChangeNotifyBBItem(if IsFolder(BBItem) != 0 { SHCNE_RMDIR as i32 } else { SHCNE_DELETE as i32 }, BBItem);
}
unsafe extern "C" fn ConfirmDelete(lpcmi: LPCMINVOKECOMMANDINFO, cidl: UINT, pidl: LPCITEMIDLIST) -> BOOL {
    let mut pData: *mut BBITEMDATA;
    if (*lpcmi).fMask & CMIC_MASK_FLAG_NO_UI != 0 {
        return TRUE;
    } else {
        pData = ValidateItem(pidl);
        if cidl == 1 && pData != core::ptr::null_mut() {
            let ask: UINT = if IsFolder(pidl) != 0 {
                ASK_DELETE_FOLDER
            } else {
                ASK_DELETE_FILE
            };
            return SHELL_ConfirmYesNoW((*lpcmi).hwnd, ask as i32, GetItemOriginalFileName(pData));
        }
    }
    let mut buf: [WCHAR; 42] = [0; 42];
    wsprintfW(buf.as_mut_ptr(), CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), cidl);
    return SHELL_ConfirmYesNoW((*lpcmi).hwnd, ASK_DELETE_MULTIPLE_ITEM as i32, buf.as_ptr());
}
unsafe extern "C" fn CreateFileOpStrings(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, RecycledPath: BOOL) -> LPWSTR {
    let mut mem: PWSTR = core::ptr::null_mut();
    let mut newmem: PWSTR;
    let mut i: SIZE_T = 0;
    let mut cb: SIZE_T = 0;
    let mut cb2: SIZE_T;
    let mut cbPath: SIZE_T;

    'outer: loop {
        if i >= cidl as SIZE_T {
            break;
        }

        'fail: {
            let pData: *mut BBITEMDATA = ValidateItem(*apidl.offset(i as isize));
            if pData.is_null() {
                LocalFree(mem as *mut c_void);
                return core::ptr::null_mut();
            }

            let path: LPCWSTR = if RecycledPath != 0 {
                GetItemRecycledFullPath(pData)
            } else {
                GetItemOriginalFullPath(pData)
            };

            cbPath = (lstrlenW(path) as SIZE_T + 1) * core::mem::size_of::<WCHAR>() as SIZE_T;
            cb2 = cb + cbPath;
            let cbTot: SIZE_T = cb2 as SIZE_T + core::mem::size_of::<WCHAR>() as SIZE_T;

            newmem = if i != 0 {
                LocalReAlloc(mem as *mut c_void, cbTot, LMEM_MOVEABLE) as PWSTR
            } else {
                LocalAlloc(LPTR, cbTot) as PWSTR
            };

            if newmem.is_null() {
                LocalFree(mem as *mut c_void);
                return core::ptr::null_mut();
            }

            mem = newmem;
            CopyMemory((mem as *mut u8).offset(cb as isize) as *mut c_void, path as *const c_void, cbPath as usize);
            *((mem as *mut u8).offset((cb + cbPath) as isize) as PWSTR) = UNICODE_NULL;

            break 'fail;
        }

        cb = cb2;
        i += 1;
    }

    mem
}
unsafe extern "C" fn CompareCanonical(Data1: *const BBITEMDATA, Data2: *const BBITEMDATA) -> HRESULT {
    let mut result = CompareFileTime(
        core::ptr::addr_of!((*Data1).DeletionTime),
        core::ptr::addr_of!((*Data2).DeletionTime)
    );
    if result == 0 {
        result = _wcsicmp(
            GetItemOriginalFullPath(Data1),
            GetItemOriginalFullPath(Data2)
        );
    }
    MAKE_COMPARE_HRESULT!(result)
}
#[no_mangle]
pub unsafe extern "C" fn TRASH_CanTrashFile(wszPath: PWSTR) -> BOOL {
    let mut ret: LONG;
    let mut dwNukeOnDelete: DWORD = core::mem::zeroed();
    let mut dwType: DWORD = core::mem::zeroed();
    let mut VolSerialNumber: DWORD = core::mem::zeroed();
    let mut MaxComponentLength: DWORD = core::mem::zeroed();
    let mut FileSystemFlags: DWORD = core::mem::zeroed();
    let mut dwSize: DWORD;
    let mut dwDisposition: DWORD = core::mem::zeroed();
    let mut hKey: HKEY = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 10] = [0; 10];
    let mut szKey: [WCHAR; 150] = [0; 150];

    core::ptr::copy_nonoverlapping(
        CPP_COM_WSTR_BCF17C907FBF.as_ptr(),
        szKey.as_mut_ptr(),
        CPP_COM_WSTR_BCF17C907FBF.len()
    );

    if *wszPath.offset(1) != 0x003Au16 {
        return FALSE;
    }

    let mut wszRootPathName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    StringCbCopyW(wszRootPathName.as_mut_ptr(), core::mem::size_of_val(&wszRootPathName) as SIZE_T, wszPath);
    PathStripToRootW(wszRootPathName.as_mut_ptr());

    if GetDriveTypeW(wszRootPathName.as_ptr()) != DRIVE_FIXED {
        return FALSE;
    }

    if GetVolumeInformationW(
        wszRootPathName.as_ptr(),
        core::ptr::null_mut(),
        0,
        &mut VolSerialNumber,
        &mut MaxComponentLength,
        &mut FileSystemFlags,
        core::ptr::null_mut(),
        0
    ) == 0
    {
        return FALSE;
    }

    swprintf(
        szBuffer.as_mut_ptr(),
        CPP_COM_WSTR_9618E75FEC6D.as_ptr(),
        LOWORD(VolSerialNumber) as UINT,
        HIWORD(VolSerialNumber) as UINT
    );
    wcscat(szKey.as_mut_ptr(), szBuffer.as_ptr());

    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        szKey.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        &mut dwDisposition
    ) != ERROR_SUCCESS as i32
    {
        return FALSE;
    }

    if (dwDisposition & REG_CREATED_NEW_KEY) != 0 {
        dwNukeOnDelete = 0;
        RegSetValueExW(
            hKey,
            CPP_COM_WSTR_604114ADC8B4.as_ptr(),
            0,
            REG_DWORD,
            (&dwNukeOnDelete as *const DWORD) as LPBYTE,
            core::mem::size_of::<DWORD>() as DWORD
        );
        dwSize = (1 as DWORD).wrapping_neg();
        RegSetValueExW(
            hKey,
            CPP_COM_WSTR_C7A5956868FE.as_ptr(),
            0,
            REG_DWORD,
            (&dwSize as *const DWORD) as LPBYTE,
            core::mem::size_of::<DWORD>() as DWORD
        );
    } else {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        ret = RegQueryValueExW(
            hKey,
            CPP_COM_WSTR_604114ADC8B4.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            (&mut dwNukeOnDelete as *mut DWORD) as LPBYTE,
            &mut dwSize
        );
        if ret != ERROR_SUCCESS as i32 {
            dwNukeOnDelete = 0;
            if ret == ERROR_FILE_NOT_FOUND as i32 {
                RegSetValueExW(
                    hKey,
                    CPP_COM_WSTR_604114ADC8B4.as_ptr(),
                    0,
                    REG_DWORD,
                    (&dwNukeOnDelete as *const DWORD) as LPBYTE,
                    core::mem::size_of::<DWORD>() as DWORD
                );
            }
        }
    }
    let bCanTrash: BOOL = if dwNukeOnDelete != 0 { 0 } else { 1 };
    RegCloseKey(hKey);
    return bCanTrash;
}

#[no_mangle]
pub unsafe extern "C" fn TRASH_TrashFile(wszPath: LPCWSTR) -> BOOL {
    // TRACE("(%s)\n", debugstr_w(wszPath));
    DeleteFileToRecycleBinW(wszPath)
}
unsafe extern "C" fn TRASH_PlayEmptyRecycleBinSound() {
    let mut regKey: HKEY = core::ptr::null_mut();
    let mut pszValue = CHeapPtr::<u16>::new();
    let mut pszSndPath = CHeapPtr::<u16>::new();
    let mut dwType: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut lError: i32;

    lError = RegOpenKeyExW(HKEY_CURRENT_USER, CPP_COM_WSTR_49A455CD42F8.as_ptr(), 0, KEY_READ,
                           core::ptr::addr_of_mut!(regKey));
    if lError != ERROR_SUCCESS as i32 {
        return;   // nothing opened yet
    }

    lError = RegQueryValueExW(regKey, core::ptr::null(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize));
    if lError != ERROR_SUCCESS as i32 {
        { RegCloseKey(regKey); return; }
    }

    if pszValue.Allocate(dwSize as usize / core::mem::size_of::<WCHAR>()) == 0 {
        { RegCloseKey(regKey); return; }
    }

    lError = RegQueryValueExW(regKey, core::ptr::null(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), pszValue.m_pData as LPBYTE, core::ptr::addr_of_mut!(dwSize));
    if lError != ERROR_SUCCESS as i32 {
        { RegCloseKey(regKey); return; }
    }

    if dwType == REG_EXPAND_SZ {
        dwSize = ExpandEnvironmentStringsW(pszValue.m_pData, core::ptr::null_mut(), 0);
        if dwSize == 0 {
            { RegCloseKey(regKey); return; }
        }

        if pszSndPath.Allocate(dwSize as usize) == 0 {
            { RegCloseKey(regKey); return; }
        }

        if ExpandEnvironmentStringsW(pszValue.m_pData, pszSndPath.m_pData, dwSize) == 0 {
            { RegCloseKey(regKey); return; }
        }
    } else if dwType == REG_SZ {
        pszSndPath.Attach(pszValue.Detach());
    } else {
        { RegCloseKey(regKey); return; }
    }

    PlaySoundW(pszSndPath.m_pData, core::ptr::null_mut(), SND_FILENAME | SND_ASYNC | SND_NODEFAULT);
    RegCloseKey(regKey);
}

#[no_mangle]
pub unsafe extern "system" fn SHUpdateRecycleBinIcon() -> HRESULT {
    let dwItem2: DWORD = (1 as DWORD).wrapping_neg();
    SHChangeNotify(SHCNE_UPDATEIMAGE as i32, SHCNF_DWORD, core::ptr::null_mut(), core::ptr::addr_of!(dwItem2) as *const c_void);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SHEmptyRecycleBinA(hwnd: HWND, pszRootPath: LPCSTR, dwFlags: DWORD) -> HRESULT {
    let mut szRootPathW: LPWSTR = core::ptr::null_mut();
    let mut len: i32;
    let hr: HRESULT;

    if !pszRootPath.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pszRootPath, -1, core::ptr::null_mut(), 0);
        if len == 0 {
            return HRESULT_FROM_WIN32(GetLastError() as i32);
        }
        szRootPathW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if szRootPathW.is_null() {
            return E_OUTOFMEMORY;
        }
        if MultiByteToWideChar(CP_ACP, 0, pszRootPath, -1, szRootPathW, len) == 0 {
            HeapFree(GetProcessHeap(), 0, szRootPathW as *mut _);
            return HRESULT_FROM_WIN32(GetLastError() as i32);
        }
    }

    hr = SHEmptyRecycleBinW(hwnd, szRootPathW, dwFlags);
    HeapFree(GetProcessHeap(), 0, szRootPathW as *mut _);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHEmptyRecycleBinW(hwnd: HWND, mut pszRootPath: LPCWSTR, dwFlags: DWORD) -> HRESULT {
    let mut szBuffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut count: DWORD;
    let mut ret: LONG;
    let mut pDesktop: *mut IShellFolder = core::ptr::null_mut();
    let mut pRecycleBin: *mut IShellFolder = core::ptr::null_mut();
    let mut pidlRecycleBin: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut pidl: PITEMID_CHILD = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;
    let mut penumFiles: LPENUMIDLIST = core::ptr::null_mut();
    let mut StrRet: STRRET = core::mem::zeroed();

    if !pszRootPath.is_null() && *pszRootPath == 0 {
        // "If this value is an empty string or NULL, all Recycle Bins on all drives will be emptied."
        pszRootPath = core::ptr::null();
    }

    if (dwFlags & SHERB_NOCONFIRMATION) == 0 {
        hr = SHGetDesktopFolder(&mut pDesktop);
        if ((hr) < 0) {
            return hr;
        }
        hr = SHGetFolderLocation(core::ptr::null_mut(), CSIDL_BITBUCKET as i32, core::ptr::null_mut(), 0, &mut pidlRecycleBin);
        if ((hr) < 0) {
            ((*(*pDesktop).lpVtbl).Release)(pDesktop);
            return hr;
        }
        hr = ((*(*pDesktop).lpVtbl).BindToObject)(pDesktop, pidlRecycleBin.cast(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(pRecycleBin) as *mut *mut core::ffi::c_void);
        CoTaskMemFree((pidlRecycleBin).cast());
        ((*(*pDesktop).lpVtbl).Release)(pDesktop);
        if ((hr) < 0) {
            return hr;
        }
        hr = ((*(*pRecycleBin).lpVtbl).EnumObjects)(pRecycleBin, (hwnd).cast(), SHCONTF_FOLDERS | SHCONTF_NONFOLDERS | SHCONTF_INCLUDEHIDDEN, &mut penumFiles);
        if ((hr) < 0) {
            ((*(*pRecycleBin).lpVtbl).Release)(pRecycleBin);
            return hr;
        }

        count = 0;
        if hr != S_FALSE {
            while ((*(*penumFiles).lpVtbl).Next)(penumFiles, 1, core::ptr::addr_of_mut!(pidl).cast(), core::ptr::null_mut()) == S_OK {
                count = count.wrapping_add(1);
                ((*(*pRecycleBin).lpVtbl).GetDisplayNameOf)(pRecycleBin, pidl.cast(), (SHGDN_NORMAL | SHGDN_INFOLDER) as SHGDNF, core::ptr::addr_of_mut!(StrRet).cast());
                StrRetToBufW(core::ptr::addr_of_mut!(StrRet).cast(), pidl.cast(), szBuffer.as_mut_ptr(), _countof!(szBuffer) as UINT);
                CoTaskMemFree((pidl).cast());
            }
            ((*(*penumFiles).lpVtbl).Release)(penumFiles);
        }
        ((*(*pRecycleBin).lpVtbl).Release)(pRecycleBin);

        match count {
            0 => {
            }
            1 => {
                if ShellMessageBoxWrapW(shell32_hInstance, hwnd, MAKEINTRESOURCEW(IDS_DELETEITEM_TEXT), MAKEINTRESOURCEW(IDS_EMPTY_BITBUCKET),
                                   MB_ICONEXCLAMATION | MB_YESNO | MB_DEFBUTTON2, szBuffer.as_ptr()) == IDNO as i32 {
                    return S_OK;
                }
            }
            _ => {
                strsafe_rust::StringCchPrintfW!(szBuffer.as_mut_ptr(), _countof!(szBuffer) as usize, CPP_COM_WSTR_69A283E639E4.as_ptr(), count);
                if ShellMessageBoxWrapW(shell32_hInstance, hwnd, MAKEINTRESOURCEW(IDS_DELETEMULTIPLE_TEXT), MAKEINTRESOURCEW(IDS_EMPTY_BITBUCKET),
                                   MB_ICONEXCLAMATION | MB_YESNO | MB_DEFBUTTON2, szBuffer.as_ptr()) == IDNO as i32 {
                    return S_OK;
                }
            }
        }
    }

    if (dwFlags & SHERB_NOPROGRESSUI) != 0 {
        ret = EmptyRecycleBinW(pszRootPath);
    } else {
        ret = EmptyRecycleBinW(pszRootPath);
    }

    if ret == 0 {
        return HRESULT_FROM_WIN32(GetLastError() as i32);
    }

    CRecycleBin_ChangeNotifyBBItem(SHCNE_UPDATEDIR as i32, core::ptr::null_mut());
    if (dwFlags & SHERB_NOSOUND) == 0 {
        TRASH_PlayEmptyRecycleBinSound();
    }
    S_OK
}

// ── Bridges published for the retained C++ ────────────────────────────────────────────────
//
// Both companion classes in this TU are Rust now, so these two are ordinary calls into their
// modules. rustconv translated the C++ `ShellObjectCreatorInit<T>` literally; that is an ATL
// template with no Rust spelling, and the `--cpp-com --internal` slices exist precisely
// because it has none — each publishes its own non-aggregating factory instead.
//
// The other three bridges that used to live here (CRecyclerDropTarget_CreateInstance,
// CRecycleBinFolderViewCB_CreateInstance, CRecycleBinFolderViewCB_Initialize) did NOT come
// across: their far side is still C++. They moved to rust_recyclebin_bridges_stub.cpp, which
// documents why each one had to stay.

#[no_mangle]
pub unsafe extern "system" fn CRecycleBinEnum_CreateInstance_RustBridge(
    dwFlags: DWORD,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    crate::recycle_bin_enum::CRecycleBinEnum_CreateInstance(dwFlags, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CRecycleBinItemContextMenu_CreateInstance_RustBridge(
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    crate::recycle_bin_item_context_menu::CRecycleBinItemContextMenu_CreateInstance(
        cidl, apidl, riid, ppv,
    )
}

// ── SHQueryRecycleBin{A,W} — hand-written (shell32.spec:333-334) ──────────────────────────
//
// rustconv could not buy these: they walk IRecycleBin -> IRecycleBinEnumList -> IRecycleBinFile,
// and bindgen renders all three as opaque `__bindgen_vtable(c_void)` in the island, so there is
// no callable slot to translate a method call INTO. The three vtables below are transcribed by
// hand from shellrecyclebin/recyclebin.h's DECLARE_INTERFACE_ blocks, which fix slot order
// exactly; only the slots these two functions use are named, the rest are reserved so the
// indices stay right. `--skip-fn` recorded the omission as an audit finding rather than letting
// it go quiet.

#[repr(C)]
struct IRecycleBinFileVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IRecycleBinFile, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IRecycleBinFile) -> ULONG,
    Release: unsafe extern "system" fn(*mut IRecycleBinFile) -> ULONG,
    IsEqualIdentity: unsafe extern "system" fn(*mut IRecycleBinFile, *const c_void) -> HRESULT,
    GetInfo: unsafe extern "system" fn(*mut IRecycleBinFile, *mut c_void) -> HRESULT,
    GetLastModificationTime: unsafe extern "system" fn(*mut IRecycleBinFile, *mut FILETIME) -> HRESULT,
    GetDeletionTime: unsafe extern "system" fn(*mut IRecycleBinFile, *mut FILETIME) -> HRESULT,
    GetFileSize: unsafe extern "system" fn(*mut IRecycleBinFile, *mut ULARGE_INTEGER) -> HRESULT,
    // GetPhysicalFileSize, GetAttributes, GetFileName, Delete, Restore, RemoveFromDatabase
    // follow; unused here, so they are not named.
}
#[repr(C)]
struct IRecycleBinFile {
    lpVtbl: *const IRecycleBinFileVtbl,
}

#[repr(C)]
struct IRecycleBinEnumListVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IRecycleBinEnumList, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IRecycleBinEnumList) -> ULONG,
    Release: unsafe extern "system" fn(*mut IRecycleBinEnumList) -> ULONG,
    Next: unsafe extern "system" fn(*mut IRecycleBinEnumList, DWORD, *mut *mut IRecycleBinFile, *mut DWORD) -> HRESULT,
    // Skip, Reset follow; unused here.
}
#[repr(C)]
struct IRecycleBinEnumList {
    lpVtbl: *const IRecycleBinEnumListVtbl,
}

#[repr(C)]
struct IRecycleBinVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IRecycleBin, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IRecycleBin) -> ULONG,
    Release: unsafe extern "system" fn(*mut IRecycleBin) -> ULONG,
    DeleteFile: unsafe extern "system" fn(*mut IRecycleBin, LPCWSTR) -> HRESULT,
    EmptyRecycleBin: unsafe extern "system" fn(*mut IRecycleBin) -> HRESULT,
    EnumObjects: unsafe extern "system" fn(*mut IRecycleBin, *mut *mut IRecycleBinEnumList) -> HRESULT,
    // GetDirectory follows; unused here.
}
#[repr(C)]
struct IRecycleBin {
    lpVtbl: *const IRecycleBinVtbl,
}

unsafe extern "C" {
    // shellrecyclebin/recyclebin_private.h — plain C, already inside an `extern "C"` block.
    fn GetDefaultRecycleBin(pszVolume: LPCWSTR, pprb: *mut *mut IRecycleBin) -> HRESULT;
}

/// `SHQUERYRBINFO` (shellapi.h). Not in the island: shell32's include closure reaches the
/// declaration of these two exports but never a use of the struct, so bindgen has no reason to
/// emit it. Both fields are `__int64` in the header on every arch — do NOT widen to `isize`.
#[repr(C)]
pub struct SHQUERYRBINFO {
    pub cbSize: DWORD,
    pub i64Size: i64,
    pub i64NumItems: i64,
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryRecycleBinW(
    pszRootPath: LPCWSTR,
    pSHQueryRBInfo: *mut SHQUERYRBINFO,
) -> HRESULT {
    if pszRootPath.is_null()
        || *pszRootPath == 0
        || pSHQueryRBInfo.is_null()
        || ((*pSHQueryRBInfo).cbSize as usize) < core::mem::size_of::<SHQUERYRBINFO>()
    {
        return E_INVALIDARG;
    }

    (*pSHQueryRBInfo).i64Size = 0;
    (*pSHQueryRBInfo).i64NumItems = 0;

    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
    let hr = GetDefaultRecycleBin(pszRootPath, core::ptr::addr_of_mut!(prb));
    if hr < 0 {
        return hr;
    }

    let mut penum: *mut IRecycleBinEnumList = core::ptr::null_mut();
    let hr = ((*(*prb).lpVtbl).EnumObjects)(prb, core::ptr::addr_of_mut!(penum));
    if hr < 0 {
        ((*(*prb).lpVtbl).Release)(prb);
        return hr;
    }

    // The C returns straight out of the loop, leaking neither pointer because both are
    // CComPtr there. Here the two releases are explicit on every exit, so the loop sets
    // `result` and breaks instead of returning.
    let mut result = S_OK;
    loop {
        let mut pfile: *mut IRecycleBinFile = core::ptr::null_mut();
        let hr = ((*(*penum).lpVtbl).Next)(
            penum,
            1,
            core::ptr::addr_of_mut!(pfile),
            core::ptr::null_mut(),
        );
        if hr == S_FALSE {
            break;
        }
        if hr < 0 {
            result = hr;
            break;
        }

        let mut size: ULARGE_INTEGER = core::mem::zeroed();
        let hr = ((*(*pfile).lpVtbl).GetFileSize)(pfile, core::ptr::addr_of_mut!(size));
        if hr < 0 {
            ((*(*pfile).lpVtbl).Release)(pfile);
            result = hr;
            break;
        }

        (*pSHQueryRBInfo).i64Size += size.QuadPart as i64;
        (*pSHQueryRBInfo).i64NumItems += 1;
        ((*(*pfile).lpVtbl).Release)(pfile);
    }

    ((*(*penum).lpVtbl).Release)(penum);
    ((*(*prb).lpVtbl).Release)(prb);
    result
}

#[no_mangle]
pub unsafe extern "system" fn SHQueryRecycleBinA(
    pszRootPath: LPCSTR,
    pSHQueryRBInfo: *mut SHQUERYRBINFO,
) -> HRESULT {
    let mut szRootPathW: LPWSTR = core::ptr::null_mut();

    if !pszRootPath.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, pszRootPath, -1, core::ptr::null_mut(), 0);
        if len == 0 {
            return HRESULT_FROM_WIN32(GetLastError() as i32);
        }
        szRootPathW = HeapAlloc(
            GetProcessHeap(),
            0,
            len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as LPWSTR;
        if szRootPathW.is_null() {
            return E_OUTOFMEMORY;
        }
        if MultiByteToWideChar(CP_ACP, 0, pszRootPath, -1, szRootPathW, len) == 0 {
            let e = HRESULT_FROM_WIN32(GetLastError() as i32);
            HeapFree(GetProcessHeap(), 0, szRootPathW as *mut c_void);
            return e;
        }
    }

    let hr = SHQueryRecycleBinW(szRootPathW, pSHQueryRBInfo);
    HeapFree(GetProcessHeap(), 0, szRootPathW as *mut c_void);
    hr
}
