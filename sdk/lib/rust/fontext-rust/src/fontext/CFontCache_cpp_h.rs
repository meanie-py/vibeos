use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_FontCache: *mut CFontCache = core::ptr::null_mut();






















impl CFontInfo {
    pub unsafe fn ReadAttrs(&mut self) {
        let this_: *mut Self = self;
        let File: CStringW = (*g_FontCache).Filename(this_, true);

        (*this_).m_AttrsRead = true;

        let mut findFileData: WIN32_FIND_DATAW = core::mem::zeroed();
        let hFile: HANDLE = FindFirstFileW(File.GetString(), core::ptr::addr_of_mut!(findFileData));
        if hFile != INVALID_HANDLE_VALUE {

            // File write time
            FileTimeToLocalFileTime(
                core::ptr::addr_of!(findFileData.ftLastWriteTime),
                core::ptr::addr_of_mut!((*this_).m_FileWriteTime),
            );

            // File size
            (*this_).m_FileSize.__bindgen_anon_1.HighPart = findFileData.nFileSizeHigh as _;
            (*this_).m_FileSize.__bindgen_anon_1.LowPart = findFileData.nFileSizeLow as _;

            (*this_).m_dwFileAttributes = findFileData.dwFileAttributes;
            FindClose(hFile);
        }
    }
    pub unsafe fn __ctor_0(&mut self, name: PCWSTR, value: PCWSTR) {
        let this_: *mut Self = self;
        (*this_).m_Name = CStringW::from_ptr(name);
        (*this_).m_File = CStringW::from_ptr(value);
        (*this_).m_bMarkDeleted = (FALSE as i32);
        (*this_).m_FileRead = false;
        (*this_).m_AttrsRead = false;
        (*this_).m_FileWriteTime = core::mem::zeroed();
        (*this_).m_dwFileAttributes = 0;
        {
            (*this_).m_FileSize.QuadPart = 0;
        }
    }
    pub unsafe fn Name(&mut self) -> *const CStringW {
        let this_: *mut Self = self;
        return &(*this_).m_Name as *const CStringW;
    }
    pub unsafe fn Valid(&mut self) -> bool {
        let this_: *mut Self = self;
        return !(*this_).m_Name.IsEmpty();
    }
    pub unsafe fn File(&mut self) -> &'static CStringW {
        let this_: *mut Self = self;
        if !(*this_).m_FileRead {
            if (*this_).Valid() && (*this_).m_File.IsEmpty() {
                // Read the filename stored in the registry.
                // This can be either a filename or a full path
                let mut key = CRegKey__make_0();
                if key.Open(FONT_HIVE, FONT_KEY.as_ptr(), KEY_READ) == (ERROR_SUCCESS as i32) {
                    let mut Value = CStringW::new();
                    let mut dwAllocated: DWORD = MAX_PATH;
                    let mut Status: LSTATUS = 0;
                    loop {
                        let mut dwSize: DWORD = dwAllocated;
                        let Buffer: PWSTR = Value.GetBuffer(dwSize as i32);
                        Status = key.QueryStringValue(
                            (*this_).m_Name.GetString(),
                            Buffer,
                            core::ptr::addr_of_mut!(dwSize),
                        );
                        Value.ReleaseBufferSetLength(dwSize as i32);
                        if Status == (ERROR_SUCCESS as LSTATUS) {
                            // Ensure we do not re-use the same string object, by passing it a PCWSTR
                            (*this_).m_File = CStringW::from_ptr(Value.GetString());
                            break;
                        }
                        dwAllocated += 128;
                        if !(Status == (ERROR_MORE_DATA as LSTATUS)) {
                            break;
                        }
                    }
                }
            }
            (*this_).m_FileRead = true;
        }
        &(*this_).m_File
    }
    pub unsafe fn FileSize(&mut self) -> *const LARGE_INTEGER {
        let this_: *mut Self = self;
        if !(*this_).m_AttrsRead {
            (*this_).ReadAttrs();
        }

        return core::ptr::addr_of!((*this_).m_FileSize);
    }
    pub unsafe fn FileWriteTime(&mut self) -> *const FILETIME {
        let this_: *mut Self = self;
        if !(*this_).m_AttrsRead {
            (*this_).ReadAttrs();
        }

        return core::ptr::addr_of!((*this_).m_FileWriteTime);
    }
    pub unsafe fn FileAttributes(&mut self) -> DWORD {
        let this_: *mut Self = self;
        if !(*this_).m_AttrsRead {
            (*this_).ReadAttrs();
        }

        return (*this_).m_dwFileAttributes;
    }
}

impl CFontCache {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
    pub unsafe fn Insert(&mut self, fonts: &mut CAtlList<CFontInfo>, KeyName: &CStringW, Value: PCWSTR) {
        let this_: *mut Self = self;
        let mut newInfo: CFontInfo = CFontInfo__make_0(KeyName.GetString(), Value);

        let mut strFontFile: CStringW = (*g_FontCache).GetFontFilePath(newInfo.File().GetString());
        if PathFileExistsW(strFontFile.GetString()) == 0 {
            return;
        }

        let mut it: POSITION = fonts.GetHeadPosition();
        while !it.is_null() {
            let mut lastit: POSITION = it;
            let info: &mut CFontInfo = &mut *fonts.GetNext(&mut it);
            if (*info.Name()).CompareNoCase(KeyName.GetString()) >= 0 {
                fonts.InsertBefore(lastit, newInfo);
                return;
            }
        }

        fonts.AddTail(newInfo);
    }
    pub unsafe fn Read(&mut self) {
        let this_: *mut Self = self;
        let mut key = CRegKey__make_0();
        let mut error: LSTATUS = key.Open(FONT_HIVE, FONT_KEY.as_ptr(), KEY_READ);
        if error != (ERROR_SUCCESS as LSTATUS) {
            return;
        }

        let mut Name: CComHeapPtr<WCHAR> = CComHeapPtr::new();
        let mut Value: CComHeapPtr<WCHAR> = CComHeapPtr::new();
        let mut cchName: DWORD = MAX_PATH;
        let mut cchValue: DWORD = MAX_PATH;
        if Name.Allocate(cchName as usize) == 0 || Value.Allocate(cchValue as usize) == 0 {
            return;
        }

        // Enumerate all registered font names
        let mut fonts: CAtlList<CFontInfo> = CAtlList::<CFontInfo>::new();
        let mut iItem: DWORD = 0;
        'outer: loop {
            'body: {
                let mut cchName2: DWORD = cchName;
                let mut cbValue: DWORD = cchValue * core::mem::size_of::<WCHAR>() as DWORD;
                error = RegEnumValueW(
                    (key).op_to_HKEY(),
                    iItem,
                    Name.as_ptr(),
                    core::ptr::addr_of_mut!(cchName2),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    Value.as_ptr() as PBYTE,
                    core::ptr::addr_of_mut!(cbValue),
                );
                if error == (ERROR_MORE_DATA as LSTATUS) {
                    cchName += 128;
                    cchValue += 128;
                    if Name.Reallocate(cchName as usize) == 0 || Value.Reallocate(cchValue as usize) == 0 {
                        break 'outer;
                    }
                    iItem -= 1;
                    break 'body;
                }

                if error != (ERROR_SUCCESS as LSTATUS) {
                    break 'outer;
                }

                if *Name.as_ptr().add(0 as usize) != 0
                    && *Value.as_ptr().add(0 as usize) != 0
                {
                    // Insert will create an ordered list (sorted)
                    (*this_).Insert(&mut fonts, &CStringW::from_ptr(Name.as_ptr()), Value.as_ptr() as PCWSTR);
                }
            }
            iItem = iItem.wrapping_add(1);
        }

        // Move the fonts from a list to an array (for easy indexing)
        (*this_).m_Fonts.RemoveAll();
        // C++ copies each element (`m_Fonts.Add(fonts.GetNext(it))`) and the local list then
        // destroys the originals: moving them out is the same, and never leaves two owners.
        while let Some(info) = fonts.RemoveHead() {
            (*this_).m_Fonts.Add(info);
        }
    }
    pub unsafe fn SetFontDir(&mut self, Path: LPCWSTR) {
        let this_: *mut Self = self;
        if (*this_).m_FontFolderPath.IsEmpty() {
            (*this_).m_FontFolderPath = CStringW::from_ptr(Path);
        }
    }
    pub unsafe fn GetFontFilePath(&mut self, Path: PCWSTR) -> CStringW {
        let this_: *mut Self = self;
        if ((PathIsRelativeW(Path)) != 0) {
            let mut r = CStringW::from_ptr((*this_).m_FontFolderPath.GetString());
            r += Path;
            return r;
        }
        return CStringW::from_ptr(Path);
    }
    pub unsafe fn Size(&mut self) -> size_t {
        let this_: *mut Self = self;
        if (*this_).m_Fonts.GetSize() == 0 {
            (*this_).Read();
        }

        return (*this_).m_Fonts.GetSize() as size_t;
    }
    pub unsafe fn Name(&mut self, Index: size_t) -> CStringW {
        let this_: *mut Self = self;
        if (*this_).m_Fonts.GetSize() == 0 {
            (*this_).Read();
        }

        if (Index as INT) >= (*this_).m_Fonts.GetSize() {
            return CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr());
        }

        return (*(*(*this_).m_Fonts.GetAt(Index as i32)).Name()).clone();
    }
    pub unsafe fn File(&mut self, Index: size_t) -> CStringW {
        let this_: *mut Self = self;
        if (*this_).m_Fonts.GetSize() == 0 {
            (*this_).Read();
        }

        if (Index as INT) >= (*this_).m_Fonts.GetSize() {
            return CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr());
        }

        return (*(*this_).m_Fonts.GetAt(Index as i32)).File().clone();
    }
    pub unsafe fn Find(&mut self, fontEntry: *const FontPidlEntry) -> *mut CFontInfo {
        let this_: *mut Self = self;
        if (*this_).m_Fonts.GetSize() == 0 {
            (*this_).Read();
        }

        let mut i: i32 = 0;
        while i < (*this_).m_Fonts.GetSize() {
            if (*(*(*this_).m_Fonts.GetAt(i)).Name()).CompareNoCase((*fontEntry.cast_mut()).Name__1()) == 0 {
                return (*this_).m_Fonts.GetAt(i);
            }
            i += 1;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn IsMarkDeleted(&mut self, Index: usize) -> BOOL {
        let this_: *mut Self = self;
        if (Index as i32) >= (*this_).m_Fonts.GetSize() {
            return (FALSE) as BOOL;
        }
        return (*(*this_).m_Fonts.GetAt(Index as i32)).IsMarkDeleted();
    }
    pub unsafe fn MarkDeleted(&mut self, fontEntry: *const FontPidlEntry) {
        let this_: *mut Self = self;
        let mut i: i32 = 0;
        while i < (*this_).m_Fonts.GetSize() {
            if (*(*(*this_).m_Fonts.GetAt(i)).Name()).CompareNoCase((*fontEntry.cast_mut()).Name__1()) == 0 {
                (*(*this_).m_Fonts.GetAt(i)).MarkDeleted();
                break;
            }
            i += 1;
        }
    }
    pub unsafe fn Filename(&mut self, info: *mut CFontInfo, alwaysFullPath: bool) -> CStringW {
        let this_: *mut Self = self;
        let mut File = CStringW::new();
        if !info.is_null() {
            File = CStringW::from_ptr((*info).File().GetString());

            if !File.IsEmpty() && alwaysFullPath {
                // Ensure this is a full path
                if PathIsRelativeW(File.GetString()) != 0 {
                    let mut Tmp = CStringW::new();
                    Tmp += (*this_).m_FontFolderPath.GetString();
                    Tmp += File.GetString();
                    File = Tmp;
                }
            }
        }

        return File;
    }
}
