use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    // zlib (linked static) and msvcrt; outside the island.
    fn crc32(crc: uLong, buf: *const u8, len: uInt) -> uLong;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}








impl CZipEnumerator {
    pub unsafe fn CalculateCRC32(filename: PCSTR) -> DWORD {
        ATLASSERT!(filename);
        let mut crc: DWORD = crc32(0, core::ptr::null(), 0) as DWORD;
        crc = crc32(crc as uLong, filename as *const u8, strlen(filename) as uInt) as DWORD;
        return crc;
    }
    pub unsafe fn GetUtf8Name(originalName: PCSTR, extraField: *const BYTE, extraFieldLen: DWORD) -> CStringA {
        ATLASSERT!(!originalName.is_null());
        ATLASSERT!(!extraField.is_null());

        if extraFieldLen < (EF_HEADER_SIZE as DWORD) {
            return CStringA::new(); // Failure
        }

        let mut pbField: *const BYTE = extraField;
        let pbEnd: *const BYTE = extraField.add(extraFieldLen as usize);

        while pbField.add(EF_HEADER_SIZE as usize) <= pbEnd {
            // Beware of alignment exception
            let mut fieldId: WORD = 0;
            let mut fieldSize: WORD = 0;
            CopyMemory!(
                core::ptr::addr_of_mut!(fieldId) as *mut core::ffi::c_void,
                pbField as *const core::ffi::c_void,
                core::mem::size_of::<WORD>(),
            );
            CopyMemory!(
                core::ptr::addr_of_mut!(fieldSize) as *mut core::ffi::c_void,
                pbField.add(2 as usize) as *const core::ffi::c_void,
                core::mem::size_of::<WORD>(),
            );

            if fieldId != (EF_UNIPATH as WORD) {
                pbField = pbField.add(EF_HEADER_SIZE as usize + fieldSize as usize); // Next field
                continue;
            }

            if fieldSize < 5
                || pbField.add(EF_HEADER_SIZE as usize + fieldSize as usize) > pbEnd
            {
                return CStringA::new();
            }

            let fieldData: *const BYTE = pbField.add(EF_HEADER_SIZE as usize);
            let version: BYTE = *fieldData.add(0 as usize);
            if version != (EF_UNIPATH_VERSION as BYTE) {
                return CStringA::new(); // Failure
            }

            let mut storedCRC: DWORD = 0;
            let calculatedCRC: DWORD = CZipEnumerator::CalculateCRC32(originalName);
            CopyMemory!(
                core::ptr::addr_of_mut!(storedCRC) as *mut core::ffi::c_void,
                fieldData.add(1 as usize) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            );
            if storedCRC != calculatedCRC {
                return CStringA::new(); // Failure
            }

            let utf8NameLen: DWORD = (fieldSize as DWORD).wrapping_sub(5 as DWORD);
            if utf8NameLen > 0 {
                return CStringA::from_ptr_len(fieldData.add(5 as usize) as LPCSTR, utf8NameLen as i32); // Success
            }

            pbField = pbField.add(EF_HEADER_SIZE as usize + fieldSize as usize); // Next field
        }

        CStringA::new() // Failure
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_First = (TRUE as i32);
        (*this_).m_nCodePage = GetZipCodePage((TRUE) as BOOL);
        {
        }
    }
    pub unsafe fn Initialize(&mut self, zip: *mut IZip) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(zip);
        (*this_).m_Zip.assign(zip);
        return (*this_).Reset();
    }
    pub unsafe fn Reset(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let uf = (*(*this_).m_Zip.Get()).getZip();
        (*this_).m_First = (TRUE as i32);
        if unzGoToFirstFile(uf) != (UNZ_OK as i32) {
            return (FALSE) as BOOL;
        }
        (*this_).m_Returned.RemoveAll();
        return (TRUE) as BOOL;
    }
    pub unsafe fn Next(&mut self, name: &mut CStringW, info: &mut unz_file_info64) -> BOOL {
        let this_: *mut Self = self;
        let mut err: INT;
        let uf = (*(*this_).m_Zip.Get()).getZip();

        if (*this_).m_First == 0 {
            err = unzGoToNextFile(uf);
            if err == UNZ_END_OF_LIST_OF_FILE {
                return (FALSE) as BOOL;
            }
        }
        (*this_).m_First = (FALSE as i32);

        err = unzGetCurrentFileInfo64(uf, core::ptr::addr_of_mut!(*info), core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0);
        if err != (UNZ_OK as INT) {
            return (FALSE) as BOOL;
        }

        let mut extra: CAtlArray<BYTE> = CAtlArray::new();
        extra.SetCount(info.size_file_extra as usize);

        let mut nameA: CStringA = CStringA::new();
        let buf: PSTR = nameA.GetBuffer(info.size_filename as i32);
        err = unzGetCurrentFileInfo64(uf, core::ptr::null_mut(), buf, (nameA.GetAllocLength()) as uLong,
            if info.size_file_extra > 0 { (extra.GetData()).cast() } else { core::ptr::null_mut() }, info.size_file_extra,
            core::ptr::null_mut(), 0);
        nameA.ReleaseBufferSetLength(info.size_filename as i32);

        if err != (UNZ_OK as INT) {
            return (FALSE) as BOOL;
        }

        let mut utf8Name: CStringA = CStringA::new();
        if info.size_file_extra > 0 {
            utf8Name = CZipEnumerator::GetUtf8Name(nameA.GetString() as PCSTR, extra.GetData(), info.size_file_extra);
        }

        if utf8Name.GetLength() > 0 {
            *name = CA2WEX::<MAX_PATH>(utf8Name.GetString() as LPCSTR, CP_UTF8);
        } else if (info.flag & MINIZIP_UTF8_FLAG) != 0 {
            *name = CA2WEX::<MAX_PATH>(nameA.GetString() as LPCSTR, CP_UTF8);
        } else {
            *name = CA2WEX::<MAX_PATH>(nameA.GetString() as LPCSTR, (*this_).m_nCodePage);
        }

        name.Replace('\\' as WCHAR, '/' as WCHAR);

        return (TRUE) as BOOL;
    }
    pub unsafe fn NextUnique(&mut self, prefix: PCWSTR, name: &mut CStringW, folder: &mut bool, info: &mut unz_file_info64) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(!prefix.is_null());
        let mut tmp = CStringW::new();
        let cchPrefix: usize = wcslen(prefix);
        while (*this_).Next(&mut tmp, info) != 0 {
            if StrCmpNIW(tmp.GetString(), prefix, cchPrefix as i32) != 0 {
                continue;
            }

            let ichSlash: INT = tmp.Find(0x002Fu16, cchPrefix as i32);
            *folder = ichSlash >= 0;
            let sub: CStringW = if *folder {
                tmp.Mid__1(cchPrefix as i32, ichSlash - cchPrefix as i32)
            } else {
                tmp.Mid(cchPrefix as i32)
            };
            name.SetString(sub.GetString(), sub.GetLength());
            tmp.SetString(name.GetString(), name.GetLength());
            tmp.MakeLower();

            let it: POSITION = (*this_).m_Returned.FindPos(&tmp);
            if !name.IsEmpty() && it.is_null() {
                (*this_).m_Returned.AddTail(CStringW::from_ptr(tmp.GetString()));
                return (TRUE) as BOOL;
            }
        }
        return (FALSE) as BOOL;
    }
}
