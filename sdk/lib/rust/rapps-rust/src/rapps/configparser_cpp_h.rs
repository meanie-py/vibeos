use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_075DE2B906DB: [WCHAR; 2] = [0x000A, 0x0000];
pub static CPP_COM_WSTR_11BF324B77A7: [WCHAR; 6] = [0x0025, 0x0073, 0x002E, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_2D68F1367DCD: [WCHAR; 9] = [0x0053, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_7ECAD61ED393: [WCHAR; 5] = [0x002E, 0x0078, 0x0038, 0x0036, 0x0000];
pub static CPP_COM_WSTR_828FD8C0C0B5: [WCHAR; 12] = [0x0053, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E, 0x0078, 0x0038, 0x0036, 0x0000];
pub static CPP_COM_WSTR_AF6C2F3E276C: [WCHAR; 8] = [0x0053, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_E17C172CF886: [WCHAR; 9] = [0x0025, 0x0073, 0x002E, 0x0025, 0x0073, 0x002E, 0x0025, 0x0073, 0x0000];

use core::ffi::c_void;










#[no_mangle]
pub unsafe extern "C" fn ReadIniValue(File: LPCWSTR, Section: LPCWSTR, Name: LPCWSTR, Output: *mut CStringW) -> HRESULT {
    let mut len: DWORD = 256;
    loop {
        let ret: DWORD = GetPrivateProfileString(Section, Name, CPP_COM_WSTR_075DE2B906DB.as_ptr(), (*Output).GetBuffer(len as i32), len, File);
        if ret + 1 != len {
            (*Output).ReleaseBufferSetLength(ret as i32);
            return if ret != 0 && (*Output).GetAt((0) as i32) != 0x000Au16 { ret as HRESULT } else { HRESULT_FROM_WIN32!(ERROR_NOT_FOUND) };
        }
        len *= 2;
    }
}

impl CConfigParser {
    pub unsafe fn CacheINI(&mut self) {
        let this_: *mut Self = self;
        // Cache section names
        if g_Names.ArchSpecific.Locale.IsEmpty() {
            let mut szLocaleID = CStringW::new();
            const cchLocaleSize: i32 = 5;

            GetLocaleInfoW(GetUserDefaultLCID(), LOCALE_ILANGUAGE, szLocaleID.GetBuffer(cchLocaleSize), cchLocaleSize);
            szLocaleID.ReleaseBuffer();
            let mut INISectionLocale = CStringW::from_ptr(CPP_COM_WSTR_2D68F1367DCD.as_ptr());
            INISectionLocale += szLocaleID.GetString();

            {
                let mut tmp = CStringW::from_ptr(INISectionLocale.GetString());
                tmp += CPP_COM_WSTR_7ECAD61ED393.as_ptr();
                g_Names.ArchSpecific.Locale = tmp;
            }
            g_Names.ArchNeutral.Locale = CStringW::from_ptr(INISectionLocale.GetString());

            // turn "Section.0c0a" into "Section.0a", keeping just the neutral lang part
            if szLocaleID.GetLength() >= 2 {
                let mut tmp1 = CStringW::from_ptr(CPP_COM_WSTR_2D68F1367DCD.as_ptr());
                tmp1 += szLocaleID.Mid(szLocaleID.GetLength() - 2).GetString();
                tmp1 += CPP_COM_WSTR_7ECAD61ED393.as_ptr();
                g_Names.ArchSpecific.LocaleNeutral = tmp1;

                let mut tmp2 = CStringW::from_ptr(CPP_COM_WSTR_2D68F1367DCD.as_ptr());
                tmp2 += szLocaleID.Mid(szLocaleID.GetLength() - 2).GetString();
                g_Names.ArchNeutral.LocaleNeutral = tmp2;
            }

            g_Names.ArchSpecific.Section = CStringW::from_ptr(CPP_COM_WSTR_828FD8C0C0B5.as_ptr());
            g_Names.ArchNeutral.Section = CStringW::from_ptr(CPP_COM_WSTR_AF6C2F3E276C.as_ptr());
        }

        // Use a shared buffer so that we don't have to re-allocate it every time
        let mut Buffer = CStringW::new();

        (*this_).ReadSection(&mut Buffer, &g_Names.ArchSpecific.Locale, (TRUE) as BOOL);
        if !g_Names.ArchSpecific.LocaleNeutral.IsEmpty() {
            (*this_).ReadSection(&mut Buffer, &g_Names.ArchSpecific.LocaleNeutral, (TRUE) as BOOL);
        }
        (*this_).ReadSection(&mut Buffer, &g_Names.ArchSpecific.Section, (TRUE) as BOOL);

        (*this_).ReadSection(&mut Buffer, &g_Names.ArchNeutral.Locale, (FALSE) as BOOL);
        if !g_Names.ArchNeutral.LocaleNeutral.IsEmpty() {
            (*this_).ReadSection(&mut Buffer, &g_Names.ArchNeutral.LocaleNeutral, (FALSE) as BOOL);
        }
        (*this_).ReadSection(&mut Buffer, &g_Names.ArchNeutral.Section, (FALSE) as BOOL);
    }
    pub unsafe fn ReadSection(&mut self, Buffer: &mut CStringW, Section: &CStringW, isArch: BOOL) {
        let this_: *mut Self = self;
        let mut len: DWORD = 512;
        let mut result: DWORD = 0;

        loop {
            len *= 2;

            result = GetPrivateProfileSectionW(Section.GetString(), Buffer.GetBuffer(len as i32), len, (*this_).szConfigPath.GetString());
            Buffer.ReleaseBufferSetLength(result as i32);
            if !(result == len - 2) {
                break;
            }
        }

        len = 0;
        while len < result {
            // Explicitly use the null terminator!
            let mut tmp: CStringW = CStringW::from_ptr(Buffer.GetBuffer(0).add(len as usize));
            if tmp.GetLength() > 0 {
                len += (tmp.GetLength() + 1) as DWORD;

                let idx: i32 = tmp.Find('=' as WCHAR, 0);
                if idx >= 0 {
                    let mut key: CStringW = tmp.Left(idx);

                    // Is this key already present from a more specific translation?
                    if (*this_).m_Keys.FindKey(&key) >= 0 {
                        continue;
                    }

                    let mut value: CStringW = tmp.Mid(idx + 1);
                    (*this_).m_Keys.Add(key, value);
                } else {
                    DPRINT1!("ERROR: invalid key/value pair: '%S'\n", tmp.GetString());
                }
            } else {
                break;
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self, FilePath: &CStringW) {
        let this_: *mut Self = self;
        (*this_).szConfigPath = CStringW::from_ptr(FilePath.GetString());
        {
            (*this_).CacheINI();
        }
    }
    pub unsafe fn GetString(&mut self, KeyName: &CStringW, ResultString: &mut CStringW) -> BOOL {
        let this_: *mut Self = self;
        let nIndex = (*this_).m_Keys.FindKey(KeyName);
        if nIndex >= 0 {
            *ResultString = (*this_).m_Keys.GetValueAt(nIndex);
            return (TRUE) as BOOL;
        }

        ResultString.Empty();
        return (FALSE) as BOOL;
    }
    pub unsafe fn GetInt(&mut self, KeyName: &CStringW, iResult: &mut INT) -> BOOL {
        let this_: *mut Self = self;
        let mut Buffer = CStringW::new();

        *iResult = 0;

        // grab the text version of our entry
        if (*this_).GetString(KeyName, &mut Buffer) == 0 {
            return (FALSE) as BOOL;
        }

        if Buffer.IsEmpty() {
            return (FALSE) as BOOL;
        }

        // convert it to an actual integer
        *iResult = StrToIntW(Buffer.GetString());

        // we only care about values > 0
        return (*iResult > 0) as BOOL;
    }
    pub unsafe fn GetSectionString(&mut self, Section: LPCWSTR, Name: LPCWSTR, Result: &mut CStringW) -> UINT {
        let this_: *mut Self = self;
        let mut hr: HRESULT; // Return value; length of ini string or 0 on failure.
        let mut SecBuf = CStringW::new();
        let mut FullLoc: [WCHAR; 5] = [0; 5];
        let mut NeutralLoc: *mut WCHAR = FullLoc.as_mut_ptr().add(2);
        GetLocaleInfoW(GetUserDefaultLCID(), LOCALE_ILANGUAGE, FullLoc.as_mut_ptr(), _countof!(FullLoc) as i32);

        atl_format!(SecBuf, CPP_COM_WSTR_E17C172CF886.as_ptr(), Section, FullLoc.as_ptr(), CurrentArchitectureW.as_ptr());
        hr = ReadIniValue((*this_).szConfigPath.GetString(), SecBuf.GetString(), Name, Result);
        if hr > 0 {
            return hr as UINT;
        }

        if *NeutralLoc != 0 {
            atl_format!(SecBuf, CPP_COM_WSTR_E17C172CF886.as_ptr(), Section, NeutralLoc as *const WCHAR, CurrentArchitectureW.as_ptr());
            hr = ReadIniValue((*this_).szConfigPath.GetString(), SecBuf.GetString(), Name, Result);
            if hr > 0 {
                return hr as UINT;
            }
        }

        atl_format!(SecBuf, CPP_COM_WSTR_11BF324B77A7.as_ptr(), Section, CurrentArchitectureW.as_ptr());
        hr = ReadIniValue((*this_).szConfigPath.GetString(), SecBuf.GetString(), Name, Result);
        if hr > 0 {
            return hr as UINT;
        }

        atl_format!(SecBuf, CPP_COM_WSTR_11BF324B77A7.as_ptr(), Section, FullLoc.as_ptr());
        hr = ReadIniValue((*this_).szConfigPath.GetString(), SecBuf.GetString(), Name, Result);
        if hr > 0 {
            return hr as UINT;
        }

        if *NeutralLoc != 0 {
            atl_format!(SecBuf, CPP_COM_WSTR_11BF324B77A7.as_ptr(), Section, NeutralLoc as *const WCHAR);
            hr = ReadIniValue((*this_).szConfigPath.GetString(), SecBuf.GetString(), Name, Result);
            if hr > 0 {
                return hr as UINT;
            }
        }

        hr = ReadIniValue((*this_).szConfigPath.GetString(), Section, Name, Result);
        if hr > 0 {
            return hr as UINT;
        }
        return 0;
    }
}

impl CLocaleSections {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CSectionNames {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

// misc.h: `#define CurrentArchitecture L"x86"` etc. The island's byte-string constant of the same name is
// a narrow mistranslation of that wide macro, and `%s` in the wide printf wants WCHARs.
#[cfg(target_arch = "x86")]
static CurrentArchitectureW: [WCHAR; 4] = [0x78, 0x38, 0x36, 0];
#[cfg(target_arch = "x86_64")]
static CurrentArchitectureW: [WCHAR; 6] = [0x61, 0x6d, 0x64, 0x36, 0x34, 0];
#[cfg(target_arch = "aarch64")]
static CurrentArchitectureW: [WCHAR; 6] = [0x61, 0x72, 0x6d, 0x36, 0x34, 0];
