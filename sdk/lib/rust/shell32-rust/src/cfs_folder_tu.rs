//! The retained closure of `dll/win32/shell32/folders/CFSFolder.cpp` — its FREE functions plus
//! the 14 `*_RustBridge` C entry points the TU published.
//!
//! CFSFolder was already a HYBRID: the activated class lives in `cfs_folder.rs` and the C++ TU
//! was retained only for this closure. Measured against the real i386 link it was 21 symbols —
//! the 14 bridges here, plus 7 C++-mangled ones wanted by CRecycleBin.cpp, CDesktopFolder.cpp
//! and CDefView.cpp. Everything else the 2,283-line TU defined was already dead code, which is
//! why the buy was ~500 lines.
//!
//! The six `CFSFolder::` statics the bridges publish are NOT re-implemented here: `cfs_folder.rs`
//! already owns them as `CFSFolder_*_body`, so each bridge forwards to that one implementation
//! with a null `this` (they are statics; none of the six touches an instance field). Translating
//! them a second time would have been two implementations of one function.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_05F96F56AE26: [WCHAR; 13] = [0x004E, 0x0065, 0x0076, 0x0065, 0x0072, 0x0053, 0x0068, 0x006F, 0x0077, 0x0045, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_0EEFFF37A29C: [WCHAR; 13] = [0x005C, 0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_173B87169959: [WCHAR; 38] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0041, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_31CCAA880800: [WCHAR; 12] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_3BE8B25B7ED1: [WCHAR; 12] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_714C3CA85096: [WCHAR; 14] = [0x0050, 0x0065, 0x0072, 0x0063, 0x0065, 0x0069, 0x0076, 0x0065, 0x0064, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7ABCE230ED71: [WCHAR; 5] = [0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7C8F589C7B27: [WCHAR; 3] = [0x0025, 0x0031, 0x0000];
pub static CPP_COM_WSTR_932FF3CCC020: [WCHAR; 16] = [0x002E, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9CC9D0F2200B: [WCHAR; 10] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0049, 0x006E, 0x0064, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_A38E93E48CB3: [WCHAR; 11] = [0x0041, 0x0074, 0x0074, 0x0072, 0x0069, 0x0062, 0x0075, 0x0074, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_AFF3436D7481: [WCHAR; 6] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static CPP_COM_WSTR_B730BA9299D0: [WCHAR; 5] = [0x002E, 0x0073, 0x0063, 0x0072, 0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C14D9C3E5FFD: [WCHAR; 67] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0042, 0x006C, 0x006F, 0x0063, 0x006B, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_D3179BE17902: [WCHAR; 4] = [0x006C, 0x006E, 0x006B, 0x0000];
pub static CPP_COM_WSTR_E23C415AD208: [WCHAR; 7] = [0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0032, 0x0000];
pub static CPP_COM_WSTR_E9AA956CFC7B: [WCHAR; 9] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_F7C0A11AA10F: [WCHAR; 13] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0052, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
// This module is the same translation unit as `cfs_folder.rs`, so it takes its substrate from
// there rather than re-deriving one: the SFGAO_*/SHCOLSTATE_* constants, the `_IL*` PIDL
// accessors and the folder-column tables are that TU's closure, already proven by the class half.
use crate::cfs_folder::*;
// A COM interface the island renders in more than one place: name the owner so the two
// globs above cannot disagree. `cfs_folder`'s definitions are the ones this TU's own
// functions are typed against, so they are the ones to pick.
use crate::cfs_folder::{IEnumIDList, IShellFolder, IStream};
// Free functions this TU's closure needs that are private in the module that hosts their real
// (still-C or already-Rust) definition. Each is `pub` there — just not reachable through the
// globs above — so these are imports, not new declarations.
use crate::_countof;
use crate::wine::classes::HCR_GetIconW;
use crate::wine::pidl::ILGetDisplayNameExW;
use crate::wine::shellord::ParseFieldW;
use crate::shell32::shlfolder::SHELL_GetDefaultFolderEnumSHCONTF;
use crate::win32::{
    GetFullPathNameW, PathFileExistsW, PathUnquoteSpacesW, PrivateExtractIconsW, RegOpenKeyW,
    RegQueryValueW, wsprintfW,
};
use crate::prelude::{
    GetPrivateProfileIntW, PathParseIconLocationW, PathRemoveExtensionW,
    SHCreateDefaultExtractIcon, SHELL_GetSetting, SHQueryValueExW, SHRegGetCLSIDKeyW,
    SHRegGetValueW,
};

/// strsafe.h `StringCbCopyW` — a byte-count STRSAFEAPI inline (no export symbol), same as the
/// precedent in `dialogs/filedefext.rs` / `shell32/shlfileop.rs`: convert bytes to WCHARs and
/// hand off to the char-count form this TU already owns via `cfs_folder::StringCchCopyW`.
#[inline]
unsafe fn StringCbCopyW(dst: LPWSTR, bytes: usize, src: LPCWSTR) -> HRESULT {
    StringCchCopyW(dst, bytes / core::mem::size_of::<WCHAR>(), src)
}

// ---- file-scope preamble (translated) ----
pub const SUPER_HIDDEN: u32 = FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM;

// `shvheader` / `GenericSFHeader` / `GENERICSHELLVIEWCOLUMNS` (CFSFolder.cpp:594-603) are NOT
// redefined here: `cfs_folder.rs` already carries the correct translation (colnameid/colstate/
// fmt/cxChar as WORD, matching the real `shfldr.h` layout) and this TU's functions never
// reference the table directly, so the glob import above resolves it — a second copy would
// just be two implementations of one file-scope table (see the module doc comment re: the six
// CFSFolder:: statics for the same pattern).
unsafe extern "C" fn ItemIsFolder(pidl: PCUITEMID_CHILD) -> u8 {
    let mask: u8 = PT_FS | PT_FS_FOLDER_FLAG | PT_FS_FILE_FLAG;
    let type_: u8 = _ILGetType(pidl);
    if (type_ & mask) == (PT_FS | PT_FS_FOLDER_FLAG) || (type_ == PT_FS && ILGetNext(pidl) != core::ptr::null_mut()) {
        1u8
    } else {
        0u8
    }
}
unsafe extern "C" fn GetItemFileName(pidl: PCUITEMID_CHILD, Buf: PWSTR, cchMax: u32) -> *const u16 {
    let pDataW = _ILGetFileStructW(pidl);
    if !pDataW.is_null() {
        // FileStructW is #[repr(C, packed)]; wszName (WCHAR, align 2) cannot be referenced
        // in place (E0793) the way szNames (CHAR, align 1) below can — take its address instead
        // of `.as_ptr()`, matching `cfs_folder::GetItemFileName_body`'s identical fix.
        return core::ptr::addr_of!((*pDataW).wszName) as *const u16;
    }
    let pdata = _ILGetDataPointer(pidl);
    if _ILGetFSType(pidl) & PT_FS_UNICODE_FLAG != 0 {
        return (*pdata).u.file.szNames.as_ptr() as PWSTR;
    }
    if _ILSimpleGetTextW(pidl, Buf, cchMax) != 0 {
        return Buf;
    }
    core::ptr::null()
}
unsafe extern "C" fn IsRealItem(idl: *const ITEMIDLIST) -> u32 {
    let fsitem: *mut FileStruct = core::ptr::addr_of_mut!(
        (*((*idl).mkid.abID.as_ptr() as *mut PIDLDATA)).u.file
    );
    ((*fsitem).dwFileSize | (*fsitem).uFileDate as u32) as u32
}
unsafe extern "C" fn GetItemDescription(pidl: PCUITEMID_CHILD, Buf: PWSTR, cchMax: u32) {
    let mut hr = E_FAIL;
    if ItemIsFolder(pidl) != 0 {
        hr = SHELL32_AssocGetFSDirectoryDescription(Buf, cchMax);
    } else {
        let mut temp: [u16; MAX_PATH] = [0; MAX_PATH];
        let name = GetItemFileName(pidl, temp.as_mut_ptr(), _countof!(temp));
        hr = SHELL32_AssocGetFileDescription(
            if !name.is_null() { name } else { CPP_COM_WSTR_96A296D224F2.as_ptr() },
            Buf,
            cchMax
        );
    }
    if ((hr) < 0) && cchMax != 0 {
        *Buf.offset(0 as isize) = UNICODE_NULL;
    }
}
unsafe extern "C" fn OpenKeyFromFileType(pExtension: LPCWSTR, KeyName: LPCWSTR) -> HKEY {
    let mut hkey: HKEY = core::ptr::null_mut();

    let mut FullName: [u16; MAX_PATH] = [0; MAX_PATH];
    let dwSize: DWORD = core::mem::size_of_val(&FullName) as DWORD;
    wsprintfW(FullName.as_mut_ptr(), CPP_COM_WSTR_BA2342D760A2.as_ptr(), pExtension, KeyName);

    let mut res: LONG = RegOpenKeyExW(HKEY_CLASSES_ROOT, FullName.as_ptr(), 0, KEY_READ, &mut hkey);
    if res == 0 {
        return hkey;
    }

    let mut dwSize_local = dwSize;
    res = RegGetValueW(HKEY_CLASSES_ROOT, pExtension, core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), FullName.as_mut_ptr() as *mut c_void, &mut dwSize_local);
    if res != 0 {
        return core::ptr::null_mut();
    }

    wcscat(FullName.as_mut_ptr(), CPP_COM_WSTR_B7FC02907B1F.as_ptr());
    wcscat(FullName.as_mut_ptr(), KeyName);

    hkey = core::ptr::null_mut();
    res = RegOpenKeyExW(HKEY_CLASSES_ROOT, FullName.as_ptr(), 0, KEY_READ, &mut hkey);
    if res != 0 {
        WARN!("Could not open key %S for extension %S\n", KeyName, pExtension);
    }

    hkey
}
unsafe extern "C" fn ExtensionFromPidl(pidl: PCUIDLIST_RELATIVE, Buf: PWSTR, cchMax: u32, AllowFolder: u8) -> *const u16 {
    // CFSFolder.cpp:93 `if (!AllowFolder && !_ILIsValue(pidl))` — the shipped translation had
    // garbled this into `_ILIsValue(AllowFolder)` plus a 1-arg `GetItemFileName(pidl)` and a
    // syntactically-broken `if(name)`; rebuilt straight from the C++ below.
    if AllowFolder == 0 && _ILIsValue(pidl) == 0 {
        ERR!("Invalid pidl!\n");
        return core::ptr::null();
    }

    let name = GetItemFileName(pidl, Buf, cchMax);
    let pExtension: *const u16 = if !name.is_null() { PathFindExtensionW(name) } else { core::ptr::null_mut() };
    if pExtension.is_null() || *pExtension == UNICODE_NULL {
        WARN!("No extension for %S!\n", name);
        return core::ptr::null();
    }
    pExtension
}
unsafe extern "C" fn GetCLSIDForFileTypeFromExtension(pExtension: LPCWSTR, KeyName: LPCWSTR, pclsid: *mut CLSID) -> HRESULT {
    let hkeyProgId = OpenKeyFromFileType(pExtension, KeyName);
    if hkeyProgId.is_null() {
        WARN!("OpenKeyFromFileType failed for key %S\n", KeyName);
        return S_FALSE;
    }

    let mut wszCLSIDValue: [u16; CHARS_IN_GUID as usize] = core::mem::zeroed();
    let mut dwSize: u32 = core::mem::size_of_val(&wszCLSIDValue) as u32;
    let res = RegGetValueW(hkeyProgId, core::ptr::null(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), wszCLSIDValue.as_mut_ptr() as *mut c_void, &mut dwSize);
    RegCloseKey(hkeyProgId);
    if res != 0 {
        ERR!("OpenKeyFromFileType succeeded but RegGetValueW failed\n");
        return S_FALSE;
    }

    if RegGetValueW(HKEY_LOCAL_MACHINE,
                     CPP_COM_WSTR_C14D9C3E5FFD.as_ptr(),
                     wszCLSIDValue.as_ptr(),
                     RRF_RT_REG_SZ,
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     core::ptr::null_mut()) == ERROR_SUCCESS {
        ERR!("Extension %S  not approved\n", wszCLSIDValue.as_ptr());
        return E_ACCESSDENIED;
    }

    let hres = CLSIDFromString(wszCLSIDValue.as_ptr(), pclsid);
    if hres < 0 {
        return hres;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn GetCLSIDForFileType(pidl: PCUIDLIST_RELATIVE, KeyName: LPCWSTR, pclsid: *mut CLSID) -> HRESULT {
    let mut buf: [u16; 256] = [0; 256];
    let pExtension = ExtensionFromPidl(pidl, buf.as_mut_ptr(), _countof!(buf), FALSE as u8);
    if pExtension.is_null() {
        return S_FALSE;
    }

    GetCLSIDForFileTypeFromExtension(pExtension, KeyName, pclsid)
}

unsafe fn GetItemCLSID(pidl: PCUIDLIST_RELATIVE, pclsid: *mut CLSID) -> HRESULT {
    let mut buf: [u16; 256] = core::mem::zeroed();
    let pExt = ExtensionFromPidl(pidl, buf.as_mut_ptr(), _countof!(buf), TRUE as u8);
    if pExt.is_null() {
        return E_FAIL;
    }
    let mut hr = E_FAIL;
    if ItemIsFolder(pidl) == 0 {
        hr = GetCLSIDForFileTypeFromExtension(pExt, CPP_COM_WSTR_AFF3436D7481.as_ptr(), pclsid);
    }
    if hr != S_OK && *pExt.offset(0 as isize) == '.' as u16 && *pExt.offset(1 as isize) == '{' as u16 {
        hr = CLSIDFromString(pExt.offset(1 as isize), pclsid);
    }
    hr
}
unsafe extern "C" fn getDefaultIconLocation(szIconFile: PWSTR, cchMax: u32, piIndex: *mut i32, uFlags: u32) -> i32 {
    if HLM_GetIconW((IDI_SHELL_FOLDER as i32) - 1, szIconFile, cchMax, piIndex) == 0 {
        if HCR_GetIconW(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), szIconFile, core::ptr::null_mut(), cchMax, piIndex) == 0 {
            StringCchCopyW(szIconFile, cchMax as usize, core::ptr::addr_of!(swShell32Name).cast());
            *piIndex = -(IDI_SHELL_FOLDER as i32);
        }
    }

    if uFlags & GIL_OPENICON != 0 {
        if *piIndex < 0 {
            (*piIndex) -= 1;
        } else {
            (*piIndex) += 1;
        }
    }

    S_OK
}
unsafe extern "C" fn getShellClassInfo(Entry: LPCWSTR, pszValue: LPWSTR, cchValueLen: u32, IniFile: LPCWSTR) -> i32 {
    GetPrivateProfileStringW(CPP_COM_WSTR_932FF3CCC020.as_ptr(), Entry, core::ptr::null(), pszValue, cchValueLen, IniFile) as i32
}
unsafe extern "C" fn getIconLocationForFolder(
    psf: *mut IShellFolder,
    pidl: PCUITEMID_CHILD,
    uFlags: u32,
    szIconFile: PWSTR,
    cchMax: u32,
    piIndex: *mut i32,
    pwFlags: *mut u32,
) -> HRESULT {
    let mut dwFileAttrs: u32;
    let mut wszPath: [u16; MAX_PATH] = core::mem::zeroed();
    let mut wszIniFullPath: [u16; MAX_PATH] = core::mem::zeroed();

    if (uFlags & GIL_DEFAULTICON) != 0 {
        return getDefaultIconLocation(szIconFile, cchMax, piIndex, uFlags);
    }

    if ILGetDisplayNameExW(psf.cast(), pidl, wszPath.as_mut_ptr(), 0) == 0 {
        return getDefaultIconLocation(szIconFile, cchMax, piIndex, uFlags);
    }
    if PathIsDirectoryW(wszPath.as_ptr()) == 0 {
        return getDefaultIconLocation(szIconFile, cchMax, piIndex, uFlags);
    }

    dwFileAttrs = _ILGetFileAttributes(ILFindLastID(pidl), core::ptr::null_mut(), 0);
    if (dwFileAttrs & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)) == 0 {
        return getDefaultIconLocation(szIconFile, cchMax, piIndex, uFlags);
    }

    StringCchCopyW(wszIniFullPath.as_mut_ptr(), _countof!(wszIniFullPath) as usize, wszPath.as_ptr());
    PathAppendW(wszIniFullPath.as_mut_ptr(), CPP_COM_WSTR_A5DBFDEBD401.as_ptr());

    let mut wszValue: [u16; MAX_PATH] = core::mem::zeroed();
    let mut wszTemp: [u16; MAX_PATH] = core::mem::zeroed();
    if getShellClassInfo(CPP_COM_WSTR_E9AA956CFC7B.as_ptr(), wszValue.as_mut_ptr(), _countof!(wszValue), wszIniFullPath.as_ptr()) != 0 {
        ExpandEnvironmentStringsW(wszValue.as_ptr(), wszTemp.as_mut_ptr(), _countof!(wszTemp));

        if PathIsRelativeW(wszTemp.as_ptr()) != 0 {
            PathAppendW(wszPath.as_mut_ptr(), wszTemp.as_ptr());
        } else {
            StringCchCopyW(wszPath.as_mut_ptr(), _countof!(wszPath) as usize, wszTemp.as_ptr());
        }

        GetFullPathNameW(wszPath.as_ptr(), cchMax, szIconFile, core::ptr::null_mut());

        *piIndex = GetPrivateProfileIntW(CPP_COM_WSTR_932FF3CCC020.as_ptr(), CPP_COM_WSTR_9CC9D0F2200B.as_ptr(), 0, wszIniFullPath.as_ptr()) as i32;
        return S_OK;
    } else if getShellClassInfo(CPP_COM_WSTR_AFF3436D7481.as_ptr(), wszValue.as_mut_ptr(), _countof!(wszValue), wszIniFullPath.as_ptr()) != 0
        && HCR_GetIconW(wszValue.as_ptr(), szIconFile, core::ptr::null_mut(), cchMax, piIndex) != 0
    {
        return S_OK;
    } else if getShellClassInfo(CPP_COM_WSTR_E23C415AD208.as_ptr(), wszValue.as_mut_ptr(), _countof!(wszValue), wszIniFullPath.as_ptr()) != 0
        && HCR_GetIconW(wszValue.as_ptr(), szIconFile, core::ptr::null_mut(), cchMax, piIndex) != 0
    {
        return S_OK;
    } else if getShellClassInfo(CPP_COM_WSTR_F7C0A11AA10F.as_ptr(), wszValue.as_mut_ptr(), _countof!(wszValue), wszIniFullPath.as_ptr()) != 0 {
        ExpandEnvironmentStringsW(wszValue.as_ptr(), wszTemp.as_mut_ptr(), _countof!(wszTemp));

        *piIndex = PathParseIconLocationW(wszTemp.as_mut_ptr());

        if PathIsRelativeW(wszTemp.as_ptr()) != 0 {
            PathAppendW(wszPath.as_mut_ptr(), wszTemp.as_ptr());
        } else {
            StringCchCopyW(wszPath.as_mut_ptr(), _countof!(wszPath) as usize, wszTemp.as_ptr());
        }

        GetFullPathNameW(wszPath.as_ptr(), cchMax, szIconFile, core::ptr::null_mut());
        return S_OK;
    }

    getDefaultIconLocation(szIconFile, cchMax, piIndex, uFlags)
}
#[no_mangle]
pub unsafe extern "C" fn CFSExtractIcon_CreateInstance(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    iid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let hr: HRESULT;
    let mut icon_idx: i32 = 0;
    let mut flags: u32 = 0;
    let mut wTemp: [u16; MAX_PATH] = [0; MAX_PATH];
    core::ptr::copy_nonoverlapping(
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        wTemp.as_mut_ptr(),
        CPP_COM_WSTR_96A296D224F2.len().min(MAX_PATH),
    );

    hr = SHCreateDefaultExtractIcon(core::ptr::addr_of!(IID_IDefaultExtractIconInit).cast(), core::ptr::addr_of_mut!(initIcon).cast());
    if ((hr) < 0) {
        return hr;
    }

    if _ILIsFolder(pidl) != 0 {
        if ((getIconLocationForFolder(
            psf,
            pidl,
            0,
            wTemp.as_mut_ptr(),
            _countof!(wTemp),
            &mut icon_idx,
            &mut flags,
        )) >= 0) {
            ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
            ((*(*initIcon).lpVtbl).SetShortcutIcon)(initIcon, wTemp.as_ptr(), icon_idx);
        }
        if ((getIconLocationForFolder(
            psf,
            pidl,
            GIL_DEFAULTICON,
            wTemp.as_mut_ptr(),
            _countof!(wTemp),
            &mut icon_idx,
            &mut flags,
        )) >= 0) {
            ((*(*initIcon).lpVtbl).SetDefaultIcon)(initIcon, wTemp.as_ptr(), icon_idx);
        }
        if ((getIconLocationForFolder(
            psf,
            pidl,
            GIL_OPENICON,
            wTemp.as_mut_ptr(),
            _countof!(wTemp),
            &mut icon_idx,
            &mut flags,
        )) >= 0) {
            ((*(*initIcon).lpVtbl).SetOpenIcon)(initIcon, wTemp.as_ptr(), icon_idx);
        }
    } else {
        let mut extbuf: [u16; 256] = [0; 256];
        let pExtension: LPCWSTR = ExtensionFromPidl(pidl, extbuf.as_mut_ptr(), _countof!(extbuf), FALSE as u8);
        let mut hkey: HKEY = if !pExtension.is_null() {
            OpenKeyFromFileType(pExtension, CPP_COM_WSTR_3BE8B25B7ED1.as_ptr())
        } else {
            core::ptr::null_mut()
        };

        if hkey.is_null() {
            let mut dwSize: u32 = core::mem::size_of_val(&wTemp) as u32;
            if RegGetValueW(
                HKEY_CLASSES_ROOT,
                pExtension,
                CPP_COM_WSTR_714C3CA85096.as_ptr(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                wTemp.as_mut_ptr() as *mut c_void,
                &mut dwSize,
            ) == ERROR_SUCCESS
            {
                let mut szSubKey: [u16; MAX_PATH] = [0; MAX_PATH];
                strsafe_rust::StringCchPrintfW!(
                    szSubKey.as_mut_ptr(),
                    _countof!(szSubKey) as usize,
                    CPP_COM_WSTR_173B87169959.as_ptr(),
                    wTemp.as_ptr(),
                );
                RegOpenKeyExW(
                    HKEY_CLASSES_ROOT,
                    szSubKey.as_ptr(),
                    0,
                    KEY_READ,
                    &mut hkey,
                );
            }
        }

        let mut dwSize: u32 = core::mem::size_of_val(&wTemp) as u32;
        if !hkey.is_null()
            && SHQueryValueExW(
                hkey.cast(),
                core::ptr::null(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                wTemp.as_mut_ptr() as *mut c_void,
                &mut dwSize,
            ) == 0
        {
            let mut sNum: [u16; 5] = [0; 5];
            if ParseFieldW(
                wTemp.as_ptr(),
                2,
                sNum.as_mut_ptr(),
                5,
            ) != 0
            {
                icon_idx = _wtoi(sNum.as_ptr());
            } else {
                icon_idx = 0;
            }
            ParseFieldW(
                wTemp.as_ptr(),
                1,
                wTemp.as_mut_ptr(),
                MAX_PATH as u32,
            );
            PathUnquoteSpacesW(wTemp.as_mut_ptr());

            if wcscmp(CPP_COM_WSTR_7C8F589C7B27.as_ptr(), wTemp.as_ptr()) == 0 {
                ILGetDisplayNameExW(psf.cast(), pidl, wTemp.as_mut_ptr(), ILGDN_FORPARSING);
                icon_idx = 0;

                let ret: i32 = PrivateExtractIconsW(
                    wTemp.as_ptr(),
                    0,
                    0,
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                    0,
                ) as i32;
                if ret <= 0 {
                    StringCbCopyW(wTemp.as_mut_ptr(), core::mem::size_of_val(&wTemp), core::ptr::addr_of!(swShell32Name).cast());
                    if lstrcmpiW(pExtension, CPP_COM_WSTR_7ABCE230ED71.as_ptr())
                        == 0
                        || lstrcmpiW(pExtension, CPP_COM_WSTR_B730BA9299D0.as_ptr())
                            == 0
                    {
                        icon_idx = -(IDI_SHELL_EXE as i32);
                    } else {
                        icon_idx = -(IDI_SHELL_DOCUMENT as i32);
                    }
                }
            }

            ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
        } else {
            ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, core::ptr::addr_of!(swShell32Name).cast(), 0);
        }

        if !hkey.is_null() {
            RegCloseKey(hkey);
        }
    }

    ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, iid, ppvOut)
}
unsafe extern "C" fn SHELL32_GetCLSIDForDirectory(pwszDir: LPCWSTR, KeyName: LPCWSTR, pclsidFolder: *mut CLSID) -> HRESULT {
    let mut wszCLSIDValue: [u16; CHARS_IN_GUID as usize] = core::mem::zeroed();
    let mut wszDesktopIni: [u16; MAX_PATH] = core::mem::zeroed();

    StringCchCopyW(wszDesktopIni.as_mut_ptr(), MAX_PATH, pwszDir);
    StringCchCatW(wszDesktopIni.as_mut_ptr(), MAX_PATH, CPP_COM_WSTR_0EEFFF37A29C.as_ptr());

    if GetPrivateProfileStringW(CPP_COM_WSTR_932FF3CCC020.as_ptr(),
                                 KeyName,
                                 CPP_COM_WSTR_96A296D224F2.as_ptr(),
                                 wszCLSIDValue.as_mut_ptr(),
                                 CHARS_IN_GUID,
                                 wszDesktopIni.as_ptr()) != 0
    {
        return CLSIDFromString(wszCLSIDValue.as_ptr(), pclsidFolder);
    }
    return E_FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_GetFSItemAttributes(psf: *mut IShellFolder, pidl: LPCITEMIDLIST, pdwAttributes: *mut SFGAOF) -> HRESULT {
    let mut dwFileAttributes: u32;
    let mut dwShellAttributes: SFGAOF;

    if _ILIsFolderOrFile(pidl) == 0 {
        ERR!("Got wrong type of pidl!\n");
        *pdwAttributes &= SFGAO_CANLINK;
        return S_OK;
    }

    dwFileAttributes = _ILGetFileAttributes(pidl, core::ptr::null_mut(), 0);

    dwShellAttributes = SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANLINK | SFGAO_CANRENAME | SFGAO_CANDELETE |
                        SFGAO_HASPROPSHEET | SFGAO_DROPTARGET | SFGAO_FILESYSTEM;

    let mut bDirectory = (dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

    if SFGAO_VALIDATE & *pdwAttributes != 0 {
        let mut strret: STRRET = core::mem::zeroed();
        let mut path: PWSTR = core::ptr::null_mut();
        if (((*(*psf).lpVtbl).GetDisplayNameOf)(psf, pidl, SHGDN_FORPARSING, &mut strret)) >= 0 &&
           (StrRetToStrW(&mut strret, pidl, &mut path)) >= 0 {
            let exists = PathFileExistsW(path);
            SHFree(path);
            if exists == 0 {
                return E_FAIL;
            }
        }
    }

    if !bDirectory {
        let mut szFileName: [u16; MAX_PATH] = core::mem::zeroed();
        let mut pExtension: PWSTR;
        let hasName = _ILSimpleGetTextW(pidl, szFileName.as_mut_ptr(), _countof!(szFileName));
        dwShellAttributes |= SFGAO_STREAM;

        if hasName != 0 && szFileName[0] == 0x007Eu16 && (dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
            dwShellAttributes |= SFGAO_HIDDEN | SFGAO_SYSTEM;
        }

        if hasName != 0 {
            pExtension = PathFindExtensionW(szFileName.as_mut_ptr());
            if !pExtension.is_null() {
                let mut clsidFile: CLSID = core::mem::zeroed();
                let mut hr = GetCLSIDForFileTypeFromExtension(pExtension, CPP_COM_WSTR_AFF3436D7481.as_ptr(), &mut clsidFile);
                if hr == S_OK {
                    let mut hkey: *mut core::ffi::c_void = core::ptr::null_mut();
                    hr = SHRegGetCLSIDKeyW(core::ptr::addr_of!(clsidFile).cast(), CPP_COM_WSTR_31CCAA880800.as_ptr(), FALSE, FALSE, core::ptr::addr_of_mut!(hkey).cast());
                    if hr >= 0 {
                        let mut dwAttributes: u32 = 0;
                        let mut dwSize: u32 = core::mem::size_of::<u32>() as u32;
                        let Status = SHRegGetValueW(hkey.cast(), core::ptr::null_mut(), CPP_COM_WSTR_A38E93E48CB3.as_ptr(), RRF_RT_REG_DWORD as i32, core::ptr::null_mut(), core::ptr::addr_of_mut!(dwAttributes).cast(), &mut dwSize);
                        if Status == STATUS_SUCCESS as i32 {
                            dwShellAttributes |= dwAttributes;
                        }
                        RegCloseKey(hkey);

                        bDirectory = (dwAttributes & SFGAO_FOLDER) != 0 || dwAttributes == 0;
                    }
                }
            }
        }
    }

    if bDirectory {
        dwShellAttributes |= SFGAO_FOLDER | SFGAO_STORAGE;

        if (dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            dwShellAttributes |= SFGAO_FILESYSANCESTOR | SFGAO_STORAGEANCESTOR;
        }
    }

    if (dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
        dwShellAttributes |= SFGAO_HIDDEN | SFGAO_GHOSTED;
    }

    if (dwFileAttributes & FILE_ATTRIBUTE_READONLY) != 0 {
        dwShellAttributes |= SFGAO_READONLY;
    }

    if (dwFileAttributes & FILE_ATTRIBUTE_SYSTEM) != 0 {
        dwShellAttributes |= SFGAO_SYSTEM;
    }

    if (dwFileAttributes & FILE_ATTRIBUTE_COMPRESSED) != 0 {
        dwShellAttributes |= SFGAO_COMPRESSED;
    }

    if (dwFileAttributes & FILE_ATTRIBUTE_ENCRYPTED) != 0 {
        dwShellAttributes |= SFGAO_ENCRYPTED;
    }

    if (SFGAO_NONENUMERATED & *pdwAttributes) != 0 && (dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) != 0 {
        let shcf = SHELL_GetDefaultFolderEnumSHCONTF();
        if (shcf & SHCONTF_INCLUDEHIDDEN) == 0 || ((dwFileAttributes & FILE_ATTRIBUTE_SYSTEM) != 0 && (shcf & SHCONTF_INCLUDESUPERHIDDEN) == 0) {
            dwShellAttributes |= SFGAO_NONENUMERATED;
        }
    }

    if SFGAO_LINK & *pdwAttributes != 0 {
        let mut ext: [u16; MAX_PATH] = core::mem::zeroed();

        if _ILGetExtension(pidl, ext.as_mut_ptr(), _countof!(ext)) != 0 && lstrcmpiW(ext.as_ptr(), CPP_COM_WSTR_D3179BE17902.as_ptr()) == 0 {
            dwShellAttributes |= SFGAO_LINK;
        }
    }

    if SFGAO_HASSUBFOLDER & *pdwAttributes != 0 {
        let mut psf2: *mut IShellFolder = core::ptr::null_mut();
        // IID_PPV_ARG(IShellFolder, &psf2) expands to IID_IShellFolder, (void**)&psf2.
        if ((*(*psf).lpVtbl).BindToObject)(
            psf,
            pidl,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(psf2).cast(),
        ) >= 0 {
            let mut pEnumIL: *mut IEnumIDList = core::ptr::null_mut();
            if ((*(*psf2).lpVtbl).EnumObjects)(psf2, core::ptr::null_mut(), SHCONTF_FOLDERS, &mut pEnumIL) >= 0 {
                if ((*(*pEnumIL).lpVtbl).Skip)(pEnumIL, 1) == S_OK {
                    dwShellAttributes |= SFGAO_HASSUBFOLDER;
                }
                ((*(*pEnumIL).lpVtbl).Release)(pEnumIL);
            }
            ((*(*psf2).lpVtbl).Release)(psf2);
        }
    }

    *pdwAttributes = dwShellAttributes;

    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_FS_HideExtension(szPath: LPCWSTR) -> BOOL {
    // `shfldr.h: EXTERN_C BOOL SHELL_FS_HideExtension(LPCWSTR pwszPath);` — a full-width BOOL,
    // linked directly from CDefView.cpp. The shipped translation returned `u8` (only sets AL;
    // the C++ caller reads all 32 bits of EAX) and had mangled the field-macro call
    // `SHELL_GetSetting(SSF_SHOWEXTENSIONS, fShowExtensions)` into a null-pointer expression —
    // rebuilt as the real `SHELL_GetSetting!` macro (shellutils.h) below.
    let mut hKey: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut doHide: BOOL = FALSE;
    let mut lError: i32;

    doHide = (SHELL_GetSetting!(SSF_SHOWEXTENSIONS, fShowExtensions) == 0) as BOOL;

    if doHide == 0 {
        let DotExt: LPCWSTR = PathFindExtensionW(szPath);
        if *DotExt != 0 {
            let mut classname: [u16; MAX_PATH] = [0; MAX_PATH];
            let mut classlen: i32 = core::mem::size_of_val(&classname) as i32;
            lError = RegQueryValueW(HKEY_CLASSES_ROOT, DotExt, classname.as_mut_ptr(), core::ptr::addr_of_mut!(classlen));
            if lError == ERROR_SUCCESS {
                lError = RegOpenKeyW(HKEY_CLASSES_ROOT, classname.as_ptr(), core::ptr::addr_of_mut!(hKey));
                if lError == ERROR_SUCCESS {
                    lError = RegQueryValueExW(hKey, CPP_COM_WSTR_05F96F56AE26.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
                    if lError == ERROR_SUCCESS {
                        doHide = TRUE;
                    }

                    RegCloseKey(hKey);
                }
            }
        }
    }

    doHide
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_FS_ProcessDisplayFilename(szPath: PWSTR, dwFlags: u32) {
    if (dwFlags & SHGDN_FORPARSING) == 0 &&
        ((dwFlags & SHGDN_INFOLDER) != 0 || dwFlags == SHGDN_NORMAL) {
        if SHELL_FS_HideExtension(szPath) != 0 && *szPath.offset(0 as isize) != b'.' as u16 {
            PathRemoveExtensionW(szPath);
        }
    }
}
unsafe extern "C" fn DoLoadPicture(pszFileName: LPCWSTR) -> HBITMAP {
    let mut hr: HRESULT;
    let mut pStream: *mut IStream = core::ptr::null_mut();
    hr = SHCreateStreamOnFileEx(
        pszFileName,
        STGM_READ,
        FILE_ATTRIBUTE_NORMAL,
        FALSE,
        core::ptr::null_mut(),
        &mut pStream,
    );
    if ((hr) < 0) {
        return core::ptr::null_mut();
    }

    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut pPicture: *mut IPicture = core::ptr::null_mut();
    OleLoadPicture(
        pStream,
        0,
        FALSE,
        &IID_IPicture,
        &mut pPicture as *mut *mut IPicture as *mut core::ffi::c_void,
    );

    if !pPicture.is_null() {
        ((*(*pPicture).lpVtbl).get_Handle)(pPicture, &mut hbm as *mut HBITMAP as *mut OLE_HANDLE);

        hbm = CopyImage(
            hbm,
            IMAGE_BITMAP,
            0,
            0,
            LR_CREATEDIBSECTION,
        ) as HBITMAP;

        ((*(*pPicture).lpVtbl).Release)(pPicture);
    }

    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).Release)(pStream);
    }

    hbm
}

// The six `CFSFolder::` statics below are C++ static member functions: no `this`, no instance
// state. `cfs_folder.rs` owns their single implementation, so each bridge forwards there with a
// null `this` rather than standing up a second copy.
//
// Two of them take a C++ REFERENCE out-parameter. The generated forms dereferenced the caller's
// pointer and passed the value, which type-checks and silently drops every write the callee
// makes — `GetFSColumnDetails` fills `SHELLDETAILS` and `GetDefaultFSColumnState` fills
// `SHCOLSTATEF`, so both callers would have read uninitialised memory. Reborrow, don't copy.

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_CompareSortFoldersFirst_RustBridge(
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
) -> HRESULT {
    crate::cfs_folder::CFSFolder_CompareSortFoldersFirst_body(core::ptr::null_mut(), pidl1, pidl2)
}

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_CompareUiStrings_RustBridge(
    a: LPCWSTR,
    b: LPCWSTR,
    lParam: LPARAM,
) -> i32 {
    crate::cfs_folder::CFSFolder_CompareUiStrings_body(core::ptr::null_mut(), a, b, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_FormatDateTime_RustBridge(
    ft: *const FILETIME,
    buf: PWSTR,
    cchBuf: u32,
) -> HRESULT {
    crate::cfs_folder::CFSFolder_FormatDateTime_body(core::ptr::null_mut(), &*ft, buf, cchBuf)
}

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_FormatSize_RustBridge(
    size: u64,
    buf: LPWSTR,
    cchBuf: u32,
) -> HRESULT {
    crate::cfs_folder::CFSFolder_FormatSize_body(core::ptr::null_mut(), size, buf, cchBuf)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_FS_ProcessDisplayFilename_RustBridge(
    szPath: LPWSTR,
    dwFlags: DWORD,
) {
    SHELL_FS_ProcessDisplayFilename(szPath, dwFlags);
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_GetFSItemAttributes_RustBridge(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    pdwAttributes: LPDWORD,
) -> HRESULT {
    SHELL32_GetFSItemAttributes(psf, pidl, pdwAttributes)
}

#[no_mangle]
pub unsafe extern "C" fn GetCLSIDForFileType_RustBridge(
    pidl: PCUIDLIST_RELATIVE,
    KeyName: LPCWSTR,
    pclsid: *mut CLSID,
) -> HRESULT {
    GetCLSIDForFileType(pidl, KeyName, pclsid)
}

#[no_mangle]
pub unsafe extern "C" fn SHELL_FS_HideExtension_RustBridge(pwszPath: LPCWSTR) -> BOOL {
    SHELL_FS_HideExtension(pwszPath)
}

#[no_mangle]
pub unsafe extern "C" fn GetItemCLSID_RustBridge(
    pidl: PCUIDLIST_RELATIVE,
    pclsid: *mut CLSID,
) -> HRESULT {
    GetItemCLSID(pidl, pclsid)
}

#[no_mangle]
pub unsafe extern "system" fn CFSExtractIcon_CreateInstance_RustBridge(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    iid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CFSExtractIcon_CreateInstance(psf, pidl, iid, ppvOut)
}

// `SHELL32_CreateSimpleIDListFromPath` has been Rust-owned since the wine/pidl wave; the C++ TU
// only ever republished it under a bridge name for its C++ callers.
#[no_mangle]
pub unsafe extern "system" fn SHELL32_CreateSimpleIDListFromPath_RustBridge(
    pszPath: LPCWSTR,
    dwAttributes: DWORD,
) -> LPITEMIDLIST {
    crate::wine::pidl::SHELL32_CreateSimpleIDListFromPath(pszPath, dwAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn CFileSysEnum_CreateInstance_RustBridge(
    pszPath: LPWSTR,
    dwFlags: u32,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    crate::file_sys_enum::CFileSysEnum_CreateInstance(pszPath, dwFlags, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_GetFSColumnDetails_RustBridge(
    iColumn: UINT,
    sd: *mut SHELLDETAILS,
) -> HRESULT {
    crate::cfs_folder::CFSFolder_GetFSColumnDetails_body(core::ptr::null_mut(), iColumn, &mut *sd)
}

#[no_mangle]
pub unsafe extern "system" fn CFSFolder_GetDefaultFSColumnState_RustBridge(
    iColumn: UINT,
    csFlags: *mut SHCOLSTATEF,
) -> HRESULT {
    crate::cfs_folder::CFSFolder_GetDefaultFSColumnState_body(
        core::ptr::null_mut(),
        iColumn,
        &mut *csFlags,
    )
}
