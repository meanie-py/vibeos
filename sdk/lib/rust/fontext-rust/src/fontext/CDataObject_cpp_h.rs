use crate::prelude::*;
use crate::fontext::CFontCache_cpp_h::g_FontCache;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----


pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    core::ptr::addr_of_mut!(__wine_dbch_fontext);
#[no_mangle]
pub unsafe extern "C" fn _CDataObject_CreateInstance(folder: PCIDLIST_ABSOLUTE, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: REFIID, ppvOut: LPVOID) -> HRESULT {
    let mut hr: HRESULT = CIDLData_CreateFromIDArray(folder, cidl, apidl, ppvOut as *mut *mut IDataObject);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    // Now that we have an IDataObject with the shell itemid list (CFSTR_SHELLIDLIST, aka HIDA) format
    // we will augment this IDataObject with the CF_HDROP format. (Full filepaths)
    // This enabled the objects for the 'copy' and drag to copy actions

    let mut data: CComHeapPtr<BYTE> = CComHeapPtr::new();

    // First we allocate room for the DROPFILES structure
    data.Allocate(core::mem::size_of::<DROPFILES>());
    let mut offset: UINT = core::mem::size_of::<DROPFILES>() as UINT;

    // Then we walk all files
    let mut n: UINT = 0;
    while n < cidl {
        let fontEntry: *const FontPidlEntry = _FontFromIL(*apidl.add(n as usize));
        if !fontEntry.is_null() {
            let mut File: CStringW = (*g_FontCache).Filename((*g_FontCache).Find(fontEntry), true);
            if !File.IsEmpty() {
                // Now append the path (+ nullterminator) to the buffer
                let len: UINT = offset + ((File.GetLength() + 1) as UINT) * (core::mem::size_of::<WCHAR>() as UINT);
                data.ReallocateBytes(len as usize);
                if data.is_null() {

                    return hr;
                }
                let dataPtr: *mut BYTE = data.as_ptr();
                StringCbCopyW(dataPtr.add(offset as usize) as STRSAFE_LPWSTR, (len - offset) as usize, File.GetString());
                offset = len;
            } else {

            }
        }
        n += 1;
    }

    // Append the final nullterminator (double null terminated list)
    data.ReallocateBytes((offset as usize) + core::mem::size_of_val(&UNICODE_NULL));
    let str: LPWSTR = data.as_ptr().add(offset as usize) as LPWSTR;
    *str = UNICODE_NULL;
    offset += core::mem::size_of_val(&UNICODE_NULL) as UINT;

    // Fill in the required fields
    let pDrop: *mut DROPFILES = data.as_ptr() as *mut DROPFILES;
    (*pDrop).fWide = 1;
    (*pDrop).pFiles = core::mem::size_of::<DROPFILES>() as DWORD;
    // Zero out the rest
    (*pDrop).pt.x = 0;
    (*pDrop).pt.y = 0;
    (*pDrop).fNC = 0;

    hr = DataObject_SetData(*(ppvOut as *mut *mut IDataObject), (CF_HDROP as u16), data.as_ptr() as LPVOID, (offset) as _);
    FAILED_UNEXPECTEDLY!(hr);

    return hr;
}
