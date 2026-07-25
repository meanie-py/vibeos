use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct InterfaceDescEntry {
    pub riid: REFIID,
    pub name: *const i8,
}

// Raw pointers are not auto-Sync. This mirrors C's `static const` table: written
// once by the loader, only ever read afterwards.
unsafe impl Sync for InterfaceDescEntry {}

pub static InterfaceDesc: [InterfaceDescEntry; 22] = [
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IUnknown), name: b"IID_IUnknown\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IClassFactory), name: b"IID_IClassFactory\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellView), name: b"IID_IShellView\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IOleCommandTarget), name: b"IID_IOleCommandTarget\0"    .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IDropTarget), name: b"IID_IDropTarget\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IDropSource), name: b"IID_IDropSource\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IViewObject), name: b"IID_IViewObject\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IContextMenu), name: b"IID_IContextMenu\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellExtInit), name: b"IID_IShellExtInit\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellFolder), name: b"IID_IShellFolder\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellFolder2), name: b"IID_IShellFolder2\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IPersist), name: b"IID_IPersist\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IPersistFolder), name: b"IID_IPersistFolder\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IPersistFolder2), name: b"IID_IPersistFolder2\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IPersistFolder3), name: b"IID_IPersistFolder3\0"        .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IExtractIconA), name: b"IID_IExtractIconA\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IExtractIconW), name: b"IID_IExtractIconW\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IDataObject), name: b"IID_IDataObject\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IAutoComplete), name: b"IID_IAutoComplete\0"            .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IAutoComplete2), name: b"IID_IAutoComplete2\0"           .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellLinkA), name: b"IID_IShellLinkA\0"              .as_ptr() as *const i8 },
    InterfaceDescEntry { riid: core::ptr::addr_of!(IID_IShellLinkW), name: b"IID_IShellLinkW\0"              .as_ptr() as *const i8 },
];
unsafe extern "C" fn _dbg_ILGetType(pidl: LPCITEMIDLIST) -> u8 {
    if !pidl.is_null() && (*pidl).mkid.cb >= 3 {
        (*pidl).mkid.abID[0 as usize]
    } else {
        0
    }
}
unsafe extern "C" fn _dbg_ILGetFSType(pidl: LPCITEMIDLIST) -> u8 {
    let type_: u8 = _dbg_ILGetType(pidl);
    if (type_ & PT_FOLDERTYPEMASK) == PT_FS {
        type_
    } else {
        0
    }
}
unsafe extern "C" fn _dbg_ILGetNext(pidl: LPCITEMIDLIST) -> LPITEMIDLIST {
    if pidl.is_null() {
        return core::ptr::null_mut();
    }
    let len = (* (pidl as *const SHITEMID)).cb;
    if len == 0 {
        return core::ptr::null_mut();
    }
    (pidl as *const u8).add(len as usize) as LPITEMIDLIST
}
unsafe extern "C" fn _dbg_ILIsDesktop(pidl: LPCITEMIDLIST) -> i32 {
    (pidl.is_null() || unsafe { (*pidl).mkid.cb == 0x00 }) as i32
}
unsafe extern "C" fn _dbg_ILGetDataPointer(pidl: LPCITEMIDLIST) -> LPPIDLDATA {
    if !pidl.is_null() && (*pidl).mkid.cb != 0x00 {
        return core::ptr::addr_of!((*pidl).mkid.abID) as *const u8 as LPPIDLDATA;
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn _dbg_ILGetTextPointer(pidl: LPCITEMIDLIST) -> LPSTR {
    let pdata = _dbg_ILGetDataPointer(pidl);
    if !pdata.is_null() {
        match (*pdata).type_ {
            PT_GUID | PT_SHELLEXT | PT_YAGUID => {
                return core::ptr::null_mut();
            }
            PT_DRIVE | PT_DRIVE1 | PT_DRIVE2 | PT_DRIVE3 => {
                return core::ptr::addr_of!((*pdata).u.drive.szDriveName) as *mut i8;
            }
            PT_FOLDER | PT_FOLDER1 | PT_VALUE | PT_IESPECIAL1 | PT_IESPECIAL2 => {
                return core::ptr::addr_of!((*pdata).u.file.szNames) as *mut i8;
            }
            PT_WORKGRP | PT_COMP | PT_NETWORK | PT_NETPROVIDER | PT_SHARE => {
                return core::ptr::addr_of!((*pdata).u.network.szNames) as *mut i8;
            }
            _ => {}
        }
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn _dbg_ILGetTextPointerW(pidl: LPCITEMIDLIST) -> *mut u16 {
    let pdata = _dbg_ILGetDataPointer(pidl);
    if pdata.is_null() {
        return core::ptr::null_mut();
    }
    if _dbg_ILGetFSType(pidl) & PT_FS_UNICODE_FLAG != 0 {
        return core::ptr::addr_of_mut!((*pdata).u.file.szNames) as *mut u16;
    }
    match (*pdata).type_ {
        PT_GUID | PT_SHELLEXT | PT_YAGUID => core::ptr::null_mut(),
        PT_DRIVE | PT_DRIVE1 | PT_DRIVE2 | PT_DRIVE3 => core::ptr::null_mut(),
        PT_FOLDER | PT_FOLDER1 | PT_VALUE | PT_IESPECIAL1 | PT_IESPECIAL2 => core::ptr::null_mut(),
        PT_WORKGRP | PT_COMP | PT_NETWORK | PT_NETPROVIDER | PT_SHARE => core::ptr::null_mut(),
        _ => core::ptr::null_mut(),
    }
}
unsafe extern "C" fn _dbg_ILGetSTextPointer(pidl: LPCITEMIDLIST) -> *mut i8 {
    let pdata: LPPIDLDATA = _dbg_ILGetDataPointer(pidl);
    if pdata.is_null() {
        return core::ptr::null_mut();
    }
    let pdata_ref = &*pdata;
    match pdata_ref.type_ {
        PT_FOLDER
        | PT_VALUE
        | PT_IESPECIAL1
        | PT_IESPECIAL2 => {
            let names_ptr =
                core::ptr::addr_of!(pdata_ref.u.file.szNames) as *const u8 as *const i8;
            let len = strlen(names_ptr) as usize;
            let offset = len + 1 as usize;
            names_ptr.add(offset) as *mut i8
        }
        PT_WORKGRP => {
            let names_ptr =
                core::ptr::addr_of!(pdata_ref.u.network.szNames) as *const u8 as *const i8;
            let len = strlen(names_ptr) as usize;
            let offset = len + 1 as usize;
            names_ptr.add(offset) as *mut i8
        }
        _ => core::ptr::null_mut(),
    }
}
unsafe extern "C" fn _dbg_ILGetSTextPointerW(pidl: LPCITEMIDLIST) -> *mut u16 {
    let pdata = _dbg_ILGetDataPointer(pidl);
    if !pdata.is_null() {
        match (*pdata).type_ {
            PT_FOLDER | PT_VALUE | PT_IESPECIAL1 | PT_IESPECIAL2 | PT_WORKGRP => core::ptr::null_mut(),
            PT_VALUEW => {
                let base = &raw const (*pdata).u.file.szNames[0];
                let len = wcslen((base).cast());
                base.add(len).add(1) as *mut u16
            },
            _ => core::ptr::null_mut()
        }
    } else {
        core::ptr::null_mut()
    }
}
unsafe extern "C" fn _dbg_ILGetGUIDPointer(pidl: LPCITEMIDLIST) -> *mut IID {
    let pdata = _ILGetDataPointer(pidl);
    if !pdata.is_null() {
        match (*pdata).type_ {
            PT_SHELLEXT | PT_GUID | PT_YAGUID => {
                return core::ptr::addr_of_mut!((*pdata).u.guid.guid);
            }
            _ => {}
        }
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn _dbg_ILSimpleGetText(pidl: LPCITEMIDLIST, szOut: LPSTR, uOutSize: u32) {
    if pidl.is_null() {
        return;
    }

    if !szOut.is_null() {
        *szOut = 0;
    }

    if _dbg_ILIsDesktop(pidl) != 0 {
        // desktop
        if !szOut.is_null() {
            lstrcpynA(szOut, b"Desktop\0".as_ptr() as *const i8, (uOutSize) as i32);
        }
    } else {
        let szSrc = _dbg_ILGetTextPointer(pidl);
        if !szSrc.is_null() {
            // filesystem
            if !szOut.is_null() {
                lstrcpynA(szOut, szSrc, (uOutSize) as i32);
            }
        } else {
            let szSrcW = _dbg_ILGetTextPointerW(pidl);
            if !szSrcW.is_null() {
                // unicode filesystem
                let mut tmp: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    szSrcW,
                    -1 as i32,
                    tmp.as_mut_ptr(),
                    (MAX_PATH) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if !szOut.is_null() {
                    lstrcpynA(szOut, tmp.as_ptr(), (uOutSize) as i32);
                }
            } else {
                let riid = _dbg_ILGetGUIDPointer(pidl);
                if !riid.is_null() {
                    if !szOut.is_null() {
                        sprintf(
                            szOut,
                            c"{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}".as_ptr(),
                            (*riid).Data1 as u32,
                            (*riid).Data2 as u32,
                            (*riid).Data3 as u32,
                            (*riid).Data4[0] as u32,
                            (*riid).Data4[1] as u32,
                            (*riid).Data4[2] as u32,
                            (*riid).Data4[3] as u32,
                            (*riid).Data4[4] as u32,
                            (*riid).Data4[5] as u32,
                            (*riid).Data4[6] as u32,
                            (*riid).Data4[7] as u32,
                        );
                    }
                }
            }
        }
    }
}
unsafe extern "C" fn pdump_impl(pidl: LPCITEMIDLIST) {
    let mut pidltemp: LPCITEMIDLIST = pidl;

    if pidltemp.is_null() {
    } else {
        if (*pidltemp).mkid.cb != 0 {
            loop {
                if _ILIsUnicode(pidltemp) != 0 {
                    let mut dwAttrib: u32 = 0;
                    let pData: LPPIDLDATA = _dbg_ILGetDataPointer(pidltemp);
                    let type_: u32 = if !pData.is_null() { ((*pData).type_) as u32 } else { 0 };
                    let szLongName: *mut u16 = _dbg_ILGetTextPointerW(pidltemp);
                    let szShortName: *mut u16 = _dbg_ILGetSTextPointerW(pidltemp);
                    let mut szName: [u8; MAX_PATH] = [0u8; MAX_PATH];

                    _dbg_ILSimpleGetText(pidltemp, szName.as_mut_ptr() as *mut i8, (MAX_PATH) as u32);
                    if _dbg_ILGetFSType(pidltemp) != 0 {
                        dwAttrib = ((*pData).u.file.uFileAttribs) as u32;
                    }
                } else {
                    let mut dwAttrib: u32 = 0;
                    let pData: LPPIDLDATA = _dbg_ILGetDataPointer(pidltemp);
                    let type_: u32 = if !pData.is_null() { ((*pData).type_) as u32 } else { 0 };
                    let szLongName: LPSTR = _dbg_ILGetTextPointer(pidltemp);
                    let szShortName: LPSTR = _dbg_ILGetSTextPointer(pidltemp);
                    let mut szName: [u8; MAX_PATH] = [0u8; MAX_PATH];

                    _dbg_ILSimpleGetText(pidltemp, szName.as_mut_ptr() as *mut i8, (MAX_PATH) as u32);
                    if _dbg_ILGetFSType(pidltemp) != 0 {
                        dwAttrib = ((*pData).u.file.uFileAttribs) as u32;
                    }
                }

                pidltemp = _dbg_ILGetNext(pidltemp);
                if pidltemp.is_null() || (*pidltemp).mkid.cb == 0 {
                    break;
                }
            }
        } else {
        }
        pcheck(pidl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn pdump(pidl: LPCITEMIDLIST) {
    if !TRACE_ON!(pidl) {
        return;
    }
    pdump_impl(pidl);
}

#[no_mangle]
pub unsafe extern "C" fn pdump_always(pidl: LPCITEMIDLIST) {
    pdump_impl(pidl);
}
unsafe extern "C" fn dump_pidl_hex(pidl: LPCITEMIDLIST) {
    let p = pidl as *const u8;
    let max_bytes: i32 = 0x80 as i32;
    let max_line: usize = 0x10 as usize;
    let mut szHex = [0u8; (0x10 * 3 + 1) as usize];
    let mut szAscii = [0u8; (0x10 + 1) as usize];
    let mut n: i32 = (*pidl).mkid.cb as i32;
    if n > max_bytes {
        n = max_bytes;
    }
    let mut i: i32 = 0 as i32;
    while i < n {
        let byte = *p.add(i as usize);
        sprintf(
            szHex.as_mut_ptr().add(((i as usize % max_line) * (3 as usize))) as *mut i8,
            b"%02X \0".as_ptr() as *const i8,
            byte as UINT,
        );
        szAscii[i as usize % max_line] = if isprint((byte) as i32) != 0 {
            byte
        } else {
            b'.'
        };
        if i != (n - 1 as i32) && (i as usize % max_line) != (max_line - 1 as usize) {
            i += 1;
            continue;
        }
        szAscii[((i as usize % max_line) + (1 as usize))] = 0 as u8;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pcheck(pidl: LPCITEMIDLIST) -> i32 {
    let mut pidltemp: LPCITEMIDLIST = pidl;

    while !pidltemp.is_null() && (*pidltemp).mkid.cb != 0 {
        let pidlData: LPPIDLDATA = _dbg_ILGetDataPointer(pidltemp);

        if !pidlData.is_null() {
            let type_val = (*pidlData).type_;
            match type_val {
                PT_CPLAPPLET
                | PT_GUID
                | PT_SHELLEXT
                | PT_DRIVE
                | PT_DRIVE1
                | PT_DRIVE2
                | PT_DRIVE3
                | PT_FOLDER
                | PT_VALUE
                | PT_VALUEW
                | PT_FOLDER1
                | PT_WORKGRP
                | PT_COMP
                | PT_NETPROVIDER
                | PT_NETWORK
                | PT_IESPECIAL1
                | PT_YAGUID
                | PT_IESPECIAL2
                | PT_SHARE => {}
                _ => {
                    ERR!(
                        "unknown IDLIST %p [%p] size=%u type=%x\n",
                        pidl,
                        pidltemp,
                        (*pidltemp).mkid.cb as u32,
                        type_val
                    );
                    dump_pidl_hex(pidltemp);
                    return FALSE;
                }
            }
            pidltemp = _dbg_ILGetNext(pidltemp);
        } else {
            return FALSE;
        }
    }

    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn shdebugstr_guid(id: *const _GUID) -> *const c_char {
    if id.is_null() {
        return b"(null)\0".as_ptr() as *const c_char;
    }

    let mut name: *const c_char = core::ptr::null();
    let mut clsidbuf: [u8; 100] = [0u8; 100];

    let interface_desc_len = core::mem::size_of_val(&InterfaceDesc) / core::mem::size_of_val(&InterfaceDesc[0]);
    for i in 0..interface_desc_len {
        if IsEqualIID(InterfaceDesc[i].riid, id) {
            name = InterfaceDesc[i].name as *const c_char;
            break;
        }
    }

    if name.is_null() {
        if HCR_GetClassNameA(id, clsidbuf.as_mut_ptr() as LPSTR, 100) != 0 {
            name = clsidbuf.as_ptr() as *const c_char;
        }
    }

    let format = b"\n\t{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x} (%s)\0".as_ptr() as *const c_char;
    let name_or_unknown = if name.is_null() { b"unknown\0".as_ptr() as *const c_char } else { name };

    wine_dbg_sprintf(format,
        (*id).Data1,
        (*id).Data2 as UINT,
        (*id).Data3 as UINT,
        (*id).Data4[0] as UINT,
        (*id).Data4[1] as UINT,
        (*id).Data4[2] as UINT,
        (*id).Data4[3] as UINT,
        (*id).Data4[4] as UINT,
        (*id).Data4[5] as UINT,
        (*id).Data4[6] as UINT,
        (*id).Data4[7] as UINT,
        name_or_unknown)
}
