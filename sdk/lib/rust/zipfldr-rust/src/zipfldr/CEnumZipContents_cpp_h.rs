use crate::prelude::*;
use core::ffi::c_void;








unsafe extern "C" fn IncludeInEnumIDList__0(contf: SHCONTF, att: SFGAOF) -> bool {
    let both: SHCONTF = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;
    let superbits: SFGAOF = SFGAO_HIDDEN | SFGAO_READONLY | SFGAO_SYSTEM;
    let isfile: bool = (att & (SFGAO_STREAM | SFGAO_FOLDER)) != SFGAO_FOLDER;
    if (contf & both) != both && (contf & SHCONTF_STORAGE) == 0 {
        if isfile && (contf & SHCONTF_FOLDERS) != 0 {
            return false;
        }
        if (att & SFGAO_FOLDER) != 0 && (contf & SHCONTF_NONFOLDERS) != 0 {
            return false;
        }
    }
    if (att & SFGAO_HIDDEN) != 0 && (contf & (SHCONTF_INCLUDEHIDDEN | SHCONTF_STORAGE)) == 0 {
        return false;
    }
    if (att & superbits) > SFGAO_HIDDEN && (contf & (SHCONTF_INCLUDESUPERHIDDEN | SHCONTF_STORAGE)) == 0 {
        return false;
    }
    true
}

unsafe extern "C" fn IncludeInEnumIDList__1(contf: SHCONTF, dir: bool, zfi: *mut unz_file_info64) -> bool {
    let att: SFGAOF = if dir { SFGAO_FOLDER } else { SFGAO_STREAM };
    // TODO: SFGAO_HIDDEN,READONLY,SYSTEM from zfi.external_fa? (Use MapFSToSFAttributes)
    return IncludeInEnumIDList__0(contf, att);
}
#[no_mangle]
pub unsafe extern "C" fn _CEnumZipContents_CreateInstance(zip: *mut IZip, flags: DWORD, prefix: PCWSTR, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    return ShellObjectCreatorInit!(CEnumZipContents, [zip, flags, prefix], riid, ppvOut);
}

impl CEnumZipContents {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).mEnumerator.__ctor_0();
        (*this_).dwFlags = 0;
        {
        }
    }
    pub unsafe fn Initialize(&mut self, zip: *mut IZip, flags: DWORD, prefix: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).dwFlags = flags;
        (*this_).m_Prefix = CStringW::from_ptr(prefix);
        if (*this_).mEnumerator.Initialize(zip) != 0 {
            return S_OK;
        }
        return E_FAIL;
    }
    pub unsafe fn Next(&mut self, celt: ULONG, rgelt: *mut LPITEMIDLIST, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if rgelt.is_null() || (pceltFetched.is_null() && celt != 1) {
            return E_POINTER;
        }

        let mut hr: HRESULT = S_OK;
        let mut fetched: ULONG = 0;
        let mut item: LPITEMIDLIST = core::ptr::null_mut();
        let mut name = CStringW::new();
        let mut dir: bool = false;
        let mut info: unz_file_info64 = core::mem::zeroed();

        while fetched < celt {
            if (((*this_).mEnumerator.NextUnique((&(*this_).m_Prefix).GetString(), &mut name, &mut dir, &mut info)) != 0) {
                if !IncludeInEnumIDList__1((*this_).dwFlags, dir, &mut info) {
                    continue;
                }

                item = _ILCreateZipItem(
                    if dir { ZIP_PIDL_DIRECTORY } else { ZIP_PIDL_FILE },
                    (&name).GetString(),
                    &mut info,
                );
                if item.is_null() {
                    hr = if fetched != 0 { S_FALSE } else { E_OUTOFMEMORY };
                    break;
                }
                *rgelt.offset(fetched as isize) = item;
                fetched += 1;
            } else {
                hr = S_FALSE;
                break;
            }
        }

        if !pceltFetched.is_null() {
            *pceltFetched = fetched;
        }
        return hr;
    }
    pub unsafe fn Skip(&mut self, mut celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut name = CStringW::new();
        let mut dir: bool = false;
        let mut info: unz_file_info64 = core::mem::zeroed();
        while celt != 0 {
            celt -= 1;
            if (*this_).mEnumerator.NextUnique(
                (&(*this_).m_Prefix).GetString(),
                &mut name,
                &mut dir,
                &mut info,
            ) == 0
            {
                return E_FAIL;
            }
        }
        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).mEnumerator.Reset() != 0 {
            return S_OK;
        }
        return E_FAIL;
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
