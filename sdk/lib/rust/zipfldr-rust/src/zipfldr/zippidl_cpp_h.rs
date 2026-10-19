use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _ILCreateZipItem(Type: ZipPidlType, lpString: PCWSTR, info: &mut unz_file_info64) -> LPITEMIDLIST {
    let mut cbData: usize = core::mem::size_of::<ZipPidlEntry>() + wcslen(lpString) * core::mem::size_of::<WCHAR>();
    if cbData > MAXWORD as usize {
        return core::ptr::null_mut();
    }

    let mut pidl: *mut ZipPidlEntry = SHAlloc((cbData + core::mem::size_of::<WORD>()) as SIZE_T) as *mut ZipPidlEntry;
    if pidl.is_null() {
        return core::ptr::null_mut();
    }

    ZeroMemory!(pidl as LPVOID, cbData + core::mem::size_of::<WORD>());

    (*pidl).cb = cbData as WORD;
    (*pidl).MagicType = b'z' as BYTE;
    (*pidl).ZipType = Type;

    if Type != ZIP_PIDL_DIRECTORY {
        (*pidl).CompressedSize = info.compressed_size;
        (*pidl).UncompressedSize = info.uncompressed_size;
        (*pidl).DosDate = info.dosDate as ULONG;
        (*pidl).Password = (info.flag & MINIZIP_PASSWORD_FLAG) as BOOLEAN;
    }

    wcscpy(core::ptr::addr_of_mut!((*pidl).Name).cast(), lpString);
    *((pidl as PBYTE).add(cbData) as PWORD) = 0 as WORD; // The end of an ITEMIDLIST

    return pidl as LPITEMIDLIST;
}
#[no_mangle]
pub unsafe extern "C" fn _ZipFromIL(pidl: LPCITEMIDLIST) -> *const ZipPidlEntry {
    let zipPidl: *const ZipPidlEntry = pidl as *const ZipPidlEntry;
    if (*zipPidl).MagicType == 'z' as BYTE {
        return zipPidl;
    }
    return core::ptr::null();
}
