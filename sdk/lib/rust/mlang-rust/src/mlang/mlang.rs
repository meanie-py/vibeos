use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn TRACE(format: *const i8, ...);
    fn WARN(format: *const i8, ...);
    fn debugstr_guid(guid: *const GUID) -> *const i8;
    static IID_IUnknown: GUID;
    static IID_IClassFactory: GUID;
    fn TranslateCharsetInfo(lpSrc: *const core::ffi::c_void, lpCs: *mut CHARSETINFO, dwFlags: u32) -> i32;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn wine_dbgstr_w(str: *const u16) -> *const i8;
    fn FIXME(fmt: *const core::ffi::c_char, ...);
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn __wine_unregister_resources(hInstance: HINSTANCE) -> HRESULT;
    fn debugstr_w(wstr: *const u16) -> *const i8;
    unsafe fn map_font(hDC: HDC, dwCodePages: u32, hSrcFont: HFONT, phDestFont: *mut HFONT) -> HRESULT;
    static IID_IEnumRfc1766: GUID;
    static IID_IMultiLanguage: GUID;
    static IID_IMLangCodePages: GUID;
    static IID_IMLangFontLink: GUID;
    static IID_IMLangFontLink2: GUID;
    static IID_IMultiLanguage2: GUID;
    static IID_IMultiLanguage3: GUID;
    static IID_IMLangLineBreakConsole: GUID;
    unsafe fn EnumRfc1766_create(LangId: LANGID, ppEnum: *mut *mut IEnumRfc1766) -> HRESULT;
    unsafe fn EnumScript_create(This: *mut MLang_impl, dwFlags: DWORD, LangId: LANGID, ppEnumScript: *mut *mut IEnumScript) -> HRESULT;
    unsafe fn fill_cp_info(ml_data: *const mlang_data, index: u32, mime_cp_info: *mut MIMECPINFO) ;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(heap: *mut core::ffi::c_void, flags: u32, size: u32) -> *mut core::ffi::c_void;
    fn HeapFree(heap: *mut core::ffi::c_void, flags: u32, ptr: *mut core::ffi::c_void) -> i32;
    fn debugstr_an(s: *const i8, len: u32) -> *const i8;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn ConvertINetMultiByteToUnicode( pdwMode: *mut u32, dwEncoding: u32, pSrcStr: *const i8, pcSrcSize: *mut i32, pDstStr: *mut u16, pcDstSize: *mut i32, ) -> HRESULT;
}
extern "C" {
    fn DisableThreadLibraryCalls(hLibModule: HINSTANCE) -> BOOL;
    fn TlsAlloc() -> DWORD;
    fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    fn GetLocaleInfoA(lcid: u32, lctype: u32, lcdata: *mut i8, cchdata: i32) -> i32;
    fn LCMapStringA(locale: u32, dwmapflags: u32, lpsrc: *const i8, cchsrc: i32, lpdest: *mut i8, cchdest: i32) -> i32;
    fn IEnumRfc1766_Next(iface: *mut IEnumRfc1766, celt: u32, rgelt: *mut RFC1766INFO, pceltFetched: *mut u32) -> i32;
    fn wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn lstrlenW(s: *const u16) -> i32;
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DeleteObject(ho: HGDIOBJ) -> i32;
    fn list_remove(entry: *mut list);
    fn IClassFactory_AddRef(iface: *mut IClassFactory) -> u32;
    fn TlsSetValue(dwTlsIndex: u32, lpTlsValue: *mut core::ffi::c_void) -> i32;
    fn EnumSystemLocalesW(lpLocaleEnumProc: Option<unsafe extern "system" fn(*const u16) -> i32>, dwFlags: u32) -> i32;
    unsafe fn enum_locales_proc(lpLocaleString: *const u16) -> i32;
    fn IMultiLanguage3_ValidateCodePageEx( iface: *mut IMultiLanguage3, uiCodePage: u32, hwnd: isize, dwFlags: u32, ) -> i32;
    fn GetFontUnicodeRanges(hdc: HDC, lpgs: *mut GLYPHSET) -> u32;
    fn debugstr_wn(pszSrc: *const u16, cchSrc: i32) -> *const i8;
    fn RegCreateKeyW(hKey: HKEY, lpSubKey: *const u16, phkResult: *mut HKEY) -> LSTATUS;
    fn RegSetValueExA(hKey: HKEY, lpValueName: *const i8, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32) -> LSTATUS;
    fn RegSetValueExW(hKey: HKEY, lpValueName: *const u16, Reserved: u32, dwType: u32, lpData: *const u8, cbData: u32) -> LSTATUS;
    fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    unsafe fn GetFamilyCodePage(cp: u32, family: *mut u32) -> i32;
    fn TlsGetValue(dwTlsIndex: u32) -> *mut core::ffi::c_void;
    fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32;
    unsafe fn lcid_to_rfc1766W(lcid: u32, buf: *mut u16, len: usize) ;
    fn GetLocaleInfoW(locale: u32, lctype: u32, lplcdata: *mut u16, cchdata: i32) -> i32;
    fn HeapReAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, dwBytes: usize) -> *mut core::ffi::c_void;
    fn IMultiLanguage3_EnumRfc1766(iface: *mut IMultiLanguage3, langid: u16, ppEnum: *mut *mut IEnumRfc1766) -> i32;
    fn IEnumRfc1766_Release(This: *mut IEnumRfc1766) -> u32;
    fn GetCurrentObject(hdc: *mut c_void, uObjectType: u32) -> *mut c_void;
    fn IEnumCodePage_AddRef(iface: *mut IEnumCodePage);
    static IID_IEnumCodePage: GUID;
    type IEnumCodePage;
    fn IEnumScript_AddRef(iface: *mut IEnumScript) -> u32;
    fn IMultiLanguage3_QueryInterface( iface: *mut IMultiLanguage3, riid: *const IID, ppvObject: *mut *mut core::ffi::c_void, ) -> HRESULT;
    fn IMultiLanguage3_AddRef(iface: *mut IMultiLanguage3) -> u32;
    fn IMultiLanguage3_Release(iface: *mut IMultiLanguage3) -> u32;
    fn IMLangFontLink2_GetCharCodePages(iface: *mut IMLangFontLink2, ch_src: u16, codepages: *mut u32) -> i32;
    fn IMLangFontLink2_GetStrCodePages( iface: *mut IMLangFontLink2, src: *const u16, src_len: i32, priority_cp: u32, codepages: *mut u32, ret_len: *mut i32, ) -> HRESULT;
    fn IMLangFontLink2_CodePageToCodePages( iface: *mut IMLangFontLink2, codepage: u32, codepages: *mut u32, ) -> HRESULT;
    fn IMLangFontLink2_CodePagesToCodePage( iface: *mut IMLangFontLink2, codepages: u32, def_codepage: u32, codepage: *mut u32, ) -> i32;
    fn IMLangFontLink2_GetFontCodePages( iface: *mut IMLangFontLink2, hdc: HDC, hfont: HFONT, codepages: *mut u32, ) -> i32;
    fn IMultiLanguage3_GetFamilyCodePage( iface: *mut IMultiLanguage3, cp: u32, family_cp: *mut u32, ) -> HRESULT;
    fn IMultiLanguage3_ConvertString( iface: *mut IMultiLanguage3, mode: *mut u32, src_enc: u32, dst_enc: u32, src: *mut u8, src_size: *mut u32, dest: *mut u8, dest_size: *mut u32, ) -> HRESULT;
    fn IMultiLanguage3_ConvertStringToUnicode(iface: *mut IMultiLanguage3, mode: *mut DWORD, src_enc: DWORD, src: *mut CHAR, src_size: *mut UINT, dest: *mut WCHAR, dest_size: *mut UINT) -> HRESULT;
    fn IMultiLanguage3_ConvertStringFromUnicode( iface: *mut IMultiLanguage3, mode: *mut DWORD, encoding: DWORD, src: *mut WCHAR, src_size: *mut UINT, dest: *mut CHAR, dest_size: *mut UINT, ) -> HRESULT;
    fn IMultiLanguage3_ConvertStringReset(iface: *mut IMultiLanguage3) -> HRESULT;
    fn IMultiLanguage3_GetRfc1766FromLcid( iface: *mut IMultiLanguage3, lcid: LCID, pbstrRfc1766: *mut BSTR, ) -> HRESULT;
    fn IMultiLanguage3_CreateConvertCharset(iface: *mut IMultiLanguage3, src_cp: u32, dst_cp: u32, prop: u32, convert_charset: *mut *mut IMLangConvertCharset) -> i32;
    fn IEnumRfc1766_AddRef(iface: *mut IEnumRfc1766) -> u32;
    fn EnumRfc1766_create(LangId: u16, ppEnum: *mut *mut IEnumRfc1766) -> i32;
    unsafe fn lcid_from_rfc1766(enum: *mut IEnumRfc1766, pLocale: *mut u32, pszRfc1766: *const u16) -> i32;
    fn lstrcmpiW(lpString1: *const u16, lpString2: *const u16) -> i32;
    fn lstrcpyW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn min(a: LONG, b: LONG) -> LONG;
    fn SelectObject(hdc: HDC, hgdiobj: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn GetTextCharsetInfo(hdc: HDC, lpFontSig: *mut FONTSIGNATURE, dwFlags: u32) -> u32;
    fn IsEqualIID(guid1: *const IID, guid2: *const IID) -> i32;
    fn IMLangConvertCharset_AddRef(iface: *mut IMLangConvertCharset) -> u32;
    unsafe fn register_codepages() -> i32;
    fn __wine_register_resources(hinstance: HINSTANCE) -> HRESULT;
    unsafe fn ConvertSJIS2JIS(input: *const i8, count: u32, output: *mut i8) -> u32;
    unsafe fn MLangConvertCharset_create(outer: *const IUnknown, obj: *mut *mut core::ffi::c_void) -> HRESULT;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn IsConvertINetStringAvailable(dwSrcEncoding: u32, dwDstEncoding: u32) -> i32;
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    unsafe fn ConvertJapaneseUnicodeToJIS(src: *const u16, src_len: i32, dst: *mut u8, dst_size: i32) -> i32;
    fn ConvertINetMultiByteToUnicode(pdwMode: *mut u32, dwEncoding: u32, pSrcStr: *mut i8, pcSrcSize: *mut i32, pDstStr: *mut u16, pcDstSize: *mut i32) -> i32;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn ConvertINetUnicodeToMultiByte( pdwMode: *mut u32, dwEncoding: u32, pSrcStr: *const u16, pcSrcSize: *mut i32, pDstStr: *mut i8, pcDstSize: *mut i32, ) -> HRESULT;
    fn debugstr_a(s: *const i8) -> *const i8;
    fn IStream_Stat( this: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: u32, ) -> i32;
    fn IStream_Read( this: *mut IStream, pv: *mut core::ffi::c_void, cb: u32, pcbRead: *mut u32, ) -> i32;
    fn IStream_Write( this: *mut IStream, pv: *const core::ffi::c_void, cb: u32, pcbWritten: *mut u32, ) -> i32;
    #[no_mangle] #[no_mangle] #[no_mangle] pub unsafe extern "system" fn ConvertINetString( pdwMode: *mut u32, dwSrcEncoding: u32, dwDstEncoding: u32, pSrcStr: *const i8, pcSrcSize: *mut i32, pDstStr: *mut i8, pcDstSize: *mut i32, ) -> i32;
}
            } else {
                rc = MultiByteToWideChar(20932, 0, input, count as i32, core::ptr::null_mut(), 0) as u32;
            }
        }
        50220 => {
            sjis_string = HeapAlloc(GetProcessHeap(), 0, count) as *mut i8;
            rc = ConvertJIS2SJIS(input, count, sjis_string);
            if rc != 0 {
                TRACE(b"%s\n\0".as_ptr() as *const i8, debugstr_an(sjis_string, rc));
                if !output.is_null() {
                    rc = MultiByteToWideChar(932, 0, sjis_string, rc as i32, output, out_count as i32) as u32;
                } else {
                    rc = MultiByteToWideChar(932, 0, sjis_string, rc as i32, core::ptr::null_mut(), 0) as u32;
                }
            }
            HeapFree(GetProcessHeap(), 0, sjis_string as *mut core::ffi::c_void);
        }
        _ => {}
    }

    return rc;
}

unsafe fn LockModule() {
    InterlockedIncrement(core::ptr::addr_of_mut!(dll_count));
}

unsafe fn UnlockModule() {
    InterlockedDecrement(core::ptr::addr_of_mut!(dll_count));
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstDLL: HINSTANCE, fdwReason: DWORD, lpv: LPVOID) -> BOOL {
    unsafe {
        match fdwReason {
            DLL_PROCESS_ATTACH => {
                instance = hInstDLL;
                MLANG_tls_index = TlsAlloc();
                DisableThreadLibraryCalls(hInstDLL);
            }
            DLL_PROCESS_DETACH => {
                if lpv.is_null() {
                    TlsFree(MLANG_tls_index);
                }
            }
            _ => {}
        }
        return 1;
    }
}

unsafe fn lcid_to_rfc1766A(lcid: u32, rfc1766: *mut i8, len: i32) -> i32 {
    let mut buffer = [0i8; MAX_RFC1766_NAME];
    let n = GetLocaleInfoA(lcid, LOCALE_SISO639LANGNAME, buffer.as_mut_ptr(), MAX_RFC1766_NAME as i32);
    let mut i: i32;

    if n != 0 {
        let primary_lang = (lcid & 0x3FF) as i32;         // PRIMARYLANGID
        let sub_lang = ((lcid >> 10) & 0x3F) as i32;      // SUBLANGID
        i = primary_lang;
        if ((i == LANG_ENGLISH || i == LANG_CHINESE || i == LANG_ARABIC) &&
            sub_lang == SUBLANG_DEFAULT) ||
            sub_lang > SUBLANG_DEFAULT
        {
            *buffer.as_mut_ptr().offset((n - 1) as isize) = '-' as i8;
            let i2 = GetLocaleInfoA(lcid, LOCALE_SISO3166CTRYNAME, buffer.as_mut_ptr().offset(n as isize), MAX_RFC1766_NAME as i32 - n);
            if i2 == 0 {
                *buffer.as_mut_ptr().offset((n - 1) as isize) = 0;
            }
            i = i2;
        } else {
            i = 0;
        }
        LCMapStringA(LOCALE_USER_DEFAULT, LCMAP_LOWERCASE, buffer.as_ptr(), n + i, rfc1766, len);
        if (n + i) > len {
            return E_INVALIDARG;
        } else {
            return S_OK;
        }
    }
    return E_FAIL;
}

#[no_mangle]
pub unsafe extern "system" fn LcidToRfc1766A(
    lcid: u32,
    pszRfc1766: *mut i8,
    nChar: i32,
) -> i32 {

    TRACE(b"%04x %p %u\n\0".as_ptr() as *const i8, lcid, pszRfc1766 as *const core::ffi::c_void, nChar as u32);

    if pszRfc1766.is_null() {
        return E_INVALIDARG;
    }

    return lcid_to_rfc1766A(lcid, pszRfc1766, nChar);
}

#[no_mangle]
pub unsafe extern "system" fn LcidToRfc1766W(
    lcid: u32,
    pszRfc1766: *mut u16,
    nChar: i32,
) -> i32 {
    {
        let format = b"%04x %p %u\n\0".as_ptr() as *const i8;
        TRACE(format, lcid, pszRfc1766, nChar);
    }
    if pszRfc1766.is_null() {
        return 0x80070057u32 as i32; // E_INVALIDARG
    }
    lcid_to_rfc1766W(lcid, pszRfc1766, nChar)
}

#[no_mangle]
pub unsafe extern "system" fn Rfc1766ToLcidW(pLocale: *mut u32, pszRfc1766: *const u16) -> i32 {
    let mut enumrfc1766: *mut IEnumRfc1766;
    let hr: i32;

    unsafe {
        TRACE(b"(%p, %s)\n\0".as_ptr() as *const i8, pLocale as *const core::ffi::c_void, debugstr_w(pszRfc1766));

        if pLocale.is_null() || pszRfc1766.is_null() {
            return E_INVALIDARG;
        }

        hr = EnumRfc1766_create(0, core::ptr::addr_of_mut!(enumrfc1766));
        if hr < 0 {
            return hr;
        }

        hr = lcid_from_rfc1766(enumrfc1766, pLocale, pszRfc1766);
        IEnumRfc1766_Release(enumrfc1766);

        hr
    }
}

#[no_mangle]
pub unsafe extern "system" fn Rfc1766ToLcidA(lcid: *mut LCID, rfc1766A: LPCSTR) -> HRESULT {
    // WCHAR rfc1766W[MAX_RFC1766_NAME + 1];
    let mut rfc1766W: [u16; MAX_RFC1766_NAME + 1] = [0; MAX_RFC1766_NAME + 1];

    if rfc1766A.is_null() {
        return E_INVALIDARG;
    }

    MultiByteToWideChar(
        0, /* CP_ACP */
        0,
        rfc1766A,
        -1,
        rfc1766W.as_mut_ptr(),
        MAX_RFC1766_NAME as i32,
    );
    rfc1766W[MAX_RFC1766_NAME] = 0;

    Rfc1766ToLcidW(lcid, rfc1766W.as_ptr())
}

unsafe fn release_font(font: HFONT) -> HRESULT {
    // Forward declarations for unknown functions

    // Constants
    const E_FAIL: HRESULT = 0x80004005;
    const S_OK: HRESULT = 0;

    let mut hr: HRESULT = E_FAIL;
    let font_cache_ptr = core::ptr::addr_of_mut!(font_cache);
    EnterCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));

    let mut entry: *mut list = (*font_cache_ptr).Flink;
    while entry != font_cache_ptr {
        let font_list_entry: *mut font_list = (entry as usize - core::mem::offset_of!(font_list, list_entry)) as *mut font_list;
        if (*font_list_entry).font == font {
            list_remove(core::ptr::addr_of_mut!((*font_list_entry).list_entry));
            DeleteObject(font as HGDIOBJ);
            HeapFree(GetProcessHeap(), 0, font_list_entry as *mut core::ffi::c_void);
            hr = S_OK;
            break;
        }
        entry = (*entry).Flink;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(font_cache_critical));

    return hr;
}

unsafe fn clear_font_cache() -> HRESULT {
    EnterCriticalSection(&raw const font_cache_critical as *const CRITICAL_SECTION as *mut CRITICAL_SECTION);

    let head: *mut LIST_ENTRY = &raw const font_cache as *const LIST_ENTRY as *mut LIST_ENTRY;
    let mut entry: *mut LIST_ENTRY = (*head).Flink;
    let mut next: *mut LIST_ENTRY = if entry != head {
        (*entry).Flink
    } else {
        core::ptr::null_mut()
    };

    while entry != head {
        let font_list_entry: *mut font_list = entry as *mut font_list;
        let font_list_entry2: *mut font_list = next as *mut font_list;

        list_remove(&raw mut (*font_list_entry).list_entry);
        DeleteObject((*font_list_entry).font as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, font_list_entry as *mut core::ffi::c_void);

        entry = next;
        if entry != head {
            next = (*entry).Flink;
        }
    }

    LeaveCriticalSection(&raw const font_cache_critical as *const CRITICAL_SECTION as *mut CRITICAL_SECTION);

    return 0i32; // S_OK
}

unsafe fn impl_from_IClassFactory(iface: *mut IClassFactory) -> *mut IClassFactoryImpl {
    (iface as *const u8).sub(core::mem::offset_of!(IClassFactoryImpl, IClassFactory_iface)) as *mut IClassFactoryImpl
}
unsafe fn MLANGCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: *const GUID,
    ppobj: *mut *mut core::ffi::c_void,
) -> i32 {
    TRACE(
        b"%s\n\0".as_ptr() as *const i8,
        debugstr_guid(riid),
    );

    if IsEqualGUID(riid, &IID_IUnknown) || IsEqualGUID(riid, &IID_IClassFactory) {
        IClassFactory_AddRef(iface);
        *ppobj = iface as *mut core::ffi::c_void;
        return 0; // S_OK
    }

    *ppobj = core::ptr::null_mut();
    WARN(
        b"(%p)->(%s,%p), not found\n\0".as_ptr() as *const i8,
        iface as *mut core::ffi::c_void,
        debugstr_guid(riid),
        ppobj as *mut core::ffi::c_void,
    );
    return 0x80004002u32 as i32; // E_NOINTERFACE
}
unsafe fn MLANGCF_AddRef(iface: *mut IClassFactory) -> u32 {
    let this = impl_from_IClassFactory(iface);
    InterlockedIncrement(&raw mut (*this).ref) as u32
}
unsafe fn MLANGCF_Release(iface: *mut IClassFactory) -> u32 {
    let This = impl_from_IClassFactory(iface);
    let ref_val = InterlockedDecrement(&raw mut (*This).ref) as u32;

    if ref_val == 0 {
        TRACE(b"Destroying %p\n\0".as_ptr() as *const i8, This);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }

    ref_val
}

unsafe extern "system" {
    fn TRACE(format: *const core::ffi::c_char, ...);
    fn debugstr_guid(guid: *const core::ffi::c_void) -> *const core::ffi::c_char;
    fn IUnknown_QueryInterface(punk: *mut IUnknown, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn IUnknown_Release(punk: *mut IUnknown) -> u32;
}
unsafe fn MLANGCF_CreateInstance(
    iface: *mut IClassFactory,
    pOuter: *mut IUnknown,
    riid: *const GUID,
    ppobj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface);
    let hres: HRESULT;
    let mut punk: *mut IUnknown = core::ptr::null_mut();

    TRACE(b"(%p)->(%p,%s,%p)\n\0".as_ptr() as *const core::ffi::c_char, This as *const core::ffi::c_void, pOuter as *const core::ffi::c_void, debugstr_guid(riid as *const core::ffi::c_void), ppobj as *const core::ffi::c_void);

    *ppobj = core::ptr::null_mut();
    hres = ((*This).pfnCreateInstance)(pOuter, &mut punk as *mut *mut IUnknown as *mut *mut core::ffi::c_void);
    if hres >= 0 {
        hres = IUnknown_QueryInterface(punk, riid, ppobj);
        IUnknown_Release(punk);
    }

    TRACE(b"returning (%p) -> %x\n\0".as_ptr() as *const core::ffi::c_char, *ppobj, hres);

    hres
}
unsafe fn MLANGCF_LockServer(iface: *mut IClassFactory, dolock: i32) -> i32 {
    if dolock != 0 {
        LockModule();
    } else {
        UnlockModule();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    iid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut i: usize = 0;
    let factory: *mut IClassFactoryImpl;

    TRACE(
        b"%s %s %p\n\0".as_ptr() as *const core::ffi::c_char,
        debugstr_guid(rclsid),
        debugstr_guid(iid),
        ppv as *const core::ffi::c_void,
    );

    if !IsEqualGUID(core::ptr::addr_of!(IID_IClassFactory), iid)
        && !IsEqualGUID(core::ptr::addr_of!(IID_IUnknown), iid)
    {
        return E_NOINTERFACE;
    }

    while i < object_creation.len() {
        if IsEqualGUID(core::ptr::addr_of!(object_creation[i].clsid), rclsid) {
            break;
        }
        i += 1;
    }

    if i == object_creation.len() {
        FIXME(
            b"%s: no class found.\n\0".as_ptr() as *const core::ffi::c_char,
            debugstr_guid(rclsid),
        );
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    TRACE(
        b"Creating a class factory for %s\n\0".as_ptr() as *const core::ffi::c_char,
        object_creation[i].szClassName,
    );

    factory = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<IClassFactoryImpl>(),
    ) as *mut IClassFactoryImpl;

    if factory.is_null() {
        return E_OUTOFMEMORY;
    }

    (*factory).IClassFactory_iface.lpVtbl = &raw const MLANGCF_Vtbl;
    (*factory).ref = 1;
    (*factory).pfnCreateInstance = object_creation[i].pfnCreateInstance;

    *ppv = core::ptr::addr_of_mut!((*factory).IClassFactory_iface) as *mut core::ffi::c_void;

    TRACE(
        b"(%p) <- %p\n\0".as_ptr() as *const core::ffi::c_char,
        ppv as *const core::ffi::c_void,
        core::ptr::addr_of!((*factory).IClassFactory_iface) as *const core::ffi::c_void,
    );

    S_OK
}

unsafe fn impl_from_IEnumCodePage(iface: *mut IEnumCodePage) -> *mut EnumCodePage_impl {
    let offset = core::mem::offset_of!(EnumCodePage_impl, IEnumCodePage_iface);
    (iface as *mut u8).sub(offset) as *mut EnumCodePage_impl
}
unsafe fn fnIEnumCodePage_QueryInterface(
    iface: *mut IEnumCodePage,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);

    TRACE(
        b"%p -> %s\n\0".as_ptr() as *const core::ffi::c_char,
        This as *mut core::ffi::c_void,
        debugstr_guid(riid),
    );

    if IsEqualGUID(riid, &IID_IUnknown as *const GUID)
        || IsEqualGUID(riid, &IID_IEnumCodePage as *const GUID)
    {
        IEnumCodePage_AddRef(iface);
        TRACE(
            b"Returning IID_IEnumCodePage %p ref = %d\n\0".as_ptr() as *const core::ffi::c_char,
            This as *mut core::ffi::c_void,
            (*This).ref,
        );
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumCodePage_iface) as *mut core::ffi::c_void;
        return 0; // S_OK
    }

    WARN(
        b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const core::ffi::c_char,
        This as *mut core::ffi::c_void,
        debugstr_guid(riid),
        ppvObject as *mut core::ffi::c_void,
    );
    return 0x80004002u32 as HRESULT; // E_NOINTERFACE
}
unsafe fn fnIEnumCodePage_AddRef(iface: *mut IEnumCodePage) -> u32 {
    unsafe {
        let This = impl_from_IEnumCodePage(iface);
        return InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref)) as u32;
    }
}
unsafe fn fnIEnumCodePage_Release(iface: *mut IEnumCodePage) -> u32 {
    let This = impl_from_IEnumCodePage(iface);
    let ref = ntdll_sys::nt::InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref));
    TRACE(
        b"%p ref = %d\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        ref as i32,
    );
    if ref == 0 {
        TRACE(
            b"Destroying %p\n\0".as_ptr() as *const i8,
            This as *const core::ffi::c_void,
        );
        HeapFree(GetProcessHeap(), 0, (*This).cpinfo as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }
    ref
}

const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
unsafe fn fnIEnumCodePage_Clone(
    iface: *mut IEnumCodePage,
    ppEnum: *mut *mut IEnumCodePage,
) -> HRESULT {
    let This = impl_from_IEnumCodePage(iface);
    FIXME(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}

unsafe extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn wine_dbgstr_w(wstr: *const u16) -> *const i8;
}
unsafe fn fnIEnumCodePage_Next(
    iface: *mut IEnumCodePage,
    celt: u32,
    rgelt: *mut MIMECPINFO,
    pceltFetched: *mut u32,
) -> i32 {
    let mut celt = celt;
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);

    TRACE(
        b"%p %u %p %p\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        celt,
        rgelt as *const core::ffi::c_void,
        pceltFetched as *const core::ffi::c_void,
    );

    if pceltFetched.is_null() {
        return 1; // S_FALSE
    }
    *pceltFetched = 0;

    if rgelt.is_null() {
        return 1; // S_FALSE
    }

    if (*This).pos + celt > (*This).total {
        celt = (*This).total - (*This).pos;
    }

    if celt == 0 {
        return 1; // S_FALSE
    }

    core::ptr::copy_nonoverlapping(
        rgelt,
        (*This).cpinfo.add((*This).pos as usize),
        celt as usize,
    );
    *pceltFetched = celt;
    (*This).pos += celt;

    let mut i: u32 = 0;
    while i < celt {
        let r = &*rgelt.add(i as usize);
        TRACE(
            b"#%u: %08x %u %u %s %s %s %s %s %s %d\n\0".as_ptr() as *const i8,
            i,
            r.dwFlags,
            r.uiCodePage,
            r.uiFamilyCodePage,
            wine_dbgstr_w(r.wszDescription.as_ptr()),
            wine_dbgstr_w(r.wszWebCharset.as_ptr()),
            wine_dbgstr_w(r.wszHeaderCharset.as_ptr()),
            wine_dbgstr_w(r.wszBodyCharset.as_ptr()),
            wine_dbgstr_w(r.wszFixedWidthFont.as_ptr()),
            wine_dbgstr_w(r.wszProportionalFont.as_ptr()),
            r.bGDICharset as i32,
        );
        i += 1;
    }

    return 0; // S_OK
}
unsafe fn fnIEnumCodePage_Reset(iface: *mut IEnumCodePage) -> HRESULT {
    let This: *mut EnumCodePage_impl = impl_from_IEnumCodePage(iface);
    TRACE(b"%p\n\0".as_ptr() as *const i8, This);
    (*This).pos = 0;
    0 // S_OK
}
unsafe fn fnIEnumCodePage_Skip(iface: *mut IEnumCodePage, celt: u32) -> i32 {
    unsafe {
        let This = impl_from_IEnumCodePage(iface);
        TRACE(b"%p %u\n\0".as_ptr() as *const i8, This, celt);
        if celt >= (*This).total {
            return 1;
        }
        (*This).pos += celt;
        return 0;
    }
}
unsafe fn EnumCodePage_create(
    mlang: *mut MLang_impl,
    mut grfFlags: u32,
    LangId: u32,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> i32 {
    let mut ecp: *mut EnumCodePage_impl;
    let mut cpinfo: *mut MIMECPINFO;
    let mut i: u32;
    let mut n: u32;

    TRACE(
        b"%p, %08x, %04x, %p\n\0".as_ptr() as *const core::ffi::c_char,
        mlang as *mut core::ffi::c_void,
        grfFlags,
        LangId as u32,
        ppEnumCodePage as *mut core::ffi::c_void,
    );

    if grfFlags == 0 {
        /* enumerate internal data base of encodings */
        grfFlags = 0x00200000; /* MIMECONTF_MIME_LATEST */
    }

    ecp = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumCodePage_impl>(),
    ) as *mut EnumCodePage_impl;
    (*ecp).IEnumCodePage_iface.lpVtbl = &IEnumCodePage_vtbl;
    (*ecp).ref = 1;
    (*ecp).pos = 0;
    (*ecp).total = 0;

    i = 0;
    while i < mlang_data.len() as u32 {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            let info = &*((*mlang_data[i as usize].mime_cp_info).add(n as usize));
            if info.flags & grfFlags != 0 {
                (*ecp).total += 1;
            }
            n += 1;
        }
        i += 1;
    }

    (*ecp).cpinfo = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<MIMECPINFO>() * (*ecp).total as usize,
    ) as *mut MIMECPINFO;
    cpinfo = (*ecp).cpinfo;

    i = 0;
    while i < mlang_data.len() as u32 {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            let info = &*((*mlang_data[i as usize].mime_cp_info).add(n as usize));
            if info.flags & grfFlags != 0 {
                fill_cp_info(
                    &raw const mlang_data[i as usize],
                    n,
                    cpinfo,
                );
                cpinfo = cpinfo.add(1);
            }
            n += 1;
        }
        i += 1;
    }

    TRACE(
        b"enumerated %d codepages with flags %08x\n\0".as_ptr() as *const core::ffi::c_char,
        (*ecp).total,
        grfFlags,
    );

    *ppEnumCodePage = &mut (*ecp).IEnumCodePage_iface as *mut IEnumCodePage;

    0 // S_OK
}

unsafe fn impl_from_IEnumScript(iface: *mut IEnumScript) -> *mut EnumScript_impl {
    (iface as *mut u8)
        .sub(core::mem::offset_of!(EnumScript_impl, IEnumScript_iface))
        as *mut EnumScript_impl
}

const S_OK: i32 = 0;
const E_NOINTERFACE: i32 = 0x80004002;
unsafe fn fnIEnumScript_QueryInterface(
    iface: *mut IEnumScript,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);

    TRACE(
        b"%p -> %s\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        debugstr_guid(riid),
    );

    if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumScript))
    {
        IEnumScript_AddRef(iface);
        TRACE(
            b"Returning IID_IEnumScript %p ref = %d\n\0".as_ptr() as *const i8,
            This as *mut core::ffi::c_void,
            (*This).ref,
        );
        *ppvObject = core::ptr::addr_of_mut!((*This).IEnumScript_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN(
        b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        debugstr_guid(riid),
        ppvObject as *mut core::ffi::c_void,
    );
    E_NOINTERFACE
}
unsafe fn fnIEnumScript_AddRef(iface: *mut IEnumScript) -> u32 {
    let This = impl_from_IEnumScript(iface);
    InterlockedIncrement(&raw mut (*This).ref) as u32
}
unsafe fn fnIEnumScript_Release(iface: *mut IEnumScript) -> u32 {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);
    let r#ref: u32 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref)) as u32;

    TRACE(
        b"%p ref = %d\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        r#ref as i32,
    );
    if r#ref == 0 {
        TRACE(
            b"Destroying %p\n\0".as_ptr() as *const i8,
            This as *mut core::ffi::c_void,
        );
        HeapFree(GetProcessHeap(), 0, (*This).script_info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }

    r#ref
}

const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
unsafe fn fnIEnumScript_Clone(
    iface: *mut IEnumScript,
    ppEnum: *mut *mut IEnumScript,
) -> HRESULT {
    let This: *mut EnumScript_impl = impl_from_IEnumScript(iface);
    FIXME(b"%p %p: stub!\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}
unsafe fn fnIEnumScript_Next(
    iface: *mut IEnumScript,
    celt: u32,
    rgelt: *mut SCRIPTINFO,
    pceltFetched: *mut u32,
) -> HRESULT {
    let This = impl_from_IEnumScript(iface);

    TRACE(b"%p %u %p %p\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);

    if pceltFetched.is_null() || rgelt.is_null() {
        return 0x80004005u32 as i32; /* E_FAIL */
    }

    *pceltFetched = 0;

    let mut celt = celt;
    if (This.pos as usize).wrapping_add(celt as usize) > This.total as usize {
        celt = This.total - This.pos;
    }

    if celt == 0 {
        return 1; /* S_FALSE */
    }

    core::ptr::copy_nonoverlapping(
        (*This).script_info.add(This.pos),
        rgelt,
        celt as usize,
    );

    *pceltFetched = celt;
    (*This).pos += celt;

    0 /* S_OK */
}
unsafe fn fnIEnumScript_Reset(
    iface: *mut IEnumScript,
) -> HRESULT {
    let This = impl_from_IEnumScript(iface);

    TRACE(b"%p\n\0".as_ptr() as *const i8, This);

    (*This).pos = 0;
    return 0; // S_OK
}

const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
unsafe fn fnIEnumScript_Skip(
    iface: *mut IEnumScript,
    celt: u32,
) -> i32 {
    let This = impl_from_IEnumScript(iface);
    TRACE(b"%p %u\n\0".as_ptr() as *const i8, This as *const c_void, celt);
    if celt >= (*This).total {
        return S_FALSE;
    }
    (*This).pos += celt;
    S_OK
}

unsafe fn impl_from_IMLangFontLink(iface: *mut IMLangFontLink) -> *mut MLang_impl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(MLang_impl, IMLangFontLink_iface) as isize)) as *mut MLang_impl
}
unsafe fn fnIMLangFontLink_QueryInterface(
    iface: *mut IMLangFontLink,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMLangFontLink(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid,
        ppvObject,
    );
}
unsafe fn fnIMLangFontLink_AddRef(iface: *mut IMLangFontLink) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}
unsafe fn fnIMLangFontLink_Release(iface: *mut IMLangFontLink) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}
unsafe fn fnIMLangFontLink_GetCharCodePages(
    iface: *mut IMLangFontLink,
    ch_src: u16,
    codepages: *mut u32,
) -> i32 {
    let this = impl_from_IMLangFontLink(iface);
    IMLangFontLink2_GetCharCodePages(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        ch_src,
        codepages,
    )
}
unsafe fn fnIMLangFontLink_GetStrCodePages(
    iface: *mut IMLangFontLink,
    src: *const u16,
    src_len: i32,
    priority_cp: u32,
    codepages: *mut u32,
    ret_len: *mut i32,
) -> HRESULT {
    let This = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_GetStrCodePages(
        core::ptr::addr_of_mut!((*This).IMLangFontLink2_iface),
        src,
        src_len,
        priority_cp,
        codepages,
        ret_len,
    );
}
unsafe fn fnIMLangFontLink_CodePageToCodePages(
    iface: *mut IMLangFontLink,
    codepage: u32,
    codepages: *mut u32,
) -> HRESULT {
    let this = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_CodePageToCodePages(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        codepage,
        codepages,
    );
}
unsafe fn fnIMLangFontLink_CodePagesToCodePage(
    iface: *mut IMLangFontLink,
    codepages: u32,
    def_codepage: u32,
    codepage: *mut u32,
) -> i32 {
    let this: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_CodePagesToCodePage(
        core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface),
        codepages,
        def_codepage,
        codepage,
    );
}
unsafe fn fnIMLangFontLink_GetFontCodePages(
    iface: *mut IMLangFontLink,
    hdc: HDC,
    hfont: HFONT,
    codepages: *mut u32,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink(iface);
    return IMLangFontLink2_GetFontCodePages(
        core::ptr::addr_of_mut!((*This).IMLangFontLink2_iface),
        hdc,
        hfont,
        codepages,
    );
}
unsafe fn fnIMLangFontLink_MapFont(
    iface: *mut IMLangFontLink,
    hDC: HDC,
    dwCodePages: u32,
    hSrcFont: HFONT,
    phDestFont: *mut HFONT,
) -> HRESULT {
    TRACE(
        b"(%p)->%p %08x %p %p\n\0".as_ptr() as *const i8,
        iface as *mut core::ffi::c_void,
        hDC as *mut core::ffi::c_void,
        dwCodePages,
        hSrcFont as *mut core::ffi::c_void,
        phDestFont as *mut core::ffi::c_void,
    );
    return map_font(hDC, dwCodePages, hSrcFont, phDestFont);
}
unsafe fn fnIMLangFontLink_ReleaseFont(
    iface: *mut IMLangFontLink,
    hFont: HFONT,
) -> HRESULT {
    TRACE(b"(%p)->%p\n\0".as_ptr() as *const i8, iface, hFont);
    return release_font(hFont);
}

unsafe extern "C" {
    fn TRACE(fmt: *const i8, ...);
}
unsafe fn fnIMLangFontLink_ResetFontMapping(iface: *mut IMLangFontLink) -> HRESULT {
    TRACE(b"(%p)\n\0".as_ptr() as *const i8, iface);
    return clear_font_cache();
}

unsafe fn impl_from_IMultiLanguage(iface: *mut IMultiLanguage) -> *mut MLang_impl {
    (iface as *mut u8).byte_offset(-(core::mem::offset_of!(MLang_impl, IMultiLanguage_iface) as isize)) as *mut MLang_impl
}
unsafe fn fnIMultiLanguage_QueryInterface(
    iface: *mut IMultiLanguage,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface),
        riid,
        obj,
    )
}
unsafe fn fnIMultiLanguage_AddRef(iface: *mut IMultiLanguage) -> u32 {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}
unsafe fn fnIMultiLanguage_Release(iface: *mut IMultiLanguage) -> u32 {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}
unsafe fn fnIMultiLanguage_GetNumberOfCodePageInfo(
    iface: *mut IMultiLanguage,
    cp: *mut UINT,
) -> HRESULT {
    unsafe extern "system" {
        fn IMultiLanguage3_GetNumberOfCodePageInfo(iface: *mut IMultiLanguage3, pc: *mut UINT) -> HRESULT;
    }
    unsafe extern "C" {
        fn TRACE(format: *const i8, ...);
    }
    let This = impl_from_IMultiLanguage(iface);
    TRACE(b"(%p, %p)\n\0".as_ptr() as *const i8, This, cp);
    IMultiLanguage3_GetNumberOfCodePageInfo(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        cp,
    )
}

const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
unsafe fn fnIMultiLanguage_GetCodePageInfo(
    iface: *mut IMultiLanguage,
    uiCodePage: u32,
    pCodePageInfo: *mut MIMECPINFO,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);

    TRACE(
        b"%p, %u, %p\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        uiCodePage,
        pCodePageInfo as *const core::ffi::c_void,
    );

    let mut i: u32 = 0;
    while (i as usize) < mlang_data.len() {
        let mlang: &mlang_data_struct = &mlang_data[i as usize];
        let mut n: u32 = 0;
        while (n as u32) < mlang.number_of_cp {
            if mlang.mime_cp_info[n as usize].cp == uiCodePage {
                fill_cp_info(
                    &raw const mlang_data[i as usize],
                    n,
                    pCodePageInfo,
                );
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    return S_FALSE;
}
unsafe fn fnIMultiLanguage_GetFamilyCodePage(
    iface: *mut IMultiLanguage,
    cp: u32,
    family_cp: *mut u32,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    let iface_ptr = &(*this).IMultiLanguage3_iface as *const IMultiLanguage3 as *mut IMultiLanguage3;
    IMultiLanguage3_GetFamilyCodePage(iface_ptr, cp, family_cp)
}
unsafe fn fnIMultiLanguage_EnumCodePages(
    iface: *mut IMultiLanguage,
    grfFlags: DWORD,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    TRACE(b"%p %08x %p\n\0".as_ptr() as *const i8, This, grfFlags, ppEnumCodePage);
    return EnumCodePage_create(This, grfFlags, 0, ppEnumCodePage);
}

unsafe extern "system" fn IMultiLanguage3_IsConvertible(iface: *mut IMultiLanguage3, src_enc: u32, dst_enc: u32) -> HRESULT;
unsafe fn fnIMultiLanguage_IsConvertible(
    iface: *mut IMultiLanguage,
    src_enc: u32,
    dst_enc: u32,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_IsConvertible(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface), src_enc, dst_enc);
}
unsafe fn fnIMultiLanguage_ConvertString(
    iface: *mut IMultiLanguage,
    mode: *mut u32,
    src_enc: u32,
    dst_enc: u32,
    src: *mut u8,
    src_size: *mut u32,
    dest: *mut u8,
    dest_size: *mut u32,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertString(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        src_enc,
        dst_enc,
        src,
        src_size,
        dest,
        dest_size,
    );
}
unsafe fn fnIMultiLanguage_ConvertStringToUnicode(
    iface: *mut IMultiLanguage,
    mode: *mut DWORD,
    src_enc: DWORD,
    src: *mut CHAR,
    src_size: *mut UINT,
    dest: *mut WCHAR,
    dest_size: *mut UINT,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertStringToUnicode(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        src_enc,
        src,
        src_size,
        dest,
        dest_size,
    );
}
unsafe fn fnIMultiLanguage_ConvertStringFromUnicode(
    iface: *mut IMultiLanguage,
    mode: *mut DWORD,
    encoding: DWORD,
    src: *mut WCHAR,
    src_size: *mut UINT,
    dest: *mut CHAR,
    dest_size: *mut UINT,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    return IMultiLanguage3_ConvertStringFromUnicode(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        mode,
        encoding,
        src,
        src_size,
        dest,
        dest_size,
    );
}
unsafe fn fnIMultiLanguage_ConvertStringReset(
    iface: *mut IMultiLanguage,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_ConvertStringReset(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}
unsafe fn fnIMultiLanguage_GetRfc1766FromLcid(
    iface: *mut IMultiLanguage,
    lcid: LCID,
    pbstrRfc1766: *mut BSTR,
) -> HRESULT {
    unsafe {
        let this: *mut MLang_impl = impl_from_IMultiLanguage(iface);
        let iface3: *mut IMultiLanguage3 = core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface);
        IMultiLanguage3_GetRfc1766FromLcid(iface3, lcid, pbstrRfc1766)
    }
}

unsafe extern "system" {
    fn IMultiLanguage3_GetLcidFromRfc1766(
        pIMultiLanguage3: *mut IMultiLanguage3,
        locale: *mut LCID,
        rfc1766: BSTR,
    ) -> HRESULT;
}
unsafe fn fnIMultiLanguage_GetLcidFromRfc1766(
    iface: *mut IMultiLanguage,
    locale: *mut LCID,
    rfc1766: BSTR,
) -> HRESULT {
    let this = impl_from_IMultiLanguage(iface);
    IMultiLanguage3_GetLcidFromRfc1766(
        core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface),
        locale,
        rfc1766,
    )
}

unsafe fn impl_from_IEnumRfc1766(iface: *mut IEnumRfc1766) -> *mut EnumRfc1766_impl {
    (iface as *mut u8).sub(core::mem::offset_of!(EnumRfc1766_impl, IEnumRfc1766_iface)) as *mut EnumRfc1766_impl
}

const S_OK: i32 = 0;
const E_NOINTERFACE: i32 = 0x80004002;
unsafe fn fnIEnumRfc1766_QueryInterface(
    iface: *mut IEnumRfc1766,
    riid: *const GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    unsafe {
        let this = impl_from_IEnumRfc1766(iface);
        TRACE(
            b"%p -> %s\n\0".as_ptr() as *const i8,
            this as *const core::ffi::c_void,
            debugstr_guid(riid),
        );
        if IsEqualGUID(riid, core::ptr::addr_of!(IID_IUnknown))
            || IsEqualGUID(riid, core::ptr::addr_of!(IID_IEnumRfc1766))
        {
            IEnumRfc1766_AddRef(iface);
            TRACE(
                b"Returning IID_IEnumRfc1766 %p ref = %d\n\0".as_ptr() as *const i8,
                this as *const core::ffi::c_void,
                (*this).ref,
            );
            *ppvObject = core::ptr::addr_of_mut!((*this).IEnumRfc1766_iface) as *mut core::ffi::c_void;
            return S_OK;
        }
        WARN(
            b"(%p) -> (%s,%p), not found\n\0".as_ptr() as *const i8,
            this as *const core::ffi::c_void,
            debugstr_guid(riid),
            ppvObject,
        );
        return E_NOINTERFACE;
    }
}
unsafe fn fnIEnumRfc1766_AddRef(iface: *mut IEnumRfc1766) -> u32 {
    let this = impl_from_IEnumRfc1766(iface);
    ntdll_sys::nt::InterlockedIncrement(core::ptr::addr_of_mut!((*this).ref)) as u32
}
unsafe fn fnIEnumRfc1766_Release(
    iface: *mut IEnumRfc1766,
) -> u32 {
    let this = impl_from_IEnumRfc1766(iface);
    let ref_count = ntdll_sys::nt::InterlockedDecrement(core::ptr::addr_of_mut!((*this).ref)) as u32;

    TRACE(b"%p ref = %d\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, ref_count as i32);
    if ref_count == 0 {
        TRACE(b"Destroying %p\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*this).info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    return ref_count;
}

const E_NOTIMPL: i32 = -2147467263; // 0x80004001
unsafe fn fnIEnumRfc1766_Clone(
    iface: *mut IEnumRfc1766,
    ppEnum: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This = impl_from_IEnumRfc1766(iface);
    FIXME(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnum);
    return E_NOTIMPL;
}
unsafe fn fnIEnumRfc1766_Next(
    iface: *mut IEnumRfc1766,
    celt: u32,
    rgelt: *mut RFC1766INFO,
    pceltFetched: *mut u32,
) -> i32 {
    let This = impl_from_IEnumRfc1766(iface);

    TRACE(
        b"%p %u %p %p\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        celt,
        rgelt as *const core::ffi::c_void,
        pceltFetched as *const core::ffi::c_void,
    );

    if pceltFetched.is_null() {
        return 1; // S_FALSE
    }
    *pceltFetched = 0;

    if rgelt.is_null() {
        return 1; // S_FALSE
    }

    if (*This).pos + celt > (*This).total {
        celt = (*This).total - (*This).pos;
    }

    if celt == 0 {
        return 1; // S_FALSE
    }

    core::ptr::copy_nonoverlapping(
        (*This).info.add((*This).pos as usize),
        rgelt,
        celt as usize,
    );
    *pceltFetched = celt;
    (*This).pos += celt;

    for i in 0..celt as usize {
        let entry = &*rgelt.add(i);
        let s1 = wine_dbgstr_w(
            &raw const entry.wszRfc1766 as *const u16,
        );
        let s2 = wine_dbgstr_w(
            &raw const entry.wszLocaleName as *const u16,
        );
        TRACE(
            b"#%u: %08x %s %s\n\0".as_ptr() as *const i8,
            i,
            entry.lcid,
            s1,
            s2,
        );
    }

    return 0; // S_OK
}
unsafe fn fnIEnumRfc1766_Reset(
    iface: *mut IEnumRfc1766,
) -> i32 {
    let This: *mut EnumRfc1766_impl = impl_from_IEnumRfc1766(iface);
    TRACE(b"%p\n\0".as_ptr() as *const i8, This);
    (*This).pos = 0;
    return 0i32;
}
unsafe fn fnIEnumRfc1766_Skip(iface: *mut IEnumRfc1766, celt: u32) -> HRESULT {
    let This = impl_from_IEnumRfc1766(iface);
    // TRACE("%p %u\n", This, celt);
    if celt >= (*This).total {
        return 1; // S_FALSE
    }
    (*This).pos += celt;
    return 0; // S_OK
}

const MAX_LOCALE_NAME: u32 = 85;
const LOCALE_SLANGUAGE: u32 = 0x00000002;
unsafe fn fnIMultiLanguage_EnumRfc1766(
    iface: *mut IMultiLanguage,
    ppEnumRfc1766: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This = impl_from_IMultiLanguage(iface);
    TRACE(b"%p %p\n\0".as_ptr() as *const i8, This, ppEnumRfc1766);
    return EnumRfc1766_create(0, ppEnumRfc1766);
}
unsafe fn fnIMultiLanguage_GetRfc1766Info(
    iface: *mut IMultiLanguage,
    Locale: LCID,
    pRfc1766Info: *mut RFC1766INFO,
) -> HRESULT {
    let mut type_: u32 = LOCALE_SLANGUAGE;

    TRACE(
        b"(%p, 0x%04x, %p)\n\0".as_ptr() as *const i8,
        iface,
        Locale,
        pRfc1766Info,
    );

    if pRfc1766Info.is_null() {
        return E_INVALIDARG;
    }

    if ((Locale as u16) & 0x3ff) == LANG_ENGLISH as u16
        || ((Locale as u16) & 0x3ff) == LANG_CHINESE as u16
        || ((Locale as u16) & 0x3ff) == LANG_ARABIC as u16
    {
        if ((Locale >> 10) as u16) == 0 {
            type_ = LOCALE_SENGLANGUAGE;
        }
    } else {
        if ((Locale >> 10) as u16) == 0 {
            TRACE(b"SUBLANGID missing in 0x%04x\n\0".as_ptr() as *const i8, Locale);
            return E_FAIL;
        }
    }

    (*pRfc1766Info).lcid = Locale;
    (*pRfc1766Info).wszRfc1766[0] = 0;
    (*pRfc1766Info).wszLocaleName[0] = 0;

    let rfc1766w_ptr = core::ptr::addr_of_mut!((*pRfc1766Info).wszRfc1766[0]);
    let localename_ptr = core::ptr::addr_of_mut!((*pRfc1766Info).wszLocaleName[0]);

    if lcid_to_rfc1766W(Locale, rfc1766w_ptr, MAX_RFC1766_NAME as u32) == 0
        && GetLocaleInfoW(Locale, type_, localename_ptr, MAX_LOCALE_NAME as i32) > 0
    {
        return S_OK;
    }

    E_INVALIDARG
}
unsafe fn fnIMultiLanguage_CreateConvertCharset(
    iface: *mut IMultiLanguage,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
    convert_charset: *mut *mut IMLangConvertCharset,
) -> i32 {
    let This = impl_from_IMultiLanguage(iface);
    let iface3 = core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface);
    IMultiLanguage3_CreateConvertCharset(iface3, src_cp, dst_cp, prop, convert_charset)
}

unsafe fn impl_from_IMultiLanguage3(iface: *mut IMultiLanguage3) -> *mut MLang_impl {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(MLang_impl, IMultiLanguage3_iface)) as *mut MLang_impl;
}

const E_NOINTERFACE: HRESULT = 0x80004002u32 as i32;
const S_OK: HRESULT = 0;
unsafe fn fnIMultiLanguage3_QueryInterface(
    iface: *mut IMultiLanguage3,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this: *mut MLang_impl = impl_from_IMultiLanguage3(iface);

    TRACE(
        b"(%p)->(%s %p)\n\0".as_ptr() as *const i8,
        this as *const core::ffi::c_void,
        debugstr_guid(riid),
        obj as *const core::ffi::c_void,
    );

    if IsEqualGUID(riid, &raw const IID_IUnknown) != 0
        || IsEqualGUID(riid, &raw const IID_IMultiLanguage) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMultiLanguage_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangCodePages) != 0
        || IsEqualGUID(riid, &raw const IID_IMLangFontLink) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMLangFontLink_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangFontLink2) != 0 {
        *obj = core::ptr::addr_of_mut!((*this).IMLangFontLink2_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMultiLanguage2) != 0
        || IsEqualGUID(riid, &raw const IID_IMultiLanguage3) != 0
    {
        *obj = core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface) as *mut core::ffi::c_void;
    } else if IsEqualGUID(riid, &raw const IID_IMLangLineBreakConsole) != 0 {
        *obj = core::ptr::addr_of_mut!((*this).IMLangLineBreakConsole_iface) as *mut core::ffi::c_void;
    } else {
        WARN(
            b"(%p)->(%s,%p),not found\n\0".as_ptr() as *const i8,
            this as *const core::ffi::c_void,
            debugstr_guid(riid),
            obj as *const core::ffi::c_void,
        );
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IMultiLanguage3_AddRef(iface);
    S_OK
}
unsafe fn fnIMultiLanguage3_AddRef(iface: *mut IMultiLanguage3) -> u32 {
    let This = impl_from_IMultiLanguage3(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref)) as u32;
}
unsafe fn fnIMultiLanguage3_Release(iface: *mut IMultiLanguage3) -> u32 {
    let This = impl_from_IMultiLanguage3(iface);
    let ref_val = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref)) as u32;
    TRACE(b"(%p)->(%d)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, ref_val as i32);
    if ref_val == 0 {
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
        UnlockModule();
    }
    return ref_val;
}

const E_INVALIDARG: i32 = 0x80070057;
const S_OK: i32 = 0;
unsafe fn fnIMultiLanguage3_GetNumberOfCodePageInfo(
    iface: *mut IMultiLanguage3,
    pcCodePage: *mut UINT,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);
    TRACE(b"%p, %p\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, pcCodePage as *const core::ffi::c_void);
    if pcCodePage.is_null() {
        return E_INVALIDARG;
    }
    (*pcCodePage) = (*This).total_cp;
    return S_OK;
}

const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
unsafe fn fnIMultiLanguage3_GetCodePageInfo(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    LangId: u16,
    pCodePageInfo: *mut MIMECPINFO,
) -> i32 {
    let this = impl_from_IMultiLanguage3(iface);

    TRACE(
        b"%p, %u, %04x, %p\n\0".as_ptr() as *const i8,
        this,
        uiCodePage,
        LangId,
        pCodePageInfo,
    );

    let array_len = core::mem::size_of_val(&mlang_data) / core::mem::size_of_val(&mlang_data[0]);

    for i in 0..array_len {
        let n_loop = mlang_data[i].number_of_cp;
        for n in 0..n_loop {
            if mlang_data[i].mime_cp_info[n].cp == uiCodePage {
                fill_cp_info(
                    core::ptr::addr_of!(mlang_data[i]) as *const mlang_data,
                    n as u32,
                    pCodePageInfo,
                );
                return S_OK;
            }
        }
    }

    return S_FALSE;
}
unsafe fn fnIMultiLanguage3_GetFamilyCodePage(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    puiFamilyCodePage: *mut u32,
) -> i32 {
    GetFamilyCodePage(uiCodePage, puiFamilyCodePage)
}
unsafe fn fnIMultiLanguage3_EnumCodePages(
    iface: *mut IMultiLanguage3,
    grfFlags: u32,
    LangId: u16,
    ppEnumCodePage: *mut *mut IEnumCodePage,
) -> HRESULT {

    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE(
        b"%p %08x %04x %p\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        grfFlags as u32,
        LangId as u32,
        ppEnumCodePage as *mut *mut IEnumCodePage,
    );
    EnumCodePage_create(This, grfFlags, LangId, ppEnumCodePage)
}

const CP_ACP: u32 = 0;
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
const S_OK: HRESULT = 0;
const MAX_MIMECSET_NAME: usize = 64;
unsafe fn fnIMultiLanguage3_GetCharsetInfo(
    iface: *mut IMultiLanguage3,
    Charset: BSTR,
    pCharsetInfo: PMIMECSETINFO,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);

    TRACE(
        b"%p %s %p\n\0".as_ptr() as *const i8,
        This as *const _,
        debugstr_w(Charset) as *const _,
        pCharsetInfo as *const _,
    );

    if pCharsetInfo.is_null() {
        return E_FAIL;
    }

    let mut i = 0;
    while i < mlang_data.len() {
        let mut n = 0;
        while n < mlang_data[i].number_of_cp as usize {
            let mut csetW = [0u16; MAX_MIMECSET_NAME];
            MultiByteToWideChar(
                CP_ACP,
                0,
                mlang_data[i].mime_cp_info[n].web_charset as *const u8,
                -1i32,
                csetW.as_mut_ptr(),
                MAX_MIMECSET_NAME as i32,
            );
            if lstrcmpiW(Charset, csetW.as_ptr()) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), csetW.as_ptr());
                return S_OK;
            }
            let alias = mlang_data[i].mime_cp_info[n].alias;
            if !alias.is_null() && lstrcmpiW(Charset, alias) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), alias);
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    /* FIXME:
     * Since we do not support charsets like iso-2022-jp and do not have
     * them in our database as a primary (web_charset) encoding this loop
     * does an attempt to 'approximate' charset name by header_charset.
     */
    i = 0;
    while i < mlang_data.len() {
        let mut n = 0;
        while n < mlang_data[i].number_of_cp as usize {
            let mut csetW = [0u16; MAX_MIMECSET_NAME];
            MultiByteToWideChar(
                CP_ACP,
                0,
                mlang_data[i].mime_cp_info[n].header_charset as *const u8,
                -1i32,
                csetW.as_mut_ptr(),
                MAX_MIMECSET_NAME as i32,
            );
            if lstrcmpiW(Charset, csetW.as_ptr()) == 0 {
                (*pCharsetInfo).uiCodePage = mlang_data[i].family_codepage;
                (*pCharsetInfo).uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
                lstrcpyW((*pCharsetInfo).wszCharset.as_mut_ptr(), csetW.as_ptr());
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }

    E_FAIL
}
unsafe fn fnIMultiLanguage3_IsConvertible(
    iface: *mut IMultiLanguage3,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
) -> i32 {
    unsafe { IsConvertINetStringAvailable(dwSrcEncoding, dwDstEncoding) }
}
unsafe fn fnIMultiLanguage3_ConvertString(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
    pSrcStr: *mut u8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u8,
    pcDstSize: *mut u32,
) -> HRESULT {
    ConvertINetString(
        pdwMode,
        dwSrcEncoding,
        dwDstEncoding,
        pSrcStr as *const i8,
        pcSrcSize as *mut i32,
        pDstStr as *mut i8,
        pcDstSize as *mut i32,
    )
}
unsafe fn fnIMultiLanguage3_ConvertStringToUnicode(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut i8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u16,
    pcDstSize: *mut u32,
) -> i32 {
    unsafe {
        ConvertINetMultiByteToUnicode(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr, pcDstSize as *mut i32)
    }
}
unsafe fn fnIMultiLanguage3_ConvertStringFromUnicode(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut u16,
    pcSrcSize: *mut u32,
    pDstStr: *mut i8,
    pcDstSize: *mut u32,
) -> i32 {
    ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr, pcDstSize as *mut i32)
}
unsafe fn fnIMultiLanguage3_ConvertStringReset(iface: *mut IMultiLanguage3) -> HRESULT {
    const E_NOTIMPL: HRESULT = 0x80004001u32 as i32;
    unsafe extern "C" {
        fn FIXME(format: *const i8, ...);
    }
    unsafe {
        FIXME(b"\n\0".as_ptr() as *const i8);
    }
    return E_NOTIMPL;
}
unsafe fn fnIMultiLanguage3_GetRfc1766FromLcid(
    iface: *mut IMultiLanguage3,
    lcid: u32,
    pbstrRfc1766: *mut *mut u16,
) -> i32 {
    const MAX_RFC1766_NAME: usize = 65;
    const E_INVALIDARG: i32 = 0x80070057u32 as i32;
    const E_FAIL: i32 = 0x80004005u32 as i32;
    const S_OK: i32 = 0;

    let mut buf: [u16; 65] = [0u16; 65];

    // TRACE("%p %04x %p\n", iface, lcid, pbstrRfc1766);
    let fmt = b"%p %04x %p\n\0".as_ptr() as *const i8;
    unsafe {
        TRACE(
            fmt,
            iface as *mut u8,
            lcid,
            pbstrRfc1766 as *mut u8,
        );
    }

    if pbstrRfc1766.is_null() {
        return E_INVALIDARG;
    }

    if lcid_to_rfc1766W(lcid, buf.as_mut_ptr(), MAX_RFC1766_NAME as u32) == 0 {
        let bstr = unsafe { SysAllocString(buf.as_ptr()) };
        unsafe { *pbstrRfc1766 = bstr; }
        return S_OK;
    }

    return E_FAIL;
}

const E_INVALIDARG: i32 = 0x80070057u32 as i32;
unsafe fn fnIMultiLanguage3_GetLcidFromRfc1766(
    iface: *mut IMultiLanguage3,
    pLocale: *mut u32,
    bstrRfc1766: *const u16,
) -> i32 {
    let mut hr: i32;
    let mut rfc1766: *mut IEnumRfc1766 = core::ptr::null_mut();

    TRACE(
        b"%p %p %s\n\0".as_ptr() as *const i8,
        iface as *mut core::ffi::c_void,
        pLocale as *mut core::ffi::c_void,
        debugstr_w(bstrRfc1766),
    );

    if pLocale.is_null() || bstrRfc1766.is_null() {
        return E_INVALIDARG;
    }

    hr = IMultiLanguage3_EnumRfc1766(iface, 0, core::ptr::addr_of_mut!(rfc1766));
    if hr < 0 {
        return hr;
    }

    hr = lcid_from_rfc1766(rfc1766, pLocale, bstrRfc1766);

    IEnumRfc1766_Release(rfc1766);
    return hr;
}
unsafe fn fnIMultiLanguage3_EnumRfc1766(
    iface: *mut IMultiLanguage3,
    LangId: LANGID,
    ppEnumRfc1766: *mut *mut IEnumRfc1766,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE(
        b"%p %p\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        ppEnumRfc1766 as *mut core::ffi::c_void,
    );
    return EnumRfc1766_create(LangId, ppEnumRfc1766);
}
unsafe fn fnIMultiLanguage3_GetRfc1766Info(
    iface: *mut IMultiLanguage3,
    Locale: LCID,
    LangId: LANGID,
    pRfc1766Info: *mut RFC1766INFO,
) -> HRESULT {
    unsafe {
        const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
        const E_FAIL: HRESULT = 0x80004005u32 as i32;
        const LOCALE_SLANGUAGE: LCTYPE = 2;
        const LOCALE_SENGLANGUAGE: LCTYPE = 0x1001;
        const MAX_LOCALE_NAME: i32 = 85;
        const MAX_RFC1766_NAME: usize = 65;

        static mut last_lang: LANGID = 0xFFFF;
        let mut type_: LCTYPE = LOCALE_SLANGUAGE;

        TRACE(
            b"(%p, 0x%04x, 0x%04x, %p)\n\0".as_ptr() as *const i8,
            iface,
            Locale,
            LangId,
            pRfc1766Info,
        );

        if pRfc1766Info.is_null() {
            return E_INVALIDARG;
        }

        let primary_locale = (Locale & 0x3FF) as LANGID;
        let sub_locale = ((Locale >> 10) & 0x3F) as LANGID;

        if primary_locale == LANG_ENGLISH
            || primary_locale == LANG_CHINESE
            || primary_locale == LANG_ARABIC
        {
            if sub_locale == 0 {
                type_ = LOCALE_SENGLANGUAGE;
            }
        } else {
            if sub_locale == 0 {
                TRACE(
                    b"SUBLANGID missing in 0x%04x\n\0".as_ptr() as *const i8,
                    Locale,
                );
                return E_FAIL;
            }
        }

        (*pRfc1766Info).lcid = Locale;
        (*pRfc1766Info).wszRfc1766[0] = 0;
        (*pRfc1766Info).wszLocaleName[0] = 0;

        let primary_langid = (LangId & 0x3FF) as LANGID;
        if primary_langid != LANG_ENGLISH && last_lang != LangId {
            FIXME(
                b"Only English names supported (requested: 0x%04x)\n\0".as_ptr() as *const i8,
                LangId,
            );
            last_lang = LangId;
        }

        if !lcid_to_rfc1766W(
            Locale,
            (*pRfc1766Info).wszRfc1766.as_mut_ptr(),
            MAX_RFC1766_NAME,
        ) && GetLocaleInfoW(
            Locale,
            type_,
            (*pRfc1766Info).wszLocaleName.as_mut_ptr(),
            MAX_LOCALE_NAME,
        ) > 0
        {
            return 0;
        }

        return E_INVALIDARG;
    }
}
unsafe fn fnIMultiLanguage3_CreateConvertCharset(
    iface: *mut IMultiLanguage3,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
    convert_charset: *mut *mut IMLangConvertCharset,
) -> HRESULT {
    let hr: HRESULT;

    TRACE(b"(%u %u 0x%08x %p)\n\0".as_ptr() as *const i8, src_cp, dst_cp, prop, convert_charset);

    hr = MLangConvertCharset_create(core::ptr::null_mut() as *const IUnknown, convert_charset as *mut *mut core::ffi::c_void);
    // FAILED(hr) equivalent: check if HRESULT is negative
    if hr < 0 { return hr; }

    // IMLangConvertCharset_Initialize(*convert_charset, src_cp, dst_cp, prop)
    // This is a COM vtable call; expand the macro faithfully
    let this: *mut IMLangConvertCharset = *convert_charset;
    let vtbl: *mut IMLangConvertCharsetVtbl = (*this).lpVtbl;
    return ((*vtbl).Initialize)(this, src_cp, dst_cp, prop);
}
unsafe fn fnIMultiLanguage3_ConvertStringInIStream(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwFlag: u32,
    lpFallBack: *const u16,
    dwSrcEncoding: u32,
    dwDstEncoding: u32,
    pstmIn: *mut IStream,
    pstmOut: *mut IStream,
) -> i32 {
    let mut src: *mut i8 = core::ptr::null_mut();
    let mut dst: *mut i8 = core::ptr::null_mut();
    let mut srclen: i32 = 0;
    let mut dstlen: i32 = 0;
    let mut stat: STATSTG = core::mem::zeroed();
    let mut hr: i32;

    TRACE(
        b"%p %0x8 %s %u %u %p %p\n\0".as_ptr() as *const i8,
        pdwMode as *mut core::ffi::c_void,
        dwFlag,
        debugstr_w(lpFallBack),
        dwSrcEncoding,
        dwDstEncoding,
        pstmIn as *mut core::ffi::c_void,
        pstmOut as *mut core::ffi::c_void,
    );

    FIXME(b"dwFlag and lpFallBack not handled\n\0".as_ptr() as *const i8);

    hr = IStream_Stat(pstmIn, &mut stat, STATFLAG_NONAME);
    if hr < 0 {
        return hr;
    }

    if stat.cbSize.QuadPart > MAXLONG {
        return E_INVALIDARG;
    }

    src = HeapAlloc(GetProcessHeap(), 0, stat.cbSize.QuadPart as usize) as *mut i8;
    if src.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = 'exit: loop {
        let mut tmp_hr;

        tmp_hr = IStream_Read(
            pstmIn,
            src as *mut core::ffi::c_void,
            stat.cbSize.QuadPart as u32,
            &mut srclen as *mut i32 as *mut u32,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        tmp_hr = ConvertINetString(
            pdwMode,
            dwSrcEncoding,
            dwDstEncoding,
            src,
            &mut srclen,
            core::ptr::null_mut(),
            &mut dstlen,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        dst = HeapAlloc(GetProcessHeap(), 0, dstlen as usize) as *mut i8;
        if dst.is_null() {
            tmp_hr = E_OUTOFMEMORY;
            break 'exit tmp_hr;
        }

        tmp_hr = ConvertINetString(
            pdwMode,
            dwSrcEncoding,
            dwDstEncoding,
            src,
            &mut srclen,
            dst,
            &mut dstlen,
        );
        if tmp_hr < 0 {
            break 'exit tmp_hr;
        }

        tmp_hr = IStream_Write(
            pstmOut,
            dst as *const core::ffi::c_void,
            dstlen as u32,
            core::ptr::null_mut(),
        );
        break 'exit tmp_hr;
    };

    HeapFree(GetProcessHeap(), 0, src as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, dst as *mut core::ffi::c_void);
    hr
}
unsafe fn fnIMultiLanguage3_ConvertStringToUnicodeEx(
    iface: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut i8,
    pcSrcSize: *mut u32,
    pDstStr: *mut u16,
    pcDstSize: *mut u32,
    dwFlag: u32,
    lpFallBack: *mut u16,
) -> HRESULT {
    if dwFlag != 0 || !lpFallBack.is_null() {
        FIXME(
            b"Ignoring dwFlag (0x%x/%d) and lpFallBack (%p)\n\0".as_ptr() as *const i8,
            dwFlag,
            dwFlag,
            lpFallBack,
        );
    }
    ConvertINetMultiByteToUnicode(
        pdwMode,
        dwEncoding,
        pSrcStr,
        pcSrcSize as *mut i32,
        pDstStr,
        pcDstSize as *mut i32,
    )
}
unsafe fn fnIMultiLanguage3_ConvertStringFromUnicodeEx(
    This: *mut IMultiLanguage3,
    pdwMode: *mut u32,
    dwEncoding: u32,
    pSrcStr: *mut u16,
    pcSrcSize: *mut u32,
    pDstStr: *mut i8,
    pcDstSize: *mut u32,
    dwFlag: u32,
    lpFallBack: *mut u16,
) -> HRESULT {
    FIXME(b"\n\0".as_ptr() as *const i8);
    ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding, pSrcStr, pcSrcSize as *mut i32, pDstStr, pcDstSize as *mut i32)
}

const E_NOTIMPL: i32 = 0x80004001u32 as i32;
unsafe fn fnIMultiLanguage3_DetectCodepageInIStream(
    iface: *mut IMultiLanguage3,
    dwFlag: u32,
    dwPrefWinCodePage: u32,
    pstmIn: *mut IStream,
    lpEncoding: *mut DetectEncodingInfo,
    pnScores: *mut i32,
) -> i32 {
    FIXME(b"\n\0".as_ptr() as *const i8);
    return E_NOTIMPL;
}

const E_NOTIMPL: HRESULT = 0x80004001;
unsafe fn fnIMultiLanguage3_DetectInputCodepage(
    iface: *mut IMultiLanguage3,
    dwFlag: DWORD,
    dwPrefWinCodePage: DWORD,
    pSrcStr: *mut i8,
    pcSrcSize: *mut i32,
    lpEncoding: *mut DetectEncodingInfo,
    pnScores: *mut i32,
) -> HRESULT {
    FIXME(b"\n\0".as_ptr().cast());
    return E_NOTIMPL;
}
unsafe fn fnIMultiLanguage3_ValidateCodePage(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    hwnd: isize,
) -> i32 {
    IMultiLanguage3_ValidateCodePageEx(iface, uiCodePage, hwnd, 0)
}

const CP_ACP: u32 = 0;
const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
unsafe fn fnIMultiLanguage3_GetCodePageDescription(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    lcid: u32,
    lpWideCharStr: *mut u16,
    cchWideChar: i32,
) -> i32 {
    let mut i: u32;
    let mut n: u32;

    TRACE(
        b"%u, %04x, %p, %d\n\0".as_ptr() as *const i8,
        uiCodePage,
        lcid,
        lpWideCharStr as *const core::ffi::c_void,
        cchWideChar,
    );

    i = 0;
    while (i as usize) < mlang_data.len() {
        n = 0;
        while n < mlang_data[i as usize].number_of_cp {
            if (*mlang_data[i as usize].mime_cp_info.add(n as usize)).cp == uiCodePage {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*mlang_data[i as usize].mime_cp_info.add(n as usize)).description,
                    -1,
                    lpWideCharStr,
                    cchWideChar,
                );
                return S_OK;
            }
            n = n + 1;
        }
        i = i + 1;
    }

    return S_FALSE;
}

const CPIOD_PEEK: u32 = 0x00000400; // CPIOD_PEEK
unsafe fn fnIMultiLanguage3_IsCodePageInstallable(iface: *mut IMultiLanguage3, uiCodePage: u32) -> HRESULT {
    TRACE(b"%u\n\0".as_ptr() as *const i8, uiCodePage);
    // FIXME: the installable set is usually larger than the set of valid codepages
    return IMultiLanguage3_ValidateCodePageEx(iface, uiCodePage, core::ptr::null(), CPIOD_PEEK);
}
unsafe fn fnIMultiLanguage3_SetMimeDBSource(
    iface: *mut IMultiLanguage3,
    dwSource: MIMECONTF,
) -> HRESULT {
    FIXME(
        b"0x%08x\n\0".as_ptr() as *const core::ffi::c_char,
        dwSource,
    );
    0 // S_OK
}
unsafe fn fnIMultiLanguage3_GetNumberOfScripts(iface: *mut IMultiLanguage3, pnScripts: *mut u32) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE(b"%p %p\n\0".as_ptr() as *const i8, This, pnScripts);
    if pnScripts.is_null() {
        return 1;
    }
    (*pnScripts) = (*This).total_scripts;
    return 0;
}
unsafe fn fnIMultiLanguage3_EnumScripts(
    iface: *mut IMultiLanguage3,
    dwFlags: DWORD,
    LangId: LANGID,
    ppEnumScript: *mut *mut IEnumScript,
) -> HRESULT {
    unsafe {
        let This = impl_from_IMultiLanguage3(iface);
        TRACE(b"%p %08x %04x %p\n\0".as_ptr() as *const i8, This, dwFlags, LangId as u32, ppEnumScript);
        return EnumScript_create(This, dwFlags, LangId, ppEnumScript);
    }
}

const CPIOD_PEEK: u32 = 2;
const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
unsafe fn fnIMultiLanguage3_ValidateCodePageEx(
    iface: *mut IMultiLanguage3,
    uiCodePage: u32,
    hwnd: HWND,
    dwfIODControl: u32,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);
    TRACE(b"%p %u %p %08x\n\0".as_ptr() as *const i8, This, uiCodePage, hwnd, dwfIODControl);
    if IsValidCodePage(uiCodePage) != 0 {
        return S_OK;
    }
    let mut i = 0u32;
    while i < mlang_data.len() as u32 {
        let mut n = 0u32;
        while n < mlang_data[i as usize].number_of_cp {
            if mlang_data[i as usize].mime_cp_info[n as usize].cp == uiCodePage {
                return S_OK;
            }
            n += 1;
        }
        i += 1;
    }
    if dwfIODControl != CPIOD_PEEK {
        FIXME(b"Request to install codepage language pack not handled\n\0".as_ptr() as *const i8);
    }
    S_FALSE
}

const E_INVALIDARG: i32 = 0x80070057u32 as i32;
const CP_UTF8: u32 = 65001;
const S_OK: i32 = 0;
unsafe fn fnIMultiLanguage3_DetectOutboundCodePage(
    iface: *mut IMultiLanguage3,
    dwFlags: u32,
    lpWideCharStr: *const u16,
    cchWideChar: u32,
    puiPreferredCodePages: *mut u32,
    nPreferredCodePages: u32,
    puiDetectedCodePages: *mut u32,
    pnDetectedCodePages: *mut u32,
    lpSpecialChar: *mut u16,
) -> i32 {
    let This: *mut MLang_impl = impl_from_IMultiLanguage3(iface);

    FIXME(
        b"(%p)->(%08x %s %p %u %p %p(%u) %s)\n\0".as_ptr() as *const i8,
        This,
        dwFlags,
        debugstr_w(lpWideCharStr),
        puiPreferredCodePages,
        nPreferredCodePages,
        puiDetectedCodePages,
        if pnDetectedCodePages.is_null() { 0 } else { *pnDetectedCodePages },
        debugstr_w(lpSpecialChar),
    );

    if puiDetectedCodePages.is_null() || pnDetectedCodePages.is_null() || *pnDetectedCodePages == 0 {
        return E_INVALIDARG;
    }

    *puiDetectedCodePages = CP_UTF8;
    *pnDetectedCodePages = 1;
    return S_OK;
}

const E_INVALIDARG: HRESULT = 0x80070057_u32 as i32;
const S_OK: HRESULT = 0;
const CP_UTF8: u32 = 65001;
unsafe fn fnIMultiLanguage3_DetectOutboundCodePageInIStream(
    iface: *mut IMultiLanguage3,
    dwFlags: u32,
    pStrIn: *mut IStream,
    puiPreferredCodePages: *mut u32,
    nPreferredCodePages: u32,
    puiDetectedCodePages: *mut u32,
    pnDetectedCodePages: *mut u32,
    lpSpecialChar: *mut u16,
) -> HRESULT {
    let This = impl_from_IMultiLanguage3(iface);
    // FIXME("(%p)->(%08x %p %p %u %p %p(%u) %s)\n", This, dwFlags, pStrIn,
    //       puiPreferredCodePages, nPreferredCodePages, puiDetectedCodePages,
    //       pnDetectedCodePages, pnDetectedCodePages ? *pnDetectedCodePages : 0,
    //       debugstr_w(lpSpecialChar));

    if puiDetectedCodePages.is_null() || pnDetectedCodePages.is_null() || *pnDetectedCodePages == 0 {
        return E_INVALIDARG;
    }

    *puiDetectedCodePages = CP_UTF8;
    *pnDetectedCodePages = 1;
    return S_OK;
}
unsafe fn impl_from_IMLangFontLink2(iface: *mut IMLangFontLink2) -> *mut MLang_impl {
    let offset = core::mem::offset_of!(MLang_impl, IMLangFontLink2_iface);
    (iface as *mut u8).sub(offset) as *mut MLang_impl
}
unsafe fn fnIMLangFontLink2_QueryInterface(
    iface: *mut IMLangFontLink2,
    riid: *const win32::core::GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> i32 {
    let This = impl_from_IMLangFontLink2(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid,
        ppvObject,
    );
}
unsafe fn fnIMLangFontLink2_AddRef(iface: *mut IMLangFontLink2) -> u32 {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}
unsafe fn fnIMLangFontLink2_Release(iface: *mut IMLangFontLink2) -> u32 {
    let This = impl_from_IMLangFontLink2(iface);
    IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface))
}

const S_OK: HRESULT = 0;
const E_INVALIDARG: HRESULT = 0x80070057_u32 as HRESULT;
const E_FAIL: HRESULT = 0x80004005_u32 as HRESULT;
unsafe fn fnIMLangFontLink2_GetStrCodePages(
    iface: *mut IMLangFontLink2,
    src: *const WCHAR,
    src_len: LONG,
    priority_cp: DWORD,
    codepages: *mut DWORD,
    ret_len: *mut LONG,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    let mut i: LONG;
    let mut cps: DWORD = 0;

    TRACE(b"(%p)->(%s:%d %x %p %p)\n\0".as_ptr() as *const i8, This, debugstr_wn(src, src_len), src_len, priority_cp, codepages, ret_len);

    if !codepages.is_null() {
        *codepages = 0;
    }
    if !ret_len.is_null() {
        *ret_len = 0;
    }

    if src.is_null() || src_len == 0 || src_len < 0 {
        return E_INVALIDARG;
    }

    i = 0;
    while i < src_len {
        let mut cp: DWORD;
        let ret: HRESULT;

        ret = IMLangFontLink2_GetCharCodePages(iface, *src.offset(i as isize), core::ptr::addr_of_mut!(cp));
        if ret != S_OK {
            return E_FAIL;
        }

        if cps == 0 {
            cps = cp;
        } else {
            cps &= cp;
        }

        if (priority_cp & cps) != 0 {
            break;
        }
        i += 1;
    }

    if !codepages.is_null() {
        *codepages = cps;
    }
    if !ret_len.is_null() {
        *ret_len = min(i + 1, src_len);
    }
    return S_OK;
}

const TCI_SRCCODEPAGE: u32 = 1;
const S_OK: i32 = 0;
const E_FAIL: i32 = 0x80004005u32 as i32;
unsafe fn fnIMLangFontLink2_CodePageToCodePages(iface: *mut IMLangFontLink2, codepage: u32, codepages: *mut u32) -> i32 {
    let This = impl_from_IMLangFontLink2(iface);
    let mut cs: CHARSETINFO = core::mem::zeroed();
    let mut rc: i32 = 0;

    TRACE(b"(%p)->(%u %p)\n\0".as_ptr() as *const i8, This, codepage, codepages);

    let src: *mut u32 = codepage as usize as *mut u32;
    rc = TranslateCharsetInfo(src, &mut cs as *mut CHARSETINFO, TCI_SRCCODEPAGE);
    if rc != 0 {
        *codepages = cs.fs.fsCsb[0];
        TRACE(b"resulting codepages 0x%x\n\0".as_ptr() as *const i8, *codepages);
        return S_OK;
    }

    TRACE(b"codepage not found\n\0".as_ptr() as *const i8);
    *codepages = 0;
    return E_FAIL;
}
unsafe fn fnIMLangFontLink2_CodePagesToCodePage(
    iface: *mut IMLangFontLink2,
    codepages: u32,
    def_codepage: u32,
    codepage: *mut u32,
) -> i32 {
    const TCI_SRCCODEPAGE: u32 = 0x00000001;
    const TCI_SRCFONTSIG: u32 = 0x00000002;
    const S_OK: i32 = 0;
    const E_FAIL: i32 = 0x80004005u32 as i32;

    unsafe {
        let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
        let mut mask: u32 = 0;
        let mut cs: CHARSETINFO = core::mem::zeroed();
        let mut rc: i32;
        let mut i: u32;

        TRACE(
            b"(%p)->(0x%x %u %p)\n\0" as *const u8 as *const i8,
            This,
            codepages,
            def_codepage,
            codepage,
        );

        *codepage = 0;

        rc = TranslateCharsetInfo(
            (def_codepage as usize as *const u32),
            &mut cs,
            TCI_SRCCODEPAGE,
        );
        if (rc != 0) && ((codepages & cs.fs.fsCsb[0]) != 0) {
            TRACE(b"Found Default Codepage\n\0" as *const u8 as *const i8);
            *codepage = def_codepage;
            return S_OK;
        }

        for i in 0..32 {
            mask = 1u32.wrapping_shl(i);
            if (codepages & mask) != 0 {
                let mut Csb: [u32; 2];
                Csb[0] = mask;
                Csb[1] = 0x0;
                rc = TranslateCharsetInfo(Csb.as_mut_ptr(), &mut cs, TCI_SRCFONTSIG);
                if rc == 0 {
                    continue;
                }
                TRACE(
                    b"Falling back to least significant found CodePage %u\n\0" as *const u8 as *const i8,
                    cs.ciACP,
                );
                *codepage = cs.ciACP;
                return S_OK;
            }
        }

        TRACE(b"no codepage found\n\0" as *const u8 as *const i8);
        return E_FAIL;
    }
}
unsafe fn fnIMLangFontLink2_GetFontCodePages(
    iface: *mut IMLangFontLink2,
    hdc: HDC,
    hfont: HFONT,
    codepages: *mut DWORD,
) -> HRESULT {
    let This: *mut MLang_impl = impl_from_IMLangFontLink2(iface);
    let fontsig: core::mem::MaybeUninit<FONTSIGNATURE> = core::mem::MaybeUninit::uninit();
    let old_font: HFONT;

    TRACE(
        b"(%p)->(%p %p %p)\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        hdc as *const core::ffi::c_void,
        hfont as *const core::ffi::c_void,
        codepages as *const core::ffi::c_void,
    );

    old_font = SelectObject(hdc, hfont as *mut core::ffi::c_void) as HFONT;
    GetTextCharsetInfo(hdc, fontsig.as_mut_ptr(), 0);
    SelectObject(hdc, old_font as *mut core::ffi::c_void);

    let fontsig = fontsig.assume_init();
    *codepages = fontsig.fsCsb[0];
    TRACE(
        b"ret 0x%x\n\0".as_ptr() as *const i8,
        fontsig.fsCsb[0],
    );

    return 0; // S_OK
}
unsafe fn fnIMLangFontLink2_ReleaseFont(
    This: *mut IMLangFontLink2,
    hFont: HFONT,
) -> HRESULT {
    TRACE(
        b"(%p)->%p\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        hFont as *const core::ffi::c_void,
    );
    release_font(hFont)
}
unsafe fn fnIMLangFontLink2_ResetFontMapping(
    This: *mut IMLangFontLink2,
) -> HRESULT {
    TRACE(
        b"(%p)\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
    );

    clear_font_cache()
}
unsafe fn fnIMLangFontLink2_MapFont(
    This: *mut IMLangFontLink2,
    hDC: *mut c_void,
    dwCodePages: u32,
    chSrc: u16,
    pFont: *mut *mut c_void,
) -> i32 {
    let mut old_font: *mut c_void = core::ptr::null_mut();
    let fmt1 = b"(%p)->%p %08x %04x %p\n\0".as_ptr() as *const i8;
    TRACE(fmt1, This as *const c_void, hDC as *const c_void, dwCodePages, chSrc as u32, pFont as *const c_void);

    if hDC.is_null() {
        return E_FAIL;
    }

    if dwCodePages != 0 {
        old_font = GetCurrentObject(hDC, OBJ_FONT);
        return map_font(hDC, dwCodePages, old_font, pFont);
    } else {
        if pFont.is_null() {
            return E_INVALIDARG;
        }
        let fixme_str = b"the situation where dwCodepages is set to zero is not implemented\n\0".as_ptr() as *const i8;
        FIXME(fixme_str);
        return E_FAIL;
    }
}

const E_INVALIDARG: HRESULT = 0x80070057i32;
const E_FAIL: HRESULT = 0x80004005i32;
const E_OUTOFMEMORY: HRESULT = 0x8007000Ei32;
const S_OK: HRESULT = 0;
unsafe fn fnIMLangFontLink2_GetFontUnicodeRanges(
    This: *mut IMLangFontLink2,
    hDC: HDC,
    puiRanges: *mut u32,
    pUranges: *mut UNICODERANGE,
) -> HRESULT {
    let mut size: u32;
    let mut gs: *mut GLYPHSET;

    TRACE(
        b"(%p)->%p %p %p\n\0".as_ptr() as *const i8,
        This as *mut core::ffi::c_void,
        hDC as *mut core::ffi::c_void,
        puiRanges as *mut core::ffi::c_void,
        pUranges as *mut core::ffi::c_void,
    );

    if puiRanges.is_null() {
        return E_INVALIDARG;
    }
    size = GetFontUnicodeRanges(hDC, core::ptr::null_mut());
    if size == 0 {
        return E_FAIL;
    }
    gs = HeapAlloc(GetProcessHeap(), 0, size) as *mut GLYPHSET;
    if gs.is_null() {
        return E_OUTOFMEMORY;
    }

    GetFontUnicodeRanges(hDC, gs);
    *puiRanges = (*gs).cRanges;
    if !pUranges.is_null() {
        let mut i: u32 = 0;
        while i < (*gs).cRanges {
            if i >= *puiRanges {
                break;
            }
            let pUr = &mut *pUranges.offset(i as isize);
            pUr.wcFrom = (*gs).ranges[i as usize].wcLow;
            pUr.wcTo = (*gs).ranges[i as usize].wcLow + (*gs).ranges[i as usize].cGlyphs;
            i += 1;
        }
        *puiRanges = i;
    }
    HeapFree(GetProcessHeap(), 0, gs as *mut core::ffi::c_void);
    S_OK
}

const CP_ACP: UINT = 0;
const MAX_MIMEFACE_NAME: i32 = 32;
const SCRIPTCONTF_FIXED_FONT: DWORD = 1;
const SCRIPTCONTF_PROPORTIONAL_FONT: DWORD = 2;
const S_OK: HRESULT = 0;
unsafe fn fnIMLangFontLink2_GetScriptFontInfo(
    This: *mut IMLangFontLink2,
    sid: SCRIPT_ID,
    dwFlags: DWORD,
    puiFonts: *mut UINT,
    pScriptFont: *mut SCRIPTFONTINFO,
) -> HRESULT {
    let mut i: UINT;
    let mut j: UINT;

    TRACE(
        b"(%p)->%u %x %p %p\n\0".as_ptr() as *const i8,
        This,
        sid,
        dwFlags,
        puiFonts,
        pScriptFont,
    );

    if dwFlags == 0 {
        dwFlags = SCRIPTCONTF_PROPORTIONAL_FONT;
    }

    i = 0;
    j = 0;
    while i < mlang_data.len() as UINT {
        if sid == mlang_data[i as usize].sid {
            if !pScriptFont.is_null() {
                if j >= *puiFonts {
                    break;
                }

                (*pScriptFont.add(j as usize)).scripts = (1u32 << mlang_data[i as usize].sid) as u32; // SCRIPT_IDS cast

                if dwFlags == SCRIPTCONTF_FIXED_FONT {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        mlang_data[i as usize].fixed_font,
                        -1i32,
                        (*pScriptFont.add(j as usize)).wszFont.as_mut_ptr() as *mut u16,
                        MAX_MIMEFACE_NAME,
                    );
                } else if dwFlags == SCRIPTCONTF_PROPORTIONAL_FONT {
                    MultiByteToWideChar(
                        CP_ACP,
                        0,
                        mlang_data[i as usize].proportional_font,
                        -1i32,
                        (*pScriptFont.add(j as usize)).wszFont.as_mut_ptr() as *mut u16,
                        MAX_MIMEFACE_NAME,
                    );
                }
            }
            j += 1;
        }
        i += 1;
    }

    *puiFonts = j;
    return S_OK;
}
unsafe fn fnIMLangFontLink2_CodePageToScriptID(
    This: *mut IMLangFontLink2,
    uiCodePage: u32,
    pSid: *mut u32,
) -> i32 {
    unsafe {
        TRACE(b"(%p)->%i %p\n\0".as_ptr() as *const i8, This, uiCodePage, pSid);
    }
    if uiCodePage == 1200 /* CP_UNICODE */ {
        return -2147467259i32; /* E_FAIL */
    }
    for i in 0..mlang_data.len() as u32 {
        if uiCodePage == mlang_data[i as usize].family_codepage {
            if !pSid.is_null() {
                unsafe {
                    *pSid = mlang_data[i as usize].sid;
                }
            }
            return 0; /* S_OK */
        }
    }
    return -2147467259i32; /* E_FAIL */
}

unsafe fn impl_from_IMLangLineBreakConsole(iface: *mut IMLangLineBreakConsole) -> *mut MLang_impl {
    let offset = core::mem::offset_of!(MLang_impl, IMLangLineBreakConsole_iface) as isize;
    let base = (iface as *mut u8).offset(-offset);
    base as *mut MLang_impl
}
unsafe fn fnIMLangLineBreakConsole_QueryInterface(
    iface: *mut IMLangLineBreakConsole,
    riid: *const c_void,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMLangLineBreakConsole(iface);
    return IMultiLanguage3_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface),
        riid,
        ppvObject,
    );
}
unsafe fn fnIMLangLineBreakConsole_AddRef(iface: *mut IMLangLineBreakConsole) -> u32 {
    let this: *mut MLang_impl = impl_from_IMLangLineBreakConsole(iface);
    IMultiLanguage3_AddRef(core::ptr::addr_of_mut!((*this).IMultiLanguage3_iface))
}
unsafe fn fnIMLangLineBreakConsole_Release(
    iface: *mut IMLangLineBreakConsole,
) -> u32 {
    let This = impl_from_IMLangLineBreakConsole(iface);
    return IMultiLanguage3_Release(core::ptr::addr_of_mut!((*This).IMultiLanguage3_iface));
}

const E_NOTIMPL: i32 = 0x80004001u32 as i32;
unsafe fn fnIMLangLineBreakConsole_BreakLineML(
    iface: *mut IMLangLineBreakConsole,
    pSrcMLStr: *mut IMLangString,
    lSrcPos: i32,
    lSrcLen: i32,
    cMinColumns: i32,
    cMaxColumns: i32,
    plLineLen: *mut i32,
    plSkipLen: *mut i32,
) -> i32 {
    let fmt = b"(%p)->%p %i %i %i %i %p %p\n\0".as_ptr() as *const i8;
    FIXME(fmt, iface, pSrcMLStr, lSrcPos, lSrcLen, cMinColumns, cMaxColumns, plLineLen, plSkipLen);
    return E_NOTIMPL;
}
unsafe fn fnIMLangLineBreakConsole_BreakLineW(
    iface: *mut IMLangLineBreakConsole,
    locale: LCID,
    pszSrc: *const u16,
    cchSrc: i32,
    cMaxColumns: i32,
    pcchLine: *mut i32,
    pcchSkip: *mut i32,
) -> i32 {

    unsafe {
        FIXME(
            b"(%p)->%i %s %i %i %p %p\n\0".as_ptr() as *const i8,
            iface,
            locale,
            debugstr_wn(pszSrc, cchSrc),
            cchSrc,
            cMaxColumns,
            pcchLine,
            pcchSkip,
        );

        *pcchLine = cchSrc;
        *pcchSkip = 0;
        0 // S_OK
    }
}

const CP_USASCII: u32 = 20127;
unsafe fn fnIMLangLineBreakConsole_BreakLineA(
    iface: *mut IMLangLineBreakConsole,
    locale: u32,
    uCodePage: u32,
    pszSrc: *const i8,
    cchSrc: i32,
    cMaxColumns: i32,
    pcchLine: *mut i32,
    pcchSkip: *mut i32,
) -> i32 {
    let mut line: i32 = cchSrc;
    let mut skip: i32 = 0;
    FIXME(
        b"(%p)->%i %i %s %i %i %p %p\n\0".as_ptr() as *const i8,
        iface,
        locale,
        uCodePage,
        debugstr_an(pszSrc, cchSrc),
        cchSrc,
        cMaxColumns,
        pcchLine,
        pcchSkip,
    );
    if uCodePage == CP_USASCII && cchSrc > cMaxColumns {
        line = cMaxColumns;
        let mut i: i32 = cMaxColumns - 1;
        while i >= 0 {
            if *pszSrc.add(i as usize) == b' ' as i8 {
                while i >= 0 && *pszSrc.add(i as usize) == b' ' as i8 {
                    i -= 1;
                    line -= 1;
                    skip += 1;
                }
                break;
            }
            i -= 1;
        }
    }
    *pcchLine = line;
    *pcchSkip = skip;
    0 // S_OK
}

unsafe fn impl_from_IMLangConvertCharset(iface: *mut IMLangConvertCharset) -> *mut convert_charset {
    let offset = core::mem::offset_of!(convert_charset, IMLangConvertCharset_iface);
    (iface as *mut u8).sub(offset) as *mut convert_charset
}
unsafe fn MLangConvertCharset_QueryInterface(
    iface: *mut IMLangConvertCharset,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);

    TRACE(
        b"(%p)->(%s %p)\n\0".as_ptr() as *const i8,
        This as *const core::ffi::c_void,
        debugstr_guid(riid),
        obj as *const core::ffi::c_void,
    );

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID(riid, core::ptr::addr_of!(IID_IMLangConvertCharset)) != 0
    {
        *obj = core::ptr::addr_of_mut!((*This).IMLangConvertCharset_iface) as *mut core::ffi::c_void;
        IMLangConvertCharset_AddRef(iface);
        return 0; // S_OK
    }

    *obj = core::ptr::null_mut();
    0x80004002u32 as i32 // E_NOINTERFACE
}
unsafe fn MLangConvertCharset_AddRef(iface: *mut IMLangConvertCharset) -> u32 {
    let This = impl_from_IMLangConvertCharset(iface);
    let ref = ::ntdll_sys::nt::InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref)) as u32;
    TRACE(b"(%p)->(%u)\n\0".as_ptr() as *const i8, This as *const core::ffi::c_void, ref);
    ref
}
unsafe fn MLangConvertCharset_Release(iface: *mut IMLangConvertCharset) -> u32 {
    let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
    let ref_: u32 = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref)) as u32;
    TRACE(b"(%p)->(%u)\n\0".as_ptr() as *const core::ffi::c_char, This, ref_);
    if ref_ == 0 {
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
        UnlockModule();
    }
    ref_
}
unsafe fn MLangConvertCharset_Initialize(
    iface: *mut IMLangConvertCharset,
    src_cp: u32,
    dst_cp: u32,
    prop: u32,
) -> HRESULT {
    const MLCONVCHARF_USEDEFCHAR: u32 = 0x20000000;
    const S_OK: HRESULT = 0;

    unsafe {
        let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
        TRACE(
            b"(%p)->(%u %u 0x%08x)\n\0".as_ptr() as *const i8,
            This as *mut core::ffi::c_void,
            src_cp,
            dst_cp,
            prop,
        );
        let mut prop = prop;
        prop &= !MLCONVCHARF_USEDEFCHAR;
        if prop != 0 {
            FIXME(
                b"property 0x%08x not supported\n\0".as_ptr() as *const i8,
                prop,
            );
        }
        (*This).src_cp = src_cp;
        (*This).dst_cp = dst_cp;
        return S_OK;
    }
}

const E_INVALIDARG: i32 = 0x80070057_u32 as i32;
const S_OK: i32 = 0;
unsafe fn MLangConvertCharset_GetSourceCodePage(
    iface: *mut IMLangConvertCharset,
    src_cp: *mut u32,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);

    TRACE(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, src_cp);

    if src_cp.is_null() {
        return E_INVALIDARG;
    }
    *src_cp = (*This).src_cp;
    return S_OK;
}

const E_INVALIDARG: i32 = 0x80070057;
unsafe fn MLangConvertCharset_GetDestinationCodePage(iface: *mut IMLangConvertCharset, dst_cp: *mut u32) -> i32 {
    let this = impl_from_IMLangConvertCharset(iface);
    TRACE(b"(%p)->(%p)\n\0".as_ptr() as *const i8, this as *const core::ffi::c_void, dst_cp as *const core::ffi::c_void);
    if dst_cp.is_null() {
        return E_INVALIDARG;
    }
    *dst_cp = (*this).dst_cp;
    return 0;
}

const E_NOTIMPL: i32 = 0x80004001;
unsafe fn MLangConvertCharset_GetProperty(iface: *mut IMLangConvertCharset, prop: *mut DWORD) -> HRESULT {
    let This = impl_from_IMLangConvertCharset(iface);
    FIXME(b"(%p)->(%p): stub\n\0".as_ptr() as *const i8, This as *mut core::ffi::c_void, prop as *mut core::ffi::c_void);
    return E_NOTIMPL;
}

const E_NOTIMPL: i32 = 0x80004001u32 as i32;
unsafe fn MLangConvertCharset_DoConversion(
    iface: *mut IMLangConvertCharset,
    src: *mut u8,
    src_size: *mut u32,
    dest: *mut u8,
    dest_size: *mut u32,
) -> i32 {
    let This = impl_from_IMLangConvertCharset(iface);
    FIXME(
        b"(%p)->(%p %p %p %p): stub\n\0".as_ptr() as *const i8,
        This,
        src,
        src_size,
        dest,
        dest_size,
    );
    return E_NOTIMPL;
}
unsafe fn MLangConvertCharset_DoConversionToUnicode(
    iface: *mut IMLangConvertCharset,
    src: *mut i8,
    src_size: *mut u32,
    dest: *mut u16,
    dest_size: *mut u32,
) -> i32 {
    unsafe {
        let This = impl_from_IMLangConvertCharset(iface);
        TRACE(
            b"(%p)->(%p %p %p %p)\n\0".as_ptr() as *const i8,
            This,
            src,
            src_size,
            dest,
            dest_size,
        );
        ConvertINetMultiByteToUnicode(
            core::ptr::null_mut(),
            (*This).src_cp,
            src,
            src_size as *mut i32,
            dest,
            dest_size as *mut i32,
        )
    }
}
unsafe fn MLangConvertCharset_DoConversionFromUnicode(
    iface: *mut IMLangConvertCharset,
    src: *mut u16,
    src_size: *mut u32,
    dest: *mut i8,
    dest_size: *mut u32,
) -> HRESULT {
    let This: *mut convert_charset = impl_from_IMLangConvertCharset(iface);
    TRACE(b"(%p)->(%p %p %p %p)\n\0".as_ptr() as *const i8, This, src, src_size, dest, dest_size);
    return ConvertINetUnicodeToMultiByte(
        core::ptr::null(),
        (*This).dst_cp,
        src,
        src_size as *mut i32,
        dest,
        dest_size as *mut i32,
    );
}

const CLASS_E_NOAGGREGATION: i32 = -2147221240i32; // 0x80040110
const S_OK: i32 = 0;
unsafe fn MultiLanguage_create(pUnkOuter: *mut IUnknown, ppObj: *mut *mut core::ffi::c_void) -> HRESULT {
    TRACE(b"Creating MultiLanguage object\n\0".as_ptr().cast());
    if !pUnkOuter.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let mlang = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MLang_impl>()) as *mut MLang_impl;
    (*mlang).IMLangFontLink_iface.lpVtbl = core::ptr::addr_of!(IMLangFontLink_vtbl);
    (*mlang).IMultiLanguage_iface.lpVtbl = core::ptr::addr_of!(IMultiLanguage_vtbl);
    (*mlang).IMultiLanguage3_iface.lpVtbl = core::ptr::addr_of!(IMultiLanguage3_vtbl);
    (*mlang).IMLangFontLink2_iface.lpVtbl = core::ptr::addr_of!(IMLangFontLink2_vtbl);
    (*mlang).IMLangLineBreakConsole_iface.lpVtbl = core::ptr::addr_of!(IMLangLineBreakConsole_vtbl);

    (*mlang).total_cp = 0;
    let mut i: u32 = 0;
    while i < mlang_data.len() as u32 {
        (*mlang).total_cp += mlang_data[i as usize].number_of_cp;
        i += 1;
    }

    (*mlang).total_scripts = (mlang_data.len() - 1) as u32;

    (*mlang).ref = 1;
    *ppObj = core::ptr::addr_of!((*mlang).IMultiLanguage_iface) as *mut core::ffi::c_void;
    TRACE(b"returning %p\n\0".as_ptr().cast(), mlang as *mut core::ffi::c_void);

    LockModule();

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if dll_count == 0 { 0 /* S_OK */ } else { 1 /* S_FALSE */ }
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    if register_codepages() == 0 {
        return E_FAIL;
    }
    __wine_register_resources(instance)
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    unsafe {
        __wine_unregister_resources(instance)
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetGlobalFontLinkObject(
    unknown: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if unknown.is_null() {
        return E_INVALIDARG;
    }

    FIXME(
        b"%p: stub\n\0".as_ptr() as *const u8,
        unknown as *const core::ffi::c_void,
    );

    return S_FALSE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    MLANG Class Factory
//  *
//  * Copyright 2002 Lionel Ulmer
//  * Copyright 2003,2004 Mike McCormack
//  * Copyright 2004,2005 Dmitry Timoshkov
//  * Copyright 2009 Detlef Riekenberg
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
//
// #include <stdarg.h>
// #include <stdio.h>
//
// #define COBJMACROS
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "ole2.h"
// #include "objbase.h"
// #include "rpcproxy.h"
// #include "mlang.h"
// #include "mimeole.h"
//
// #include "wine/debug.h"
// #include "wine/list.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(mlang);
//
// #include "initguid.h"
//
// static HRESULT MultiLanguage_create(IUnknown *pUnkOuter, LPVOID *ppObj);
// static HRESULT MLangConvertCharset_create(IUnknown *outer, void **obj);
// static HRESULT EnumRfc1766_create(LANGID LangId, IEnumRfc1766 **ppEnum);
//
// static HINSTANCE instance;
// static DWORD MLANG_tls_index; /* to store various per thead data */
//
// /* FIXME:
//  * Under what circumstances HKEY_CLASSES_ROOT\MIME\Database\Codepage and
//  * HKEY_CLASSES_ROOT\MIME\Database\Charset are used?
//  */
//
// typedef struct
// {
//     const char *description;
//     UINT cp;
//     DWORD flags;
//     const char *web_charset;
//     const char *header_charset;
//     const char *body_charset;
//     const WCHAR *alias;
// } MIME_CP_INFO;
//
// /* These data are based on the codepage info in libs/unicode/cpmap.pl */
// /* FIXME: Add 28604 (Celtic), 28606 (Balkan) */
//
// static const MIME_CP_INFO arabic_cp[] =
// {
//     { "Arabic (864)",
//       864, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "ibm864", "ibm864", "ibm864" },
//     { "Arabic (1006)",
//       1006, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//             MIMECONTF_MIME_LATEST,
//       "ibm1006", "ibm1006", "ibm1006" },
//     { "Arabic (Windows)",
//       1256, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1256", "windows-1256", "windows-1256" },
//     { "Arabic (ISO)",
//       28596, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-6", "iso-8859-6", "iso-8859-6" }
// };
// static const MIME_CP_INFO baltic_cp[] =
// {
//     { "Baltic (DOS)",
//       775, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm775", "ibm775", "ibm775" },
//     { "Baltic (Windows)",
//       1257, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//             MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//             MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//             MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1257", "windows-1257", "windows-1257" },
//     { "Baltic (ISO)",
//       28594, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//              MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "iso-8859-4", "iso-8859-4", "iso-8859-4" },
//     { "Estonian (ISO)",
//       28603, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "iso-8859-13", "iso-8859-13", "iso-8859-13" }
// };
// static const MIME_CP_INFO chinese_simplified_cp[] =
// {
//     { "Chinese Simplified (Auto-Select)",
//       50936, MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "_autodetect_chs", "_autodetect_chs", "_autodetect_chs" },
//     { "Chinese Simplified (GB2312)",
//       936, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//            MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "gb2312", "gb2312", "gb2312" },
//     { "Chinese Simplified (GB2312-80)",
//       20936, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "x-cp20936", "x-cp20936", "x-cp20936" },
//     { "Chinese Simplified (HZ)",
//       52936, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 |
//              MIMECONTF_MIME_LATEST,
//       "hz-gb-2312", "hz-gb-2312", "hz-gb-2312" },
//     { "Chinese Simplified (GB18030)",
//       54936, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//              MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "GB18030", "GB18030", "GB18030" },
//     { "Chinese Simplified (GBK)",
//       936, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//            MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "gbk", "gbk", "gbk" }
// };
// static const MIME_CP_INFO chinese_traditional_cp[] =
// {
//     { "Chinese Traditional (Auto-Select)",
//       50950, MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "_autodetect_cht", "_autodetect_cht", "_autodetect_cht" },
//     { "Chinese Traditional (Big5)",
//       950, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//            MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "big5", "big5", "big5" },
//     { "Chinese Traditional (CNS)",
//       20000, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "x-Chinese-CNS", "x-Chinese-CNS", "x-Chinese-CNS" }
// };
// static const MIME_CP_INFO central_european_cp[] =
// {
//     { "Central European (DOS)",
//       852, MIMECONTF_BROWSER | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_BROWSER |
//            MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "ibm852", "ibm852", "ibm852" },
//     { "Central European (Windows)",
//       1250, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//             MIMECONTF_MIME_LATEST,
//       "windows-1250", "windows-1250", "windows-1250" },
//     { "Central European (Mac)",
//       10029, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "x-mac-ce", "x-mac-ce", "x-mac-ce" },
//     { "Central European (ISO)",
//       28592, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-2", "iso-8859-2", "iso-8859-2" }
// };
// static const MIME_CP_INFO cyrillic_cp[] =
// {
//     { "OEM Cyrillic",
//       855, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm855", "ibm855", "ibm855" },
//     { "Cyrillic (DOS)",
//       866, MIMECONTF_BROWSER | MIMECONTF_IMPORT | MIMECONTF_SAVABLE_BROWSER |
//            MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 |
//            MIMECONTF_MIME_LATEST,
//       "cp866", "cp866", "cp866" },
//     { "Cyrillic (Windows)",
//       1251, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1251", "windows-1251", "windows-1251" },
//     { "Cyrillic (Mac)",
//       10007, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "x-mac-cyrillic", "x-mac-cyrillic", "x-mac-cyrillic" },
//     { "Cyrillic (KOI8-R)",
//       20866, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "koi8-r", "koi8-r", "koi8-r" },
//     { "Cyrillic (KOI8-U)",
//       21866, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "koi8-u", "koi8-u", "koi8-u" },
//     { "Cyrillic (ISO)",
//       28595, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-5", "iso-8859-5", "iso-8859-5" }
// };
// static const MIME_CP_INFO greek_cp[] =
// {
//     { "Greek (DOS)",
//       737, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "ibm737", "ibm737", "ibm737" },
//     { "Greek, Modern (DOS)",
//       869, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "ibm869", "ibm869", "ibm869" },
//     { "IBM EBCDIC (Greek Modern)",
//       875, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "cp875", "cp875", "cp875" },
//     { "Greek (Windows)",
//       1253, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1253", "windows-1253", "windows-1253" },
//     { "Greek (Mac)",
//       10006, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "x-mac-greek", "x-mac-greek", "x-mac-greek" },
//     { "Greek (ISO)",
//       28597, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-7", "iso-8859-7", "iso-8859-7" }
// };
// static const MIME_CP_INFO hebrew_cp[] =
// {
//     { "Hebrew (424)",
//       424, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "ibm424", "ibm424", "ibm424" },
//     { "Hebrew (856)",
//       856, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "cp856", "cp856", "cp856" },
//     { "Hebrew (DOS)",
//       862, MIMECONTF_BROWSER | MIMECONTF_MINIMAL | MIMECONTF_IMPORT |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "dos-862", "dos-862", "dos-862" },
//     { "Hebrew (Windows)",
//       1255, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1255", "windows-1255", "windows-1255" },
//     { "Hebrew (ISO-Visual)",
//       28598, MIMECONTF_BROWSER | MIMECONTF_MINIMAL | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-8", "iso-8859-8", "iso-8859-8" }
// };
// static const MIME_CP_INFO japanese_cp[] =
// {
//     { "Japanese (Auto-Select)",
//       50932, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "_autodetect", "_autodetect", "_autodetect" },
//     { "Japanese (EUC)",
//       51932, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "euc-jp", "euc-jp", "euc-jp" },
//     { "Japanese (JIS)",
//       50220, MIMECONTF_IMPORT | MIMECONTF_MAILNEWS | MIMECONTF_EXPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID_NLS |
//              MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST |
//              MIMECONTF_MIME_IE4,
//       "iso-2022-jp","iso-2022-jp","iso-2022-jp"},
//     { "Japanese (JIS 0208-1990 and 0212-1990)",
//       20932, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//              MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
//       "EUC-JP","EUC-JP","EUC-JP"},
//     { "Japanese (JIS-Allow 1 byte Kana)",
//       50221, MIMECONTF_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_SAVABLE_BROWSER |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID_NLS |
//              MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
//       "csISO2022JP","iso-2022-jp","iso-2022-jp"},
//     { "Japanese (JIS-Allow 1 byte Kana - SO/SI)",
//       50222, MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_VALID |
//              MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
//       "iso-2022-jp","iso-2022-jp","iso-2022-jp"},
//     { "Japanese (Mac)",
//       10001, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//              MIMECONTF_VALID | MIMECONTF_PRIVCONVERTER | MIMECONTF_MIME_LATEST,
//       "x-mac-japanese","x-mac-japanese","x-mac-japanese"},
//     { "Japanese (Shift-JIS)",
//       932, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//            MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "shift_jis", "iso-2022-jp", "iso-2022-jp" }
// };
// static const MIME_CP_INFO korean_cp[] =
// {
//     { "Korean",
//       949, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//            MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//            MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID_NLS |
//            MIMECONTF_MIME_LATEST,
//       "ks_c_5601-1987", "ks_c_5601-1987", "ks_c_5601-1987" }
// };
// static const MIME_CP_INFO thai_cp[] =
// {
//     { "Thai (Windows)",
//       874, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_MIME_LATEST,
//       "ibm-thai", "ibm-thai", "ibm-thai" }
// };
// static const MIME_CP_INFO turkish_cp[] =
// {
//     { "Turkish (DOS)",
//       857, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm857", "ibm857", "ibm857" },
//     { "IBM EBCDIC (Turkish Latin-5)",
//       1026, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//             MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm1026", "ibm1026", "ibm1026" },
//     { "Turkish (Windows)",
//       1254, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//             MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//             MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//             MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1254", "windows-1254", "windows-1254" },
//     { "Turkish (Mac)",
//       10081, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "x-mac-turkish", "x-mac-turkish", "x-mac-turkish" },
//     { "Latin 3 (ISO)",
//       28593, MIMECONTF_MAILNEWS | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "iso-8859-3", "iso-8859-3", "iso-8859-3" },
//     { "Turkish (ISO)",
//       28599, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//              MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//              MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "iso-8859-9", "iso-8859-9", "iso-8859-9" }
// };
// static const MIME_CP_INFO vietnamese_cp[] =
// {
//     { "Vietnamese (Windows)",
//       1258, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//             MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//             MIMECONTF_EXPORT | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 |
//             MIMECONTF_MIME_LATEST,
//       "windows-1258", "windows-1258", "windows-1258" }
// };
//
// static const WCHAR asciiW[] = {'a','s','c','i','i',0};
//
// static const MIME_CP_INFO western_cp[] =
// {
//     { "IBM EBCDIC (US-Canada)",
//       37, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//           MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm037", "ibm037", "ibm037" },
//     { "OEM United States",
//       437, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm437", "ibm437", "ibm437" },
//     { "IBM EBCDIC (International)",
//       500, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm500", "ibm500", "ibm500" },
//     { "Western European (DOS)",
//       850, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm850", "ibm850", "ibm850" },
//     { "Portuguese (DOS)",
//       860, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm860", "ibm860", "ibm860" },
//     { "Icelandic (DOS)",
//       861, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm861", "ibm861", "ibm861" },
//     { "French Canadian (DOS)",
//       863, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm863", "ibm863", "ibm863" },
//     { "Nordic (DOS)",
//       865, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//            MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "ibm865", "ibm865", "ibm865" },
//     { "Western European (Windows)",
//       1252, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_MINIMAL |
//             MIMECONTF_IMPORT | MIMECONTF_SAVABLE_MAILNEWS |
//             MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT | MIMECONTF_VALID |
//             MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "windows-1252", "windows-1252", "iso-8859-1" },
//     { "Western European (Mac)",
//       10000, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "macintosh", "macintosh", "macintosh" },
//     { "Icelandic (Mac)",
//       10079, MIMECONTF_IMPORT | MIMECONTF_EXPORT | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "x-mac-icelandic", "x-mac-icelandic", "x-mac-icelandic" },
//     { "US-ASCII",
//       20127, MIMECONTF_MAILNEWS | MIMECONTF_IMPORT | MIMECONTF_EXPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_VALID |
//              MIMECONTF_VALID_NLS | MIMECONTF_MIME_LATEST,
//       "us-ascii", "us-ascii", "us-ascii", asciiW },
//     { "Western European (ISO)",
//       28591, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//              MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "iso-8859-1", "iso-8859-1", "iso-8859-1" },
//     { "Latin 9 (ISO)",
//       28605, MIMECONTF_MAILNEWS | MIMECONTF_IMPORT |
//              MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//              MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//              MIMECONTF_MIME_LATEST,
//       "iso-8859-15", "iso-8859-15", "iso-8859-15" }
// };
// static const MIME_CP_INFO unicode_cp[] =
// {
//     { "Unicode",
//       CP_UNICODE, MIMECONTF_MINIMAL | MIMECONTF_IMPORT |
//                   MIMECONTF_SAVABLE_BROWSER | MIMECONTF_EXPORT |
//                   MIMECONTF_VALID | MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 |
//                   MIMECONTF_MIME_LATEST,
//       "unicode", "unicode", "unicode" },
//     { "Unicode (UTF-7)",
//       CP_UTF7, MIMECONTF_MAILNEWS | MIMECONTF_IMPORT |
//                MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_EXPORT | MIMECONTF_VALID |
//                MIMECONTF_VALID_NLS | MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "utf-7", "utf-7", "utf-7" },
//     { "Unicode (UTF-8)",
//       CP_UTF8, MIMECONTF_MAILNEWS | MIMECONTF_BROWSER | MIMECONTF_IMPORT |
//                MIMECONTF_SAVABLE_MAILNEWS | MIMECONTF_SAVABLE_BROWSER |
//                MIMECONTF_EXPORT | MIMECONTF_VALID | MIMECONTF_VALID_NLS |
//                MIMECONTF_MIME_IE4 | MIMECONTF_MIME_LATEST,
//       "utf-8", "utf-8", "utf-8" }
// };
//
// static const struct mlang_data
// {
//     const char *description;
//     UINT family_codepage;
//     UINT number_of_cp;
//     const MIME_CP_INFO *mime_cp_info;
//     const char *fixed_font;
//     const char *proportional_font;
//     SCRIPT_ID sid;
// } mlang_data[] =
// {
//     { "Arabic", 1256, ARRAY_SIZE(arabic_cp), arabic_cp,
//       "Simplified Arabic Fixed","Simplified Arabic", sidArabic },
//     { "Baltic",  1257, ARRAY_SIZE(baltic_cp), baltic_cp,
//       "Courier New","Arial", sidAsciiLatin },
//     { "Chinese Simplified", 936, ARRAY_SIZE(chinese_simplified_cp), chinese_simplified_cp,
//       "Simsun","Simsun", sidHan },
//     { "Chinese Traditional", 950, ARRAY_SIZE(chinese_traditional_cp), chinese_traditional_cp,
//       "MingLiu","New MingLiu", sidBopomofo },
//     { "Central European", 1250, ARRAY_SIZE(central_european_cp), central_european_cp,
//       "Courier New","Arial", sidAsciiLatin },
//     { "Cyrillic", 1251, ARRAY_SIZE(cyrillic_cp), cyrillic_cp,
//       "Courier New","Arial", sidCyrillic },
//     { "Greek", 1253, ARRAY_SIZE(greek_cp), greek_cp,
//       "Courier New","Arial", sidGreek },
//     { "Hebrew", 1255, ARRAY_SIZE(hebrew_cp), hebrew_cp,
//       "Miriam Fixed","David", sidHebrew },
//     { "Japanese", 932, ARRAY_SIZE(japanese_cp), japanese_cp,
//       "MS Gothic","MS PGothic", sidKana },
//     { "Korean", 949, ARRAY_SIZE(korean_cp), korean_cp,
//       "GulimChe","Gulim", sidHangul },
//     { "Thai", 874, ARRAY_SIZE(thai_cp), thai_cp,
//       "Tahoma","Tahoma", sidThai },
//     { "Turkish", 1254, ARRAY_SIZE(turkish_cp), turkish_cp,
//       "Courier New","Arial", sidAsciiLatin },
//     { "Vietnamese", 1258, ARRAY_SIZE(vietnamese_cp), vietnamese_cp,
//       "Courier New","Arial", sidAsciiLatin },
//     { "Western European", 1252, ARRAY_SIZE(western_cp), western_cp,
//       "Courier New","Arial", sidAsciiLatin },
//     { "Unicode", CP_UNICODE, ARRAY_SIZE(unicode_cp), unicode_cp,
//       "Courier New","Arial" }
// };
//
// struct font_list
// {
//     struct list list_entry;
//     HFONT base_font;
//     HFONT font;
//     UINT charset;
// };
//
// static struct list font_cache = LIST_INIT(font_cache);
// static CRITICAL_SECTION font_cache_critical;
// static CRITICAL_SECTION_DEBUG font_cache_critical_debug =
// {
//     0, 0, &font_cache_critical,
//     { &font_cache_critical_debug.ProcessLocksList, &font_cache_critical_debug.ProcessLocksList },
//     0, 0, { (DWORD_PTR)(__FILE__ ": font_cache_critical") }
// };
// static CRITICAL_SECTION font_cache_critical = { &font_cache_critical_debug, -1, 0, 0, 0, 0 };
//
// static void fill_cp_info(const struct mlang_data *ml_data, UINT index, MIMECPINFO *mime_cp_info);
//
// static LONG dll_count;
//
// /*
//  * Japanese Detection and Conversion Functions
//  */
//
// #define HANKATA(A)  ((A >= 161) && (A <= 223))
// #define ISEUC(A)    ((A >= 161) && (A <= 254))
// #define NOTEUC(A,B) (((A >= 129) && (A <= 159)) && ((B >= 64) && (B <= 160)))
// #define SJIS1(A)    (((A >= 129) && (A <= 159)) || ((A >= 224) && (A <= 239)))
// #define SJIS2(A)    ((A >= 64) && (A <= 252))
// #define ISMARU(A)   ((A >= 202) && (A <= 206))
// #define ISNIGORI(A) (((A >= 182) && (A <= 196)) || ((A >= 202) && (A <= 206)))
//
// static UINT DetectJapaneseCode(LPCSTR input, DWORD count)
// {
//     UINT code = 0;
//     DWORD i = 0;
//     unsigned char c1,c2;
//
//     while ((code == 0 || code == 51932) && i < count)
//     {
//         c1 = input[i];
//         if (c1 == 0x1b /* ESC */)
//         {
//             i++;
//             if (i >= count)
//                 return code;
//             c1 = input[i];
//             if (c1 == '$')
//             {
//                 i++;
//                 if (i >= count)
//                     return code;
//                 c1 = input[i];
//                 if (c1 =='B' || c1 == '@')
//                     code = 50220;
//             }
//             if (c1 == 'K')
//                 code = 50220;
//         }
//         else if (c1 >= 129)
//         {
//             i++;
//             if (i >= count)
//                 return code;
//             c2 = input[i];
//             if NOTEUC(c1,c2)
//                 code = 932;
//             else if (ISEUC(c1) && ISEUC(c2))
//                 code = 51932;
//             else if (((c1 == 142)) && HANKATA(c2))
//                 code = 51932;
//         }
//         i++;
//     }
//     return code;
// }
//
// static inline void jis2sjis(unsigned char *p1, unsigned char *p2)
// {
//     unsigned char c1 = *p1;
//     unsigned char c2 = *p2;
//     int row = c1 < 95 ? 112 : 176;
//     int cell = c1 % 2 ? 31 + (c2 > 95) : 126;
//
//     *p1 = ((c1 + 1) >> 1) + row;
//     *p2 = c2 + cell;
// }
//
// static inline void sjis2jis(unsigned char *p1, unsigned char *p2)
// {
//     unsigned char c1 = *p1;
//     unsigned char c2 = *p2;
//     int shift = c2 < 159;
//     int row = c1 < 160 ? 112 : 176;
//     int cell = shift ? (31 + (c2 > 127)): 126;
//
//     *p1 = ((c1 - row) << 1) - shift;
//     *p2 -= cell;
// }
//
// static int han2zen(unsigned char *p1, unsigned char *p2)
// {
//     BOOL maru = FALSE;
//     BOOL nigori = FALSE;
//     static const unsigned char char1[] = {129,129,129,129,129,131,131,131,131,
//         131,131,131,131,131,131,129,131,131,131,131,131,131,131,131,131,131,
//         131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,
//         131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,131,
//         131,129,129 };
//     static const unsigned char char2[] = {66,117,118,65,69,146,64,66,68,70,
//         72,131,133,135,98,91,65,67,69,71,73,74,76,78,80,82,84,86,88,90,92,94,
//         96,99,101,103,105,106,107,108,109,110,113,116,119,122,125,126,128,
//         129,130,132,134,136,137,138,139,140,141,143,147,74,75};
//
//     if (( *p2 == 222) && ((ISNIGORI(*p1) || (*p1 == 179))))
//             nigori = TRUE;
//     else if ((*p2 == 223) && (ISMARU(*p1)))
//             maru = TRUE;
//
//     if (*p1 >= 161 && *p1 <= 223)
//     {
//         unsigned char index = *p1 - 161;
//         *p1 = char1[index];
//         *p2 = char2[index];
//     }
//
//     if (maru || nigori)
//     {
//         if (nigori)
//         {
//             if (((*p2 >= 74) && (*p2 <= 103)) || ((*p2 >= 110) && (*p2 <= 122)))
//                 (*p2)++;
//             else if ((*p1 == 131) && (*p2 == 69))
//                 *p2 = 148;
//         }
//         else if ((maru) && ((*p2 >= 110) && (*p2 <= 122)))
//             *p2+= 2;
//
//         return 1;
//     }
//
//     return 0;
// }
//
//
// static UINT ConvertJIS2SJIS(LPCSTR input, DWORD count, LPSTR output)
// {
//     DWORD i = 0;
//     int j = 0;
//     unsigned char p2,p;
//     BOOL shifted = FALSE;
//
//     while (i < count)
//     {
//         p = input[i];
//         if (p == 0x1b /* ESC */)
//         {
//             i++;
//             if (i >= count)
//                 return 0;
//             p2 = input[i];
//             if (p2 == '$' || p2 =='(')
//                 i++;
//             if (p2 == 'K' || p2 =='$')
//                 shifted = TRUE;
//             else
//                 shifted = FALSE;
//         }
//         else
//         {
//             if (shifted)
//             {
//                 i++;
//                 if (i >= count)
//                     return 0;
//                 p2 = input[i];
//                 jis2sjis(&p,&p2);
//                 output[j++]=p;
//                 output[j++]=p2;
//             }
//             else
//             {
//                 output[j++] = p;
//             }
//         }
//         i++;
//     }
//     return j;
// }
//
// static inline int exit_shift(LPSTR out, int c)
// {
//     if (out)
//     {
//         out[c] = 0x1b;
//         out[c+1] = '(';
//         out[c+2] = 'B';
//     }
//     return 3;
// }
//
// static inline int enter_shift(LPSTR out, int c)
// {
//     if (out)
//     {
//         out[c] = 0x1b;
//         out[c+1] = '$';
//         out[c+2] = 'B';
//     }
//     return 3;
// }
//
//
// static UINT ConvertSJIS2JIS(LPCSTR input, DWORD count, LPSTR output)
// {
//     DWORD i = 0;
//     int j = 0;
//     unsigned char p2,p;
//     BOOL shifted = FALSE;
//
//     while (i < count)
//     {
//         p = input[i] & 0xff;
//         if (p == 10 || p == 13) /* NL and CR */
//         {
//             if (shifted)
//             {
//                 shifted = FALSE;
//                 j += exit_shift(output,j);
//             }
//             if (output)
//                 output[j++] = p;
//             else
//                 j++;
//         }
//         else
//         {
//             if (SJIS1(p))
//             {
//                 i++;
//                 if (i >= count)
//                     return 0;
//                 p2 = input[i] & 0xff;
//                 if (SJIS2(p2))
//                 {
//                     sjis2jis(&p,&p2);
//                     if (!shifted)
//                     {
//                         shifted = TRUE;
//                         j+=enter_shift(output,j);
//                     }
//                 }
//
//                 if (output)
//                 {
//                     output[j++]=p;
//                     output[j++]=p2;
//                 }
//                 else
//                     j+=2;
//             }
//             else
//             {
//                 if (HANKATA(p))
//                 {
//                     if ((i+1) >= count)
//                         return 0;
//                     p2 = input[i+1] & 0xff;
//                     i+=han2zen(&p,&p2);
//                     sjis2jis(&p,&p2);
//                     if (!shifted)
//                     {
//                         shifted = TRUE;
//                         j+=enter_shift(output,j);
//                     }
//                     if (output)
//                     {
//                         output[j++]=p;
//                         output[j++]=p2;
//                     }
//                     else
//                         j+=2;
//                 }
//                 else
//                 {
//                     if (shifted)
//                     {
//                         shifted = FALSE;
//                         j += exit_shift(output,j);
//                     }
//                     if (output)
//                         output[j++]=p;
//                     else
//                         j++;
//                 }
//             }
//         }
//         i++;
//     }
//     if (shifted)
//         j += exit_shift(output,j);
//     return j;
// }
//
// static UINT ConvertJISJapaneseToUnicode(LPCSTR input, DWORD count,
//                                         LPWSTR output, DWORD out_count)
// {
//     CHAR *sjis_string;
//     UINT rc = 0;
//     sjis_string = HeapAlloc(GetProcessHeap(),0,count);
//     rc = ConvertJIS2SJIS(input,count,sjis_string);
//     if (rc)
//     {
//         TRACE("%s\n",debugstr_an(sjis_string,rc));
//         if (output)
//             rc = MultiByteToWideChar(932,0,sjis_string,rc,output,out_count);
//         else
//             rc = MultiByteToWideChar(932,0,sjis_string,rc,0,0);
//     }
//     HeapFree(GetProcessHeap(),0,sjis_string);
//     return rc;
//
// }
//
// static UINT ConvertUnknownJapaneseToUnicode(LPCSTR input, DWORD count,
//                                             LPWSTR output, DWORD out_count)
// {
//     CHAR *sjis_string;
//     UINT rc = 0;
//     int code = DetectJapaneseCode(input,count);
//     TRACE("Japanese code %i\n",code);
//
//     switch (code)
//     {
//     case 0:
//         if (output)
//             rc = MultiByteToWideChar(CP_ACP,0,input,count,output,out_count);
//         else
//             rc = MultiByteToWideChar(CP_ACP,0,input,count,0,0);
//         break;
//
//     case 932:
//         if (output)
//             rc = MultiByteToWideChar(932,0,input,count,output,out_count);
//         else
//             rc = MultiByteToWideChar(932,0,input,count,0,0);
//         break;
//
//     case 51932:
//         if (output)
//             rc = MultiByteToWideChar(20932,0,input,count,output,out_count);
//         else
//             rc = MultiByteToWideChar(20932,0,input,count,0,0);
//         break;
//
//     case 50220:
//         sjis_string = HeapAlloc(GetProcessHeap(),0,count);
//         rc = ConvertJIS2SJIS(input,count,sjis_string);
//         if (rc)
//         {
//             TRACE("%s\n",debugstr_an(sjis_string,rc));
//             if (output)
//                 rc = MultiByteToWideChar(932,0,sjis_string,rc,output,out_count);
//             else
//                 rc = MultiByteToWideChar(932,0,sjis_string,rc,0,0);
//         }
//         HeapFree(GetProcessHeap(),0,sjis_string);
//         break;
//     }
//     return rc;
// }
//
// static UINT ConvertJapaneseUnicodeToJIS(LPCWSTR input, DWORD count,
//                                         LPSTR output, DWORD out_count)
// {
//     CHAR *sjis_string;
//     INT len;
//     UINT rc = 0;
//
//     len = WideCharToMultiByte(932,0,input,count,0,0,NULL,NULL);
//     sjis_string = HeapAlloc(GetProcessHeap(),0,len);
//     WideCharToMultiByte(932,0,input,count,sjis_string,len,NULL,NULL);
//     TRACE("%s\n",debugstr_an(sjis_string,len));
//
//     rc = ConvertSJIS2JIS(sjis_string, len, NULL);
//     if (out_count >= rc)
//     {
//         ConvertSJIS2JIS(sjis_string, len, output);
//     }
//     HeapFree(GetProcessHeap(),0,sjis_string);
//     return rc;
//
// }
//
// /*
//  * Dll lifetime tracking declaration
//  */
// static void LockModule(void)
// {
//     InterlockedIncrement(&dll_count);
// }
//
// static void UnlockModule(void)
// {
//     InterlockedDecrement(&dll_count);
// }
//
// BOOL WINAPI DllMain(HINSTANCE hInstDLL, DWORD fdwReason, LPVOID lpv)
// {
//     switch(fdwReason) {
//         case DLL_PROCESS_ATTACH:
//             instance = hInstDLL;
//             MLANG_tls_index = TlsAlloc();
//             DisableThreadLibraryCalls(hInstDLL);
// 	    break;
// 	case DLL_PROCESS_DETACH:
//             if (lpv) break;
//             TlsFree(MLANG_tls_index);
// 	    break;
//     }
//     return TRUE;
// }
//
// HRESULT WINAPI ConvertINetMultiByteToUnicode(
//     LPDWORD pdwMode,
//     DWORD dwEncoding,
//     LPCSTR pSrcStr,
//     LPINT pcSrcSize,
//     LPWSTR pDstStr,
//     LPINT pcDstSize)
// {
//     INT src_len = -1;
//
//     TRACE("%p %d %s %p %p %p\n", pdwMode, dwEncoding,
//           debugstr_a(pSrcStr), pcSrcSize, pDstStr, pcDstSize);
//
//     if (!pcDstSize)
//         return E_FAIL;
//
//     if (!pcSrcSize)
//         pcSrcSize = &src_len;
//
//     if (!*pcSrcSize)
//     {
//         *pcDstSize = 0;
//         return S_OK;
//     }
//
//     /* forwarding euc-jp to EUC-JP */
//     if (dwEncoding == 51932)
//         dwEncoding = 20932;
//
//     switch (dwEncoding)
//     {
//     case CP_UNICODE:
//         if (*pcSrcSize == -1)
//             *pcSrcSize = lstrlenW((LPCWSTR)pSrcStr);
//         *pcDstSize = min(*pcSrcSize, *pcDstSize);
//         *pcSrcSize *= sizeof(WCHAR);
//         if (pDstStr)
//             memmove(pDstStr, pSrcStr, *pcDstSize * sizeof(WCHAR));
//         break;
//
//     case 50220:
//     case 50221:
//     case 50222:
//         *pcDstSize = ConvertJISJapaneseToUnicode(pSrcStr,*pcSrcSize,pDstStr,*pcDstSize);
//         break;
//     case 50932:
//         *pcDstSize = ConvertUnknownJapaneseToUnicode(pSrcStr,*pcSrcSize,pDstStr,*pcDstSize);
//         break;
//
//     default:
//         if (*pcSrcSize == -1)
//             *pcSrcSize = lstrlenA(pSrcStr);
//
//         if (pDstStr)
//             *pcDstSize = MultiByteToWideChar(dwEncoding, 0, pSrcStr, *pcSrcSize, pDstStr, *pcDstSize);
//         else
//             *pcDstSize = MultiByteToWideChar(dwEncoding, 0, pSrcStr, *pcSrcSize, NULL, 0);
//         break;
//     }
//
//     if (!*pcDstSize)
//         return E_FAIL;
//
//     return S_OK;
// }
//
// HRESULT WINAPI ConvertINetUnicodeToMultiByte(
//     LPDWORD pdwMode,
//     DWORD dwEncoding,
//     LPCWSTR pSrcStr,
//     LPINT pcSrcSize,
//     LPSTR pDstStr,
//     LPINT pcDstSize)
// {
//     INT destsz, size;
//     INT src_len = -1;
//
//     TRACE("%p %d %s %p %p %p\n", pdwMode, dwEncoding,
//           debugstr_w(pSrcStr), pcSrcSize, pDstStr, pcDstSize);
//
//     if (!pcDstSize)
//         return S_OK;
//
//     if (!pcSrcSize)
//         pcSrcSize = &src_len;
//
//     destsz = (pDstStr) ? *pcDstSize : 0;
//     *pcDstSize = 0;
//
//     if (!pSrcStr || !*pcSrcSize)
//         return S_OK;
//
//     if (*pcSrcSize == -1)
//         *pcSrcSize = lstrlenW(pSrcStr);
//
//     /* forwarding euc-jp to EUC-JP */
//     if (dwEncoding == 51932)
//         dwEncoding = 20932;
//
//     if (dwEncoding == CP_UNICODE)
//     {
//         if (*pcSrcSize == -1)
//             *pcSrcSize = lstrlenW(pSrcStr);
//
//         size = min(*pcSrcSize, destsz) * sizeof(WCHAR);
//         if (pDstStr)
//             memmove(pDstStr, pSrcStr, size);
//
//         if (size >= destsz)
//             goto fail;
//     }
//     else if (dwEncoding == 50220 || dwEncoding == 50221 || dwEncoding == 50222)
//     {
//         size = ConvertJapaneseUnicodeToJIS(pSrcStr, *pcSrcSize, NULL, 0);
//         if (!size)
//             goto fail;
//
//         if (pDstStr)
//         {
//             size = ConvertJapaneseUnicodeToJIS(pSrcStr, *pcSrcSize, pDstStr,
//                                                destsz);
//             if (!size)
//                 goto fail;
//         }
//
//     }
//     else
//     {
//         size = WideCharToMultiByte(dwEncoding, 0, pSrcStr, *pcSrcSize,
//                                    NULL, 0, NULL, NULL);
//         if (!size)
//             goto fail;
//
//         if (pDstStr)
//         {
//             size = WideCharToMultiByte(dwEncoding, 0, pSrcStr, *pcSrcSize,
//                                        pDstStr, destsz, NULL, NULL);
//             if (!size)
//                 goto fail;
//         }
//     }
//
//     *pcDstSize = size;
//     return S_OK;
//
// fail:
//     *pcSrcSize = 0;
//     *pcDstSize = 0;
//     return E_FAIL;
// }
//
// HRESULT WINAPI ConvertINetString(
//     LPDWORD pdwMode,
//     DWORD dwSrcEncoding,
//     DWORD dwDstEncoding,
//     LPCSTR pSrcStr,
//     LPINT pcSrcSize,
//     LPSTR pDstStr,
//     LPINT pcDstSize
// )
// {
//     TRACE("%p %d %d %s %p %p %p\n", pdwMode, dwSrcEncoding, dwDstEncoding,
//           debugstr_a(pSrcStr), pcSrcSize, pDstStr, pcDstSize);
//
//     if (dwSrcEncoding == CP_UNICODE)
//     {
//         INT cSrcSizeW;
//         if (pcSrcSize && *pcSrcSize != -1)
//         {
//             cSrcSizeW = *pcSrcSize / sizeof(WCHAR);
//             pcSrcSize = &cSrcSizeW;
//         }
//         return ConvertINetUnicodeToMultiByte(pdwMode, dwDstEncoding, (LPCWSTR)pSrcStr, pcSrcSize, pDstStr, pcDstSize);
//     }
//     else if (dwDstEncoding == CP_UNICODE)
//     {
//         HRESULT hr = ConvertINetMultiByteToUnicode(pdwMode, dwSrcEncoding, pSrcStr, pcSrcSize, (LPWSTR)pDstStr, pcDstSize);
//         *pcDstSize *= sizeof(WCHAR);
//         return hr;
//     }
//     else
//     {
//         INT cDstSizeW;
//         LPWSTR pDstStrW;
//         HRESULT hr;
//
//         TRACE("convert %s from %d to %d\n", debugstr_a(pSrcStr), dwSrcEncoding, dwDstEncoding);
//
//         hr = ConvertINetMultiByteToUnicode(pdwMode, dwSrcEncoding, pSrcStr, pcSrcSize, NULL, &cDstSizeW);
//         if (hr != S_OK)
//             return hr;
//
//         pDstStrW = HeapAlloc(GetProcessHeap(), 0, cDstSizeW * sizeof(WCHAR));
//         hr = ConvertINetMultiByteToUnicode(pdwMode, dwSrcEncoding, pSrcStr, pcSrcSize, pDstStrW, &cDstSizeW);
//         if (hr == S_OK)
//             hr = ConvertINetUnicodeToMultiByte(pdwMode, dwDstEncoding, pDstStrW, &cDstSizeW, pDstStr, pcDstSize);
//
//         HeapFree(GetProcessHeap(), 0, pDstStrW);
//         return hr;
//     }
// }
//
// static HRESULT GetFamilyCodePage(
//     UINT uiCodePage,
//     UINT* puiFamilyCodePage)
// {
//     UINT i, n;
//
//     TRACE("%u %p\n", uiCodePage, puiFamilyCodePage);
//
//     if (!puiFamilyCodePage) return S_FALSE;
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].cp == uiCodePage)
//             {
//                 *puiFamilyCodePage = mlang_data[i].family_codepage;
//                 return S_OK;
//             }
//         }
//     }
//
//     return S_FALSE;
// }
//
// HRESULT WINAPI IsConvertINetStringAvailable(
//     DWORD dwSrcEncoding,
//     DWORD dwDstEncoding)
// {
//     UINT src_family, dst_family;
//
//     TRACE("%d %d\n", dwSrcEncoding, dwDstEncoding);
//
//     if (GetFamilyCodePage(dwSrcEncoding, &src_family) != S_OK ||
//         GetFamilyCodePage(dwDstEncoding, &dst_family) != S_OK)
//         return S_FALSE;
//
//     if (src_family == dst_family) return S_OK;
//
//     /* we can convert any codepage to/from unicode */
//     if (src_family == CP_UNICODE || dst_family == CP_UNICODE) return S_OK;
//
//     return S_FALSE;
// }
//
// static inline HRESULT lcid_to_rfc1766A( LCID lcid, LPSTR rfc1766, INT len )
// {
//     CHAR buffer[MAX_RFC1766_NAME];
//     INT n = GetLocaleInfoA(lcid, LOCALE_SISO639LANGNAME, buffer, MAX_RFC1766_NAME);
//     INT i;
//
//     if (n)
//     {
//         i = PRIMARYLANGID(lcid);
//         if ((((i == LANG_ENGLISH) || (i == LANG_CHINESE) || (i == LANG_ARABIC)) &&
//             (SUBLANGID(lcid) == SUBLANG_DEFAULT)) ||
//             (SUBLANGID(lcid) > SUBLANG_DEFAULT)) {
//
//             buffer[n - 1] = '-';
//             i = GetLocaleInfoA(lcid, LOCALE_SISO3166CTRYNAME, buffer + n, MAX_RFC1766_NAME - n);
//             if (!i)
//                 buffer[n - 1] = '\0';
//         }
//         else
//             i = 0;
//
//         LCMapStringA( LOCALE_USER_DEFAULT, LCMAP_LOWERCASE, buffer, n + i, rfc1766, len );
//         return ((n + i) > len) ? E_INVALIDARG : S_OK;
//     }
//     return E_FAIL;
// }
//
// static inline HRESULT lcid_to_rfc1766W( LCID lcid, LPWSTR rfc1766, INT len )
// {
//     WCHAR buffer[MAX_RFC1766_NAME];
//     INT n = GetLocaleInfoW(lcid, LOCALE_SISO639LANGNAME, buffer, MAX_RFC1766_NAME);
//     INT i;
//
//     if (n)
//     {
//         i = PRIMARYLANGID(lcid);
//         if ((((i == LANG_ENGLISH) || (i == LANG_CHINESE) || (i == LANG_ARABIC)) &&
//             (SUBLANGID(lcid) == SUBLANG_DEFAULT)) ||
//             (SUBLANGID(lcid) > SUBLANG_DEFAULT)) {
//
//             buffer[n - 1] = '-';
//             i = GetLocaleInfoW(lcid, LOCALE_SISO3166CTRYNAME, buffer + n, MAX_RFC1766_NAME - n);
//             if (!i)
//                 buffer[n - 1] = '\0';
//         }
//         else
//             i = 0;
//
//         LCMapStringW(LOCALE_USER_DEFAULT, LCMAP_LOWERCASE, buffer, n + i, rfc1766, len);
//         return ((n + i) > len) ? E_INVALIDARG : S_OK;
//     }
//     return E_FAIL;
// }
//
// HRESULT WINAPI LcidToRfc1766A(
//     LCID lcid,
//     LPSTR pszRfc1766,
//     INT nChar)
// {
//     TRACE("%04x %p %u\n", lcid, pszRfc1766, nChar);
//     if (!pszRfc1766)
//         return E_INVALIDARG;
//
//     return lcid_to_rfc1766A(lcid, pszRfc1766, nChar);
// }
//
// HRESULT WINAPI LcidToRfc1766W(
//     LCID lcid,
//     LPWSTR pszRfc1766,
//     INT nChar)
// {
//     TRACE("%04x %p %u\n", lcid, pszRfc1766, nChar);
//     if (!pszRfc1766)
//         return E_INVALIDARG;
//
//     return lcid_to_rfc1766W(lcid, pszRfc1766, nChar);
// }
//
// static HRESULT lcid_from_rfc1766(IEnumRfc1766 *iface, LCID *lcid, LPCWSTR rfc1766)
// {
//     RFC1766INFO info;
//     ULONG num;
//
//     while (IEnumRfc1766_Next(iface, 1, &info, &num) == S_OK)
//     {
//         if (!wcsicmp(info.wszRfc1766, rfc1766))
//         {
//             *lcid = info.lcid;
//             return S_OK;
//         }
//         if (lstrlenW(rfc1766) == 2 && !memcmp(info.wszRfc1766, rfc1766, 2 * sizeof(WCHAR)))
//         {
//             *lcid = PRIMARYLANGID(info.lcid);
//             return S_OK;
//         }
//     }
//
//     return E_FAIL;
// }
//
// HRESULT WINAPI Rfc1766ToLcidW(LCID *pLocale, LPCWSTR pszRfc1766)
// {
//     IEnumRfc1766 *enumrfc1766;
//     HRESULT hr;
//
//     TRACE("(%p, %s)\n", pLocale, debugstr_w(pszRfc1766));
//
//     if (!pLocale || !pszRfc1766)
//         return E_INVALIDARG;
//
//     hr = EnumRfc1766_create(0, &enumrfc1766);
//     if (FAILED(hr))
//         return hr;
//
//     hr = lcid_from_rfc1766(enumrfc1766, pLocale, pszRfc1766);
//     IEnumRfc1766_Release(enumrfc1766);
//
//     return hr;
// }
//
// HRESULT WINAPI Rfc1766ToLcidA(LCID *lcid, LPCSTR rfc1766A)
// {
//     WCHAR rfc1766W[MAX_RFC1766_NAME + 1];
//
//     if (!rfc1766A)
//         return E_INVALIDARG;
//
//     MultiByteToWideChar(CP_ACP, 0, rfc1766A, -1, rfc1766W, MAX_RFC1766_NAME);
//     rfc1766W[MAX_RFC1766_NAME] = 0;
//
//     return Rfc1766ToLcidW(lcid, rfc1766W);
// }
//
// static HRESULT map_font(HDC hdc, DWORD codepages, HFONT src_font, HFONT *dst_font)
// {
//     struct font_list *font_list_entry;
//     CHARSETINFO charset_info;
//     HFONT new_font, old_font;
//     LOGFONTW font_attr;
//     DWORD mask, Csb[2];
//     BOOL found_cached;
//     UINT charset;
//     BOOL ret;
//     UINT i;
//
//     if (hdc == NULL || src_font == NULL) return E_FAIL;
//
//     for (i = 0; i < 32; i++)
//     {
//         mask = (DWORD)(1 << i);
//         if (codepages & mask)
//         {
//             Csb[0] = mask;
//             Csb[1] = 0x0;
//             ret = TranslateCharsetInfo(Csb, &charset_info, TCI_SRCFONTSIG);
//             if (!ret) continue;
//
//             /* use cached font if possible */
//             found_cached = FALSE;
//             EnterCriticalSection(&font_cache_critical);
//             LIST_FOR_EACH_ENTRY(font_list_entry, &font_cache, struct font_list, list_entry)
//             {
//                 if (font_list_entry->charset == charset_info.ciCharset &&
//                     font_list_entry->base_font == src_font)
//                 {
//                     if (dst_font != NULL)
//                         *dst_font = font_list_entry->font;
//                     found_cached = TRUE;
//                 }
//             }
//             LeaveCriticalSection(&font_cache_critical);
//             if (found_cached) return S_OK;
//
//             GetObjectW(src_font, sizeof(font_attr), &font_attr);
//             font_attr.lfCharSet = (BYTE)charset_info.ciCharset;
//             font_attr.lfWidth = 0;
//             font_attr.lfFaceName[0] = 0;
//             new_font = CreateFontIndirectW(&font_attr);
//             if (new_font == NULL) continue;
//
//             old_font = SelectObject(hdc, new_font);
//             charset = GetTextCharset(hdc);
//             SelectObject(hdc, old_font);
//             if (charset == charset_info.ciCharset)
//             {
//                 font_list_entry = HeapAlloc(GetProcessHeap(), 0, sizeof(*font_list_entry));
//                 if (font_list_entry == NULL) return E_OUTOFMEMORY;
//
//                 font_list_entry->base_font = src_font;
//                 font_list_entry->font = new_font;
//                 font_list_entry->charset = charset;
//
//                 EnterCriticalSection(&font_cache_critical);
//                 list_add_tail(&font_cache, &font_list_entry->list_entry);
//                 LeaveCriticalSection(&font_cache_critical);
//
//                 if (dst_font != NULL)
//                     *dst_font = new_font;
//                 return S_OK;
//             }
//         }
//     }
//
//     return E_FAIL;
// }
//
// static HRESULT release_font(HFONT font)
// {
//     struct font_list *font_list_entry;
//     HRESULT hr;
//
//     hr = E_FAIL;
//     EnterCriticalSection(&font_cache_critical);
//     LIST_FOR_EACH_ENTRY(font_list_entry, &font_cache, struct font_list, list_entry)
//     {
//         if (font_list_entry->font == font)
//         {
//             list_remove(&font_list_entry->list_entry);
//             DeleteObject(font);
//             HeapFree(GetProcessHeap(), 0, font_list_entry);
//             hr = S_OK;
//             break;
//         }
//     }
//     LeaveCriticalSection(&font_cache_critical);
//
//     return hr;
// }
//
// static HRESULT clear_font_cache(void)
// {
//     struct font_list *font_list_entry;
//     struct font_list *font_list_entry2;
//
//     EnterCriticalSection(&font_cache_critical);
//     LIST_FOR_EACH_ENTRY_SAFE(font_list_entry, font_list_entry2, &font_cache, struct font_list, list_entry)
//     {
//         list_remove(&font_list_entry->list_entry);
//         DeleteObject(font_list_entry->font);
//         HeapFree(GetProcessHeap(), 0, font_list_entry);
//     }
//     LeaveCriticalSection(&font_cache_critical);
//
//     return S_OK;
// }
//
// /******************************************************************************
//  * MLANG ClassFactory
//  */
// typedef struct {
//     IClassFactory IClassFactory_iface;
//     LONG ref;
//     HRESULT (*pfnCreateInstance)(IUnknown *pUnkOuter, LPVOID *ppObj);
// } IClassFactoryImpl;
//
// static inline IClassFactoryImpl *impl_from_IClassFactory(IClassFactory *iface)
// {
//     return CONTAINING_RECORD(iface, IClassFactoryImpl, IClassFactory_iface);
// }
//
// struct object_creation_info
// {
//     const CLSID *clsid;
//     LPCSTR szClassName;
//     HRESULT (*pfnCreateInstance)(IUnknown *pUnkOuter, LPVOID *ppObj);
// };
//
// static const struct object_creation_info object_creation[] =
// {
//     { &CLSID_CMultiLanguage, "CLSID_CMultiLanguage", MultiLanguage_create },
//     { &CLSID_CMLangConvertCharset, "CLSID_CMLangConvertCharset", MLangConvertCharset_create }
// };
//
// static HRESULT WINAPI MLANGCF_QueryInterface(IClassFactory *iface, REFIID riid, void **ppobj)
// {
//     TRACE("%s\n", debugstr_guid(riid) );
//
//     if (IsEqualGUID(riid, &IID_IUnknown)
// 	|| IsEqualGUID(riid, &IID_IClassFactory))
//     {
// 	IClassFactory_AddRef(iface);
//         *ppobj = iface;
// 	return S_OK;
//     }
//
//     *ppobj = NULL;
//     WARN("(%p)->(%s,%p), not found\n", iface, debugstr_guid(riid), ppobj);
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI MLANGCF_AddRef(IClassFactory *iface)
// {
//     IClassFactoryImpl *This = impl_from_IClassFactory(iface);
//     return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI MLANGCF_Release(IClassFactory *iface)
// {
//     IClassFactoryImpl *This = impl_from_IClassFactory(iface);
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     if (ref == 0)
//     {
//         TRACE("Destroying %p\n", This);
// 	HeapFree(GetProcessHeap(), 0, This);
//     }
//
//     return ref;
// }
//
// static HRESULT WINAPI MLANGCF_CreateInstance(IClassFactory *iface, IUnknown *pOuter,
//         REFIID riid, void **ppobj)
// {
//     IClassFactoryImpl *This = impl_from_IClassFactory(iface);
//     HRESULT hres;
//     LPUNKNOWN punk;
//
//     TRACE("(%p)->(%p,%s,%p)\n",This,pOuter,debugstr_guid(riid),ppobj);
//
//     *ppobj = NULL;
//     hres = This->pfnCreateInstance(pOuter, (LPVOID *) &punk);
//     if (SUCCEEDED(hres)) {
//         hres = IUnknown_QueryInterface(punk, riid, ppobj);
//         IUnknown_Release(punk);
//     }
//     TRACE("returning (%p) -> %x\n", *ppobj, hres);
//     return hres;
// }
//
// static HRESULT WINAPI MLANGCF_LockServer(IClassFactory *iface, BOOL dolock)
// {
//     if (dolock)
//         LockModule();
//     else
//         UnlockModule();
//
//     return S_OK;
// }
//
// static const IClassFactoryVtbl MLANGCF_Vtbl =
// {
//     MLANGCF_QueryInterface,
//     MLANGCF_AddRef,
//     MLANGCF_Release,
//     MLANGCF_CreateInstance,
//     MLANGCF_LockServer
// };
//
// /******************************************************************
//  *		DllGetClassObject (MLANG.@)
//  */
// HRESULT WINAPI DllGetClassObject(REFCLSID rclsid, REFIID iid, LPVOID *ppv)
// {
//     unsigned int i;
//     IClassFactoryImpl *factory;
//
//     TRACE("%s %s %p\n",debugstr_guid(rclsid), debugstr_guid(iid), ppv);
//
//     if ( !IsEqualGUID( &IID_IClassFactory, iid )
// 	 && ! IsEqualGUID( &IID_IUnknown, iid) )
// 	return E_NOINTERFACE;
//
//     for (i = 0; i < ARRAY_SIZE(object_creation); i++)
//     {
// 	if (IsEqualGUID(object_creation[i].clsid, rclsid))
// 	    break;
//     }
//
//     if (i == ARRAY_SIZE(object_creation))
//     {
// 	FIXME("%s: no class found.\n", debugstr_guid(rclsid));
// 	return CLASS_E_CLASSNOTAVAILABLE;
//     }
//
//     TRACE("Creating a class factory for %s\n",object_creation[i].szClassName);
//
//     factory = HeapAlloc(GetProcessHeap(), 0, sizeof(*factory));
//     if (factory == NULL) return E_OUTOFMEMORY;
//
//     factory->IClassFactory_iface.lpVtbl = &MLANGCF_Vtbl;
//     factory->ref = 1;
//
//     factory->pfnCreateInstance = object_creation[i].pfnCreateInstance;
//
//     *ppv = &factory->IClassFactory_iface;
//
//     TRACE("(%p) <- %p\n", ppv, &factory->IClassFactory_iface);
//
//     return S_OK;
// }
//
//
// /******************************************************************************/
//
// typedef struct tagMLang_impl
// {
//     IMLangFontLink IMLangFontLink_iface;
//     IMultiLanguage IMultiLanguage_iface;
//     IMultiLanguage3 IMultiLanguage3_iface;
//     IMLangFontLink2 IMLangFontLink2_iface;
//     IMLangLineBreakConsole IMLangLineBreakConsole_iface;
//     LONG ref;
//     DWORD total_cp, total_scripts;
// } MLang_impl;
//
// /******************************************************************************/
//
// typedef struct tagEnumCodePage_impl
// {
//     IEnumCodePage IEnumCodePage_iface;
//     LONG ref;
//     MIMECPINFO *cpinfo;
//     DWORD total, pos;
// } EnumCodePage_impl;
//
// static inline EnumCodePage_impl *impl_from_IEnumCodePage( IEnumCodePage *iface )
// {
//     return CONTAINING_RECORD( iface, EnumCodePage_impl, IEnumCodePage_iface );
// }
//
// static HRESULT WINAPI fnIEnumCodePage_QueryInterface(
//         IEnumCodePage* iface,
//         REFIID riid,
//         void** ppvObject)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//
//     TRACE("%p -> %s\n", This, debugstr_guid(riid) );
//
//     if (IsEqualGUID(riid, &IID_IUnknown)
// 	|| IsEqualGUID(riid, &IID_IEnumCodePage))
//     {
// 	IEnumCodePage_AddRef(iface);
//         TRACE("Returning IID_IEnumCodePage %p ref = %d\n", This, This->ref);
// 	*ppvObject = &This->IEnumCodePage_iface;
//         return S_OK;
//     }
//
//     WARN("(%p)->(%s,%p),not found\n",This,debugstr_guid(riid),ppvObject);
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI fnIEnumCodePage_AddRef(
//         IEnumCodePage* iface)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//     return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI fnIEnumCodePage_Release(
//         IEnumCodePage* iface)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("%p ref = %d\n", This, ref);
//     if (ref == 0)
//     {
//         TRACE("Destroying %p\n", This);
//         HeapFree(GetProcessHeap(), 0, This->cpinfo);
//         HeapFree(GetProcessHeap(), 0, This);
//     }
//
//     return ref;
// }
//
// static HRESULT WINAPI fnIEnumCodePage_Clone(
//         IEnumCodePage* iface,
//         IEnumCodePage** ppEnum)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//     FIXME("%p %p\n", This, ppEnum);
//     return E_NOTIMPL;
// }
//
// static  HRESULT WINAPI fnIEnumCodePage_Next(
//         IEnumCodePage* iface,
//         ULONG celt,
//         PMIMECPINFO rgelt,
//         ULONG* pceltFetched)
// {
//     ULONG i;
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//
//     TRACE("%p %u %p %p\n", This, celt, rgelt, pceltFetched);
//
//     if (!pceltFetched) return S_FALSE;
//     *pceltFetched = 0;
//
//     if (!rgelt) return S_FALSE;
//
//     if (This->pos + celt > This->total)
//         celt = This->total - This->pos;
//
//     if (!celt) return S_FALSE;
//
//     memcpy(rgelt, This->cpinfo + This->pos, celt * sizeof(MIMECPINFO));
//     *pceltFetched = celt;
//     This->pos += celt;
//
//     for (i = 0; i < celt; i++)
//     {
//         TRACE("#%u: %08x %u %u %s %s %s %s %s %s %d\n",
//               i, rgelt[i].dwFlags, rgelt[i].uiCodePage,
//               rgelt[i].uiFamilyCodePage,
//               wine_dbgstr_w(rgelt[i].wszDescription),
//               wine_dbgstr_w(rgelt[i].wszWebCharset),
//               wine_dbgstr_w(rgelt[i].wszHeaderCharset),
//               wine_dbgstr_w(rgelt[i].wszBodyCharset),
//               wine_dbgstr_w(rgelt[i].wszFixedWidthFont),
//               wine_dbgstr_w(rgelt[i].wszProportionalFont),
//               rgelt[i].bGDICharset);
//     }
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIEnumCodePage_Reset(
//         IEnumCodePage* iface)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//
//     TRACE("%p\n", This);
//
//     This->pos = 0;
//     return S_OK;
// }
//
// static  HRESULT WINAPI fnIEnumCodePage_Skip(
//         IEnumCodePage* iface,
//         ULONG celt)
// {
//     EnumCodePage_impl *This = impl_from_IEnumCodePage( iface );
//
//     TRACE("%p %u\n", This, celt);
//
//     if (celt >= This->total) return S_FALSE;
//
//     This->pos += celt;
//     return S_OK;
// }
//
// static const IEnumCodePageVtbl IEnumCodePage_vtbl =
// {
//     fnIEnumCodePage_QueryInterface,
//     fnIEnumCodePage_AddRef,
//     fnIEnumCodePage_Release,
//     fnIEnumCodePage_Clone,
//     fnIEnumCodePage_Next,
//     fnIEnumCodePage_Reset,
//     fnIEnumCodePage_Skip
// };
//
// static HRESULT EnumCodePage_create( MLang_impl* mlang, DWORD grfFlags,
//                      LANGID LangId, IEnumCodePage** ppEnumCodePage )
// {
//     EnumCodePage_impl *ecp;
//     MIMECPINFO *cpinfo;
//     UINT i, n;
//
//     TRACE("%p, %08x, %04x, %p\n", mlang, grfFlags, LangId, ppEnumCodePage);
//
//     if (!grfFlags) /* enumerate internal data base of encodings */
//         grfFlags = MIMECONTF_MIME_LATEST;
//
//     ecp = HeapAlloc( GetProcessHeap(), 0, sizeof (EnumCodePage_impl) );
//     ecp->IEnumCodePage_iface.lpVtbl = &IEnumCodePage_vtbl;
//     ecp->ref = 1;
//     ecp->pos = 0;
//     ecp->total = 0;
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].flags & grfFlags)
//                 ecp->total++;
//         }
//     }
//
//     ecp->cpinfo = HeapAlloc(GetProcessHeap(), 0,
//                             sizeof(MIMECPINFO) * ecp->total);
//     cpinfo = ecp->cpinfo;
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].flags & grfFlags)
//                 fill_cp_info(&mlang_data[i], n, cpinfo++);
//         }
//     }
//
//     TRACE("enumerated %d codepages with flags %08x\n", ecp->total, grfFlags);
//
//     *ppEnumCodePage = &ecp->IEnumCodePage_iface;
//
//     return S_OK;
// }
//
// /******************************************************************************/
//
// typedef struct tagEnumScript_impl
// {
//     IEnumScript IEnumScript_iface;
//     LONG ref;
//     SCRIPTINFO *script_info;
//     DWORD total, pos;
// } EnumScript_impl;
//
// static inline EnumScript_impl *impl_from_IEnumScript( IEnumScript *iface )
// {
//     return CONTAINING_RECORD( iface, EnumScript_impl, IEnumScript_iface );
// }
//
// static HRESULT WINAPI fnIEnumScript_QueryInterface(
//         IEnumScript* iface,
//         REFIID riid,
//         void** ppvObject)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//
//     TRACE("%p -> %s\n", This, debugstr_guid(riid) );
//
//     if (IsEqualGUID(riid, &IID_IUnknown)
//         || IsEqualGUID(riid, &IID_IEnumScript))
//     {
//         IEnumScript_AddRef(iface);
//         TRACE("Returning IID_IEnumScript %p ref = %d\n", This, This->ref);
//         *ppvObject = &This->IEnumScript_iface;
//         return S_OK;
//     }
//
//     WARN("(%p)->(%s,%p),not found\n",This,debugstr_guid(riid),ppvObject);
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI fnIEnumScript_AddRef(
//         IEnumScript* iface)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//     return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI fnIEnumScript_Release(
//         IEnumScript* iface)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("%p ref = %d\n", This, ref);
//     if (ref == 0)
//     {
//         TRACE("Destroying %p\n", This);
//         HeapFree(GetProcessHeap(), 0, This->script_info);
//         HeapFree(GetProcessHeap(), 0, This);
//     }
//
//     return ref;
// }
//
// static HRESULT WINAPI fnIEnumScript_Clone(
//         IEnumScript* iface,
//         IEnumScript** ppEnum)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//     FIXME("%p %p: stub!\n", This, ppEnum);
//     return E_NOTIMPL;
// }
//
// static  HRESULT WINAPI fnIEnumScript_Next(
//         IEnumScript* iface,
//         ULONG celt,
//         PSCRIPTINFO rgelt,
//         ULONG* pceltFetched)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//
//     TRACE("%p %u %p %p\n", This, celt, rgelt, pceltFetched);
//
//     if (!pceltFetched || !rgelt) return E_FAIL;
//
//     *pceltFetched = 0;
//
//     if (This->pos + celt > This->total)
//         celt = This->total - This->pos;
//
//     if (!celt) return S_FALSE;
//
//     memcpy(rgelt, This->script_info + This->pos, celt * sizeof(SCRIPTINFO));
//     *pceltFetched = celt;
//     This->pos += celt;
//
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIEnumScript_Reset(
//         IEnumScript* iface)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//
//     TRACE("%p\n", This);
//
//     This->pos = 0;
//     return S_OK;
// }
//
// static  HRESULT WINAPI fnIEnumScript_Skip(
//         IEnumScript* iface,
//         ULONG celt)
// {
//     EnumScript_impl *This = impl_from_IEnumScript( iface );
//
//     TRACE("%p %u\n", This, celt);
//
//     if (celt >= This->total) return S_FALSE;
//
//     This->pos += celt;
//     return S_OK;
// }
//
// static const IEnumScriptVtbl IEnumScript_vtbl =
// {
//     fnIEnumScript_QueryInterface,
//     fnIEnumScript_AddRef,
//     fnIEnumScript_Release,
//     fnIEnumScript_Clone,
//     fnIEnumScript_Next,
//     fnIEnumScript_Reset,
//     fnIEnumScript_Skip
// };
//
// static HRESULT EnumScript_create( MLang_impl* mlang, DWORD dwFlags,
//                      LANGID LangId, IEnumScript** ppEnumScript )
// {
//     EnumScript_impl *es;
//     UINT i;
//
//     TRACE("%p, %08x, %04x, %p\n", mlang, dwFlags, LangId, ppEnumScript);
//
//     if (!dwFlags) /* enumerate all available scripts */
//         dwFlags = SCRIPTCONTF_SCRIPT_USER | SCRIPTCONTF_SCRIPT_HIDE | SCRIPTCONTF_SCRIPT_SYSTEM;
//
//     es = HeapAlloc( GetProcessHeap(), 0, sizeof (EnumScript_impl) );
//     es->IEnumScript_iface.lpVtbl = &IEnumScript_vtbl;
//     es->ref = 1;
//     es->pos = 0;
//     /* do not enumerate unicode flavours */
//     es->total = ARRAY_SIZE(mlang_data) - 1;
//     es->script_info = HeapAlloc(GetProcessHeap(), 0, sizeof(SCRIPTINFO) * es->total);
//
//     for (i = 0; i < es->total; i++)
//     {
//         es->script_info[i].ScriptId = i;
//         es->script_info[i].uiCodePage = mlang_data[i].family_codepage;
//         MultiByteToWideChar(CP_ACP, 0, mlang_data[i].description, -1,
//             es->script_info[i].wszDescription, MAX_SCRIPT_NAME);
//         MultiByteToWideChar(CP_ACP, 0, mlang_data[i].fixed_font, -1,
//             es->script_info[i].wszFixedWidthFont, MAX_MIMEFACE_NAME);
//         MultiByteToWideChar(CP_ACP, 0, mlang_data[i].proportional_font, -1,
//             es->script_info[i].wszProportionalFont, MAX_MIMEFACE_NAME);
//     }
//
//     TRACE("enumerated %d scripts with flags %08x\n", es->total, dwFlags);
//
//     *ppEnumScript = &es->IEnumScript_iface;
//
//     return S_OK;
// }
//
// /******************************************************************************/
//
// static inline MLang_impl *impl_from_IMLangFontLink( IMLangFontLink *iface )
// {
//     return CONTAINING_RECORD( iface, MLang_impl, IMLangFontLink_iface );
// }
//
// static HRESULT WINAPI fnIMLangFontLink_QueryInterface(
//         IMLangFontLink* iface,
//         REFIID riid,
//         void** ppvObject)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMultiLanguage3_QueryInterface( &This->IMultiLanguage3_iface, riid, ppvObject );
// }
//
// static ULONG WINAPI fnIMLangFontLink_AddRef(
//         IMLangFontLink* iface)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMultiLanguage3_AddRef( &This->IMultiLanguage3_iface );
// }
//
// static ULONG WINAPI fnIMLangFontLink_Release(
//         IMLangFontLink* iface)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMultiLanguage3_Release( &This->IMultiLanguage3_iface );
// }
//
// static HRESULT WINAPI fnIMLangFontLink_GetCharCodePages(
//         IMLangFontLink* iface,
//         WCHAR ch_src,
//         DWORD* codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMLangFontLink2_GetCharCodePages(&This->IMLangFontLink2_iface, ch_src, codepages);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_GetStrCodePages(
//         IMLangFontLink* iface,
//         const WCHAR* src,
//         LONG src_len,
//         DWORD priority_cp,
//         DWORD* codepages,
//         LONG* ret_len)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMLangFontLink2_GetStrCodePages(&This->IMLangFontLink2_iface, src, src_len, priority_cp,
//         codepages, ret_len);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_CodePageToCodePages(
//         IMLangFontLink* iface,
//         UINT codepage,
//         DWORD* codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink( iface );
//     return IMLangFontLink2_CodePageToCodePages(&This->IMLangFontLink2_iface, codepage, codepages);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_CodePagesToCodePage(
//         IMLangFontLink* iface,
//         DWORD codepages,
//         UINT def_codepage,
//         UINT* codepage)
// {
//     MLang_impl *This = impl_from_IMLangFontLink(iface);
//     return IMLangFontLink2_CodePagesToCodePage(&This->IMLangFontLink2_iface, codepages,
//         def_codepage, codepage);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_GetFontCodePages(
//         IMLangFontLink* iface,
//         HDC hdc,
//         HFONT hfont,
//         DWORD* codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink(iface);
//     return IMLangFontLink2_GetFontCodePages(&This->IMLangFontLink2_iface, hdc, hfont, codepages);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_MapFont(
//         IMLangFontLink* iface,
//         HDC hDC,
//         DWORD dwCodePages,
//         HFONT hSrcFont,
//         HFONT* phDestFont)
// {
//     TRACE("(%p)->%p %08x %p %p\n",iface, hDC, dwCodePages, hSrcFont, phDestFont);
//
//     return map_font(hDC, dwCodePages, hSrcFont, phDestFont);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_ReleaseFont(
//         IMLangFontLink* iface,
//         HFONT hFont)
// {
//     TRACE("(%p)->%p\n",iface, hFont);
//
//     return release_font(hFont);
// }
//
// static HRESULT WINAPI fnIMLangFontLink_ResetFontMapping(
//         IMLangFontLink* iface)
// {
//     TRACE("(%p)\n",iface);
//
//     return clear_font_cache();
// }
//
//
// static const IMLangFontLinkVtbl IMLangFontLink_vtbl =
// {
//     fnIMLangFontLink_QueryInterface,
//     fnIMLangFontLink_AddRef,
//     fnIMLangFontLink_Release,
//     fnIMLangFontLink_GetCharCodePages,
//     fnIMLangFontLink_GetStrCodePages,
//     fnIMLangFontLink_CodePageToCodePages,
//     fnIMLangFontLink_CodePagesToCodePage,
//     fnIMLangFontLink_GetFontCodePages,
//     fnIMLangFontLink_MapFont,
//     fnIMLangFontLink_ReleaseFont,
//     fnIMLangFontLink_ResetFontMapping,
// };
//
// /******************************************************************************/
//
// static inline MLang_impl *impl_from_IMultiLanguage( IMultiLanguage *iface )
// {
//     return CONTAINING_RECORD( iface, MLang_impl, IMultiLanguage_iface );
// }
//
// static HRESULT WINAPI fnIMultiLanguage_QueryInterface(
//     IMultiLanguage* iface,
//     REFIID riid,
//     void** obj)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_QueryInterface(&This->IMultiLanguage3_iface, riid, obj);
// }
//
// static ULONG WINAPI fnIMultiLanguage_AddRef( IMultiLanguage* iface )
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_AddRef(&This->IMultiLanguage3_iface);
// }
//
// static ULONG WINAPI fnIMultiLanguage_Release( IMultiLanguage* iface )
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_Release(&This->IMultiLanguage3_iface);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetNumberOfCodePageInfo(
//     IMultiLanguage* iface,
//     UINT* cp)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     TRACE("(%p, %p)\n", This, cp);
//     return IMultiLanguage3_GetNumberOfCodePageInfo(&This->IMultiLanguage3_iface, cp);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetCodePageInfo(
//     IMultiLanguage* iface,
//     UINT uiCodePage,
//     PMIMECPINFO pCodePageInfo)
// {
//     UINT i, n;
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//
//     TRACE("%p, %u, %p\n", This, uiCodePage, pCodePageInfo);
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].cp == uiCodePage)
//             {
//                 fill_cp_info(&mlang_data[i], n, pCodePageInfo);
//                 return S_OK;
//             }
//         }
//     }
//
//     return S_FALSE;
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetFamilyCodePage(
//     IMultiLanguage* iface,
//     UINT cp,
//     UINT* family_cp)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_GetFamilyCodePage(&This->IMultiLanguage3_iface, cp, family_cp);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_EnumCodePages(
//     IMultiLanguage* iface,
//     DWORD grfFlags,
//     IEnumCodePage** ppEnumCodePage)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//
//     TRACE("%p %08x %p\n", This, grfFlags, ppEnumCodePage);
//
//     return EnumCodePage_create( This, grfFlags, 0, ppEnumCodePage );
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetCharsetInfo(
//     IMultiLanguage* iface,
//     BSTR Charset,
//     PMIMECSETINFO pCharsetInfo)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_GetCharsetInfo( &This->IMultiLanguage3_iface, Charset, pCharsetInfo );
// }
//
// static HRESULT WINAPI fnIMultiLanguage_IsConvertible(
//     IMultiLanguage* iface,
//     DWORD src_enc,
//     DWORD dst_enc)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_IsConvertible(&This->IMultiLanguage3_iface, src_enc, dst_enc);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_ConvertString(
//     IMultiLanguage* iface,
//     DWORD* mode,
//     DWORD src_enc,
//     DWORD dst_enc,
//     BYTE* src,
//     UINT* src_size,
//     BYTE* dest,
//     UINT* dest_size)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_ConvertString(&This->IMultiLanguage3_iface, mode, src_enc,
//         dst_enc, src, src_size, dest, dest_size);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_ConvertStringToUnicode(
//     IMultiLanguage* iface,
//     DWORD* mode,
//     DWORD src_enc,
//     CHAR* src,
//     UINT* src_size,
//     WCHAR* dest,
//     UINT* dest_size)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_ConvertStringToUnicode(&This->IMultiLanguage3_iface,
//         mode, src_enc, src, src_size, dest, dest_size);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_ConvertStringFromUnicode(
//     IMultiLanguage* iface,
//     DWORD* mode,
//     DWORD encoding,
//     WCHAR* src,
//     UINT* src_size,
//     CHAR* dest,
//     UINT* dest_size)
// {
//     MLang_impl *This = impl_from_IMultiLanguage(iface);
//     return IMultiLanguage3_ConvertStringFromUnicode(&This->IMultiLanguage3_iface,
//         mode, encoding, src, src_size, dest, dest_size);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_ConvertStringReset(
//     IMultiLanguage* iface)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//     return IMultiLanguage3_ConvertStringReset(&This->IMultiLanguage3_iface);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetRfc1766FromLcid(
//     IMultiLanguage* iface,
//     LCID lcid,
//     BSTR* pbstrRfc1766)
// {
//     MLang_impl *This = impl_from_IMultiLanguage(iface);
//     return IMultiLanguage3_GetRfc1766FromLcid(&This->IMultiLanguage3_iface, lcid, pbstrRfc1766);
// }
//
// static HRESULT WINAPI fnIMultiLanguage_GetLcidFromRfc1766(
//     IMultiLanguage* iface,
//     LCID* locale,
//     BSTR rfc1766)
// {
//     MLang_impl *This = impl_from_IMultiLanguage(iface);
//     return IMultiLanguage3_GetLcidFromRfc1766(&This->IMultiLanguage3_iface, locale, rfc1766);
// }
//
// /******************************************************************************/
//
// typedef struct tagEnumRfc1766_impl
// {
//     IEnumRfc1766 IEnumRfc1766_iface;
//     LONG ref;
//     RFC1766INFO *info;
//     DWORD total, pos;
// } EnumRfc1766_impl;
//
// static inline EnumRfc1766_impl *impl_from_IEnumRfc1766( IEnumRfc1766 *iface )
// {
//     return CONTAINING_RECORD( iface, EnumRfc1766_impl, IEnumRfc1766_iface );
// }
//
// static HRESULT WINAPI fnIEnumRfc1766_QueryInterface(
//         IEnumRfc1766 *iface,
//         REFIID riid,
//         void** ppvObject)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//
//     TRACE("%p -> %s\n", This, debugstr_guid(riid) );
//
//     if (IsEqualGUID(riid, &IID_IUnknown)
//         || IsEqualGUID(riid, &IID_IEnumRfc1766))
//     {
//         IEnumRfc1766_AddRef(iface);
//         TRACE("Returning IID_IEnumRfc1766 %p ref = %d\n", This, This->ref);
//         *ppvObject = &This->IEnumRfc1766_iface;
//         return S_OK;
//     }
//
//     WARN("(%p) -> (%s,%p), not found\n",This,debugstr_guid(riid),ppvObject);
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI fnIEnumRfc1766_AddRef(
//         IEnumRfc1766 *iface)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//     return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI fnIEnumRfc1766_Release(
//         IEnumRfc1766 *iface)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("%p ref = %d\n", This, ref);
//     if (ref == 0)
//     {
//         TRACE("Destroying %p\n", This);
//         HeapFree(GetProcessHeap(), 0, This->info);
//         HeapFree(GetProcessHeap(), 0, This);
//     }
//     return ref;
// }
//
// static HRESULT WINAPI fnIEnumRfc1766_Clone(
//         IEnumRfc1766 *iface,
//         IEnumRfc1766 **ppEnum)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//
//     FIXME("%p %p\n", This, ppEnum);
//     return E_NOTIMPL;
// }
//
// static  HRESULT WINAPI fnIEnumRfc1766_Next(
//         IEnumRfc1766 *iface,
//         ULONG celt,
//         PRFC1766INFO rgelt,
//         ULONG *pceltFetched)
// {
//     ULONG i;
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//
//     TRACE("%p %u %p %p\n", This, celt, rgelt, pceltFetched);
//
//     if (!pceltFetched) return S_FALSE;
//     *pceltFetched = 0;
//
//     if (!rgelt) return S_FALSE;
//
//     if (This->pos + celt > This->total)
//         celt = This->total - This->pos;
//
//     if (!celt) return S_FALSE;
//
//     memcpy(rgelt, This->info + This->pos, celt * sizeof(RFC1766INFO));
//     *pceltFetched = celt;
//     This->pos += celt;
//
//     for (i = 0; i < celt; i++)
//     {
//         TRACE("#%u: %08x %s %s\n",
//               i, rgelt[i].lcid,
//               wine_dbgstr_w(rgelt[i].wszRfc1766),
//               wine_dbgstr_w(rgelt[i].wszLocaleName));
//     }
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIEnumRfc1766_Reset(
//         IEnumRfc1766 *iface)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//
//     TRACE("%p\n", This);
//
//     This->pos = 0;
//     return S_OK;
// }
//
// static  HRESULT WINAPI fnIEnumRfc1766_Skip(
//         IEnumRfc1766 *iface,
//         ULONG celt)
// {
//     EnumRfc1766_impl *This = impl_from_IEnumRfc1766( iface );
//
//     TRACE("%p %u\n", This, celt);
//
//     if (celt >= This->total) return S_FALSE;
//
//     This->pos += celt;
//     return S_OK;
// }
//
// static const IEnumRfc1766Vtbl IEnumRfc1766_vtbl =
// {
//     fnIEnumRfc1766_QueryInterface,
//     fnIEnumRfc1766_AddRef,
//     fnIEnumRfc1766_Release,
//     fnIEnumRfc1766_Clone,
//     fnIEnumRfc1766_Next,
//     fnIEnumRfc1766_Reset,
//     fnIEnumRfc1766_Skip
// };
//
// struct enum_locales_data
// {
//     RFC1766INFO *info;
//     DWORD total, allocated;
// };
//
// static BOOL __stdcall enum_locales_proc(LPWSTR locale)
// {
//     WCHAR *end;
//     struct enum_locales_data *data = TlsGetValue(MLANG_tls_index);
//     RFC1766INFO *info;
//
//     TRACE("%s\n", debugstr_w(locale));
//
//     if (data->total >= data->allocated)
//     {
//         data->allocated += 32;
//         data->info = HeapReAlloc(GetProcessHeap(), 0, data->info, data->allocated * sizeof(RFC1766INFO));
//         if (!data->info) return FALSE;
//     }
//
//     info = &data->info[data->total];
//
//     info->lcid = wcstol(locale, &end, 16);
//     if (*end) /* invalid number */
//         return FALSE;
//
//     info->wszRfc1766[0] = 0;
//     lcid_to_rfc1766W( info->lcid, info->wszRfc1766, MAX_RFC1766_NAME );
//
//     info->wszLocaleName[0] = 0;
//     GetLocaleInfoW(info->lcid, LOCALE_SLANGUAGE, info->wszLocaleName, MAX_LOCALE_NAME);
//     TRACE("ISO639: %s SLANGUAGE: %s\n", wine_dbgstr_w(info->wszRfc1766), wine_dbgstr_w(info->wszLocaleName));
//
//     data->total++;
//
//     return TRUE;
// }
//
// static HRESULT EnumRfc1766_create(LANGID LangId, IEnumRfc1766 **ppEnum)
// {
//     EnumRfc1766_impl *rfc;
//     struct enum_locales_data data;
//
//     TRACE("%04x, %p\n", LangId, ppEnum);
//
//     rfc = HeapAlloc( GetProcessHeap(), 0, sizeof(EnumRfc1766_impl) );
//     rfc->IEnumRfc1766_iface.lpVtbl = &IEnumRfc1766_vtbl;
//     rfc->ref = 1;
//     rfc->pos = 0;
//     rfc->total = 0;
//
//     data.total = 0;
//     data.allocated = 160;
//     data.info = HeapAlloc(GetProcessHeap(), 0, data.allocated * sizeof(RFC1766INFO));
//     if (!data.info)
//     {
//         HeapFree(GetProcessHeap(), 0, rfc);
//         return E_OUTOFMEMORY;
//     }
//
//     TlsSetValue(MLANG_tls_index, &data);
//     EnumSystemLocalesW(enum_locales_proc, 0/*LOCALE_SUPPORTED*/);
//     TlsSetValue(MLANG_tls_index, NULL);
//
//     TRACE("enumerated %d rfc1766 structures\n", data.total);
//
//     if (!data.total)
//     {
//         HeapFree(GetProcessHeap(), 0, data.info);
//         HeapFree(GetProcessHeap(), 0, rfc);
//         return E_FAIL;
//     }
//
//     rfc->info = data.info;
//     rfc->total = data.total;
//
//     *ppEnum = &rfc->IEnumRfc1766_iface;
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMultiLanguage_EnumRfc1766(
//     IMultiLanguage *iface,
//     IEnumRfc1766 **ppEnumRfc1766)
// {
//     MLang_impl *This = impl_from_IMultiLanguage( iface );
//
//     TRACE("%p %p\n", This, ppEnumRfc1766);
//
//     return EnumRfc1766_create(0, ppEnumRfc1766);
// }
//
// /******************************************************************************/
//
// static HRESULT WINAPI fnIMultiLanguage_GetRfc1766Info(
//     IMultiLanguage* iface,
//     LCID Locale,
//     PRFC1766INFO pRfc1766Info)
// {
//     LCTYPE type = LOCALE_SLANGUAGE;
//
//     TRACE("(%p, 0x%04x, %p)\n", iface, Locale, pRfc1766Info);
//
//     if (!pRfc1766Info)
//         return E_INVALIDARG;
//
//     if ((PRIMARYLANGID(Locale) == LANG_ENGLISH) ||
//         (PRIMARYLANGID(Locale) == LANG_CHINESE) ||
//         (PRIMARYLANGID(Locale) == LANG_ARABIC)) {
//
//         if (!SUBLANGID(Locale))
//             type = LOCALE_SENGLANGUAGE; /* suppress country */
//     }
//     else
//     {
//         if (!SUBLANGID(Locale)) {
//             TRACE("SUBLANGID missing in 0x%04x\n", Locale);
//             return E_FAIL;
//         }
//     }
//
//     pRfc1766Info->lcid = Locale;
//     pRfc1766Info->wszRfc1766[0] = 0;
//     pRfc1766Info->wszLocaleName[0] = 0;
//
//     if ((!lcid_to_rfc1766W(Locale, pRfc1766Info->wszRfc1766, MAX_RFC1766_NAME)) &&
//         (GetLocaleInfoW(Locale, type, pRfc1766Info->wszLocaleName, MAX_LOCALE_NAME) > 0))
//             return S_OK;
//
//     /* Locale not supported */
//     return E_INVALIDARG;
// }
//
// static HRESULT WINAPI fnIMultiLanguage_CreateConvertCharset(
//     IMultiLanguage* iface,
//     UINT src_cp,
//     UINT dst_cp,
//     DWORD prop,
//     IMLangConvertCharset** convert_charset)
// {
//     MLang_impl *This = impl_from_IMultiLanguage(iface);
//     return IMultiLanguage3_CreateConvertCharset(&This->IMultiLanguage3_iface, src_cp, dst_cp, prop, convert_charset);
// }
//
// static const IMultiLanguageVtbl IMultiLanguage_vtbl =
// {
//     fnIMultiLanguage_QueryInterface,
//     fnIMultiLanguage_AddRef,
//     fnIMultiLanguage_Release,
//     fnIMultiLanguage_GetNumberOfCodePageInfo,
//     fnIMultiLanguage_GetCodePageInfo,
//     fnIMultiLanguage_GetFamilyCodePage,
//     fnIMultiLanguage_EnumCodePages,
//     fnIMultiLanguage_GetCharsetInfo,
//     fnIMultiLanguage_IsConvertible,
//     fnIMultiLanguage_ConvertString,
//     fnIMultiLanguage_ConvertStringToUnicode,
//     fnIMultiLanguage_ConvertStringFromUnicode,
//     fnIMultiLanguage_ConvertStringReset,
//     fnIMultiLanguage_GetRfc1766FromLcid,
//     fnIMultiLanguage_GetLcidFromRfc1766,
//     fnIMultiLanguage_EnumRfc1766,
//     fnIMultiLanguage_GetRfc1766Info,
//     fnIMultiLanguage_CreateConvertCharset,
// };
//
//
// /******************************************************************************/
//
// static inline MLang_impl *impl_from_IMultiLanguage3( IMultiLanguage3 *iface )
// {
//     return CONTAINING_RECORD( iface, MLang_impl, IMultiLanguage3_iface );
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_QueryInterface(
//     IMultiLanguage3* iface,
//     REFIID riid,
//     void** obj)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("(%p)->(%s %p)\n", This, debugstr_guid(riid), obj);
//
//     if (IsEqualGUID(riid, &IID_IUnknown) ||
//         IsEqualGUID(riid, &IID_IMultiLanguage))
//     {
//         *obj = &This->IMultiLanguage_iface;
//     }
//     else if (IsEqualGUID(riid, &IID_IMLangCodePages) ||
//              IsEqualGUID(riid, &IID_IMLangFontLink))
//     {
//         *obj = &This->IMLangFontLink_iface;
//     }
//     else if (IsEqualGUID(riid, &IID_IMLangFontLink2))
//     {
//         *obj = &This->IMLangFontLink2_iface;
//     }
//     else if (IsEqualGUID(riid, &IID_IMultiLanguage2) ||
//              IsEqualGUID(riid, &IID_IMultiLanguage3))
//     {
//         *obj = &This->IMultiLanguage3_iface;
//     }
//     else if (IsEqualGUID(riid, &IID_IMLangLineBreakConsole))
//     {
//         *obj = &This->IMLangLineBreakConsole_iface;
//     }
//     else
//     {
//         WARN("(%p)->(%s,%p),not found\n", This, debugstr_guid(riid), obj);
//         *obj = NULL;
//         return E_NOINTERFACE;
//     }
//
//     IMultiLanguage3_AddRef(iface);
//     return S_OK;
// }
//
// static ULONG WINAPI fnIMultiLanguage3_AddRef( IMultiLanguage3* iface )
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//     return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI fnIMultiLanguage3_Release( IMultiLanguage3* iface )
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("(%p)->(%d)\n", This, ref);
//     if (ref == 0)
//     {
//         HeapFree(GetProcessHeap(), 0, This);
//         UnlockModule();
//     }
//
//     return ref;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetNumberOfCodePageInfo(
//     IMultiLanguage3* iface,
//     UINT* pcCodePage)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p, %p\n", This, pcCodePage);
//
//     if (!pcCodePage) return E_INVALIDARG;
//
//     *pcCodePage = This->total_cp;
//     return S_OK;
// }
//
// static void fill_cp_info(const struct mlang_data *ml_data, UINT index, MIMECPINFO *mime_cp_info)
// {
//     CHARSETINFO csi;
//
//     if (TranslateCharsetInfo((DWORD*)(DWORD_PTR)ml_data->family_codepage, &csi,
//                              TCI_SRCCODEPAGE))
//         mime_cp_info->bGDICharset = csi.ciCharset;
//     else
//         mime_cp_info->bGDICharset = DEFAULT_CHARSET;
//
//     mime_cp_info->dwFlags = ml_data->mime_cp_info[index].flags;
//     mime_cp_info->uiCodePage = ml_data->mime_cp_info[index].cp;
//     mime_cp_info->uiFamilyCodePage = ml_data->family_codepage;
//     MultiByteToWideChar(CP_ACP, 0, ml_data->mime_cp_info[index].description, -1,
//                         mime_cp_info->wszDescription, ARRAY_SIZE(mime_cp_info->wszDescription));
//     MultiByteToWideChar(CP_ACP, 0, ml_data->mime_cp_info[index].web_charset, -1,
//                         mime_cp_info->wszWebCharset, ARRAY_SIZE(mime_cp_info->wszWebCharset));
//     MultiByteToWideChar(CP_ACP, 0, ml_data->mime_cp_info[index].header_charset, -1,
//                         mime_cp_info->wszHeaderCharset, ARRAY_SIZE(mime_cp_info->wszHeaderCharset));
//     MultiByteToWideChar(CP_ACP, 0, ml_data->mime_cp_info[index].body_charset, -1,
//                         mime_cp_info->wszBodyCharset, ARRAY_SIZE(mime_cp_info->wszBodyCharset));
//
//     MultiByteToWideChar(CP_ACP, 0, ml_data->fixed_font, -1,
//         mime_cp_info->wszFixedWidthFont, ARRAY_SIZE(mime_cp_info->wszFixedWidthFont));
//     MultiByteToWideChar(CP_ACP, 0, ml_data->proportional_font, -1,
//         mime_cp_info->wszProportionalFont, ARRAY_SIZE(mime_cp_info->wszProportionalFont));
//
//     TRACE("%08x %u %u %s %s %s %s %s %s %d\n",
//           mime_cp_info->dwFlags, mime_cp_info->uiCodePage,
//           mime_cp_info->uiFamilyCodePage,
//           wine_dbgstr_w(mime_cp_info->wszDescription),
//           wine_dbgstr_w(mime_cp_info->wszWebCharset),
//           wine_dbgstr_w(mime_cp_info->wszHeaderCharset),
//           wine_dbgstr_w(mime_cp_info->wszBodyCharset),
//           wine_dbgstr_w(mime_cp_info->wszFixedWidthFont),
//           wine_dbgstr_w(mime_cp_info->wszProportionalFont),
//           mime_cp_info->bGDICharset);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetCodePageInfo(
//     IMultiLanguage3* iface,
//     UINT uiCodePage,
//     LANGID LangId,
//     PMIMECPINFO pCodePageInfo)
// {
//     UINT i, n;
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p, %u, %04x, %p\n", This, uiCodePage, LangId, pCodePageInfo);
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].cp == uiCodePage)
//             {
//                 fill_cp_info(&mlang_data[i], n, pCodePageInfo);
//                 return S_OK;
//             }
//         }
//     }
//
//     return S_FALSE;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetFamilyCodePage(
//     IMultiLanguage3* iface,
//     UINT uiCodePage,
//     UINT* puiFamilyCodePage)
// {
//     return GetFamilyCodePage(uiCodePage, puiFamilyCodePage);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_EnumCodePages(
//     IMultiLanguage3* iface,
//     DWORD grfFlags,
//     LANGID LangId,
//     IEnumCodePage** ppEnumCodePage)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %08x %04x %p\n", This, grfFlags, LangId, ppEnumCodePage);
//
//     return EnumCodePage_create( This, grfFlags, LangId, ppEnumCodePage );
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetCharsetInfo(
//     IMultiLanguage3* iface,
//     BSTR Charset,
//     PMIMECSETINFO pCharsetInfo)
// {
//     UINT i, n;
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %s %p\n", This, debugstr_w(Charset), pCharsetInfo);
//
//     if (!pCharsetInfo) return E_FAIL;
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             WCHAR csetW[MAX_MIMECSET_NAME];
//
//             MultiByteToWideChar(CP_ACP, 0, mlang_data[i].mime_cp_info[n].web_charset, -1, csetW, MAX_MIMECSET_NAME);
//             if (!lstrcmpiW(Charset, csetW))
//             {
//                 pCharsetInfo->uiCodePage = mlang_data[i].family_codepage;
//                 pCharsetInfo->uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
//                 lstrcpyW(pCharsetInfo->wszCharset, csetW);
//                 return S_OK;
//             }
//             if (mlang_data[i].mime_cp_info[n].alias && !lstrcmpiW(Charset, mlang_data[i].mime_cp_info[n].alias))
//             {
//                 pCharsetInfo->uiCodePage = mlang_data[i].family_codepage;
//                 pCharsetInfo->uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
//                 lstrcpyW(pCharsetInfo->wszCharset, mlang_data[i].mime_cp_info[n].alias);
//                 return S_OK;
//             }
//         }
//     }
//
//     /* FIXME:
//      * Since we do not support charsets like iso-2022-jp and do not have
//      * them in our database as a primary (web_charset) encoding this loop
//      * does an attempt to 'approximate' charset name by header_charset.
//      */
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             WCHAR csetW[MAX_MIMECSET_NAME];
//
//             MultiByteToWideChar(CP_ACP, 0, mlang_data[i].mime_cp_info[n].header_charset, -1, csetW, MAX_MIMECSET_NAME);
//             if (!lstrcmpiW(Charset, csetW))
//             {
//                 pCharsetInfo->uiCodePage = mlang_data[i].family_codepage;
//                 pCharsetInfo->uiInternetEncoding = mlang_data[i].mime_cp_info[n].cp;
//                 lstrcpyW(pCharsetInfo->wszCharset, csetW);
//                 return S_OK;
//             }
//         }
//     }
//
//     return E_FAIL;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_IsConvertible(
//     IMultiLanguage3* iface,
//     DWORD dwSrcEncoding,
//     DWORD dwDstEncoding)
// {
//     return IsConvertINetStringAvailable(dwSrcEncoding, dwDstEncoding);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertString(
//     IMultiLanguage3* iface,
//     DWORD* pdwMode,
//     DWORD dwSrcEncoding,
//     DWORD dwDstEncoding,
//     BYTE* pSrcStr,
//     UINT* pcSrcSize,
//     BYTE* pDstStr,
//     UINT* pcDstSize)
// {
//     return ConvertINetString(pdwMode, dwSrcEncoding, dwDstEncoding,
//         (LPCSTR)pSrcStr, (LPINT)pcSrcSize, (LPSTR)pDstStr, (LPINT)pcDstSize);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringToUnicode(
//     IMultiLanguage3* iface,
//     DWORD* pdwMode,
//     DWORD dwEncoding,
//     CHAR* pSrcStr,
//     UINT* pcSrcSize,
//     WCHAR* pDstStr,
//     UINT* pcDstSize)
// {
//     return ConvertINetMultiByteToUnicode(pdwMode, dwEncoding,
//         pSrcStr, (LPINT)pcSrcSize, pDstStr, (LPINT)pcDstSize);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringFromUnicode(
//     IMultiLanguage3* iface,
//     DWORD* pdwMode,
//     DWORD dwEncoding,
//     WCHAR* pSrcStr,
//     UINT* pcSrcSize,
//     CHAR* pDstStr,
//     UINT* pcDstSize)
// {
//     return ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding,
//         pSrcStr, (LPINT)pcSrcSize, pDstStr, (LPINT)pcDstSize);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringReset(
//     IMultiLanguage3* iface)
// {
//     FIXME("\n");
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetRfc1766FromLcid(
//     IMultiLanguage3* iface,
//     LCID lcid,
//     BSTR* pbstrRfc1766)
// {
//     WCHAR buf[MAX_RFC1766_NAME];
//
//     TRACE("%p %04x %p\n", iface, lcid, pbstrRfc1766);
//     if (!pbstrRfc1766)
//         return E_INVALIDARG;
//
//     if (!lcid_to_rfc1766W( lcid, buf, MAX_RFC1766_NAME ))
//     {
//         *pbstrRfc1766 = SysAllocString( buf );
//         return S_OK;
//     }
//     return E_FAIL;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetLcidFromRfc1766(
//     IMultiLanguage3* iface,
//     LCID* pLocale,
//     BSTR bstrRfc1766)
// {
//     HRESULT hr;
//     IEnumRfc1766 *rfc1766;
//
//     TRACE("%p %p %s\n", iface, pLocale, debugstr_w(bstrRfc1766));
//
//     if (!pLocale || !bstrRfc1766)
//         return E_INVALIDARG;
//
//     hr = IMultiLanguage3_EnumRfc1766(iface, 0, &rfc1766);
//     if (FAILED(hr))
//         return hr;
//
//     hr = lcid_from_rfc1766(rfc1766, pLocale, bstrRfc1766);
//
//     IEnumRfc1766_Release(rfc1766);
//     return hr;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_EnumRfc1766(
//     IMultiLanguage3* iface,
//     LANGID LangId,
//     IEnumRfc1766** ppEnumRfc1766)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %p\n", This, ppEnumRfc1766);
//
//     return EnumRfc1766_create(LangId, ppEnumRfc1766);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetRfc1766Info(
//     IMultiLanguage3* iface,
//     LCID Locale,
//     LANGID LangId,
//     PRFC1766INFO pRfc1766Info)
// {
//     static LANGID last_lang = -1;
//     LCTYPE type = LOCALE_SLANGUAGE;
//
//     TRACE("(%p, 0x%04x, 0x%04x, %p)\n", iface, Locale, LangId, pRfc1766Info);
//
//     if (!pRfc1766Info)
//         return E_INVALIDARG;
//
//     if ((PRIMARYLANGID(Locale) == LANG_ENGLISH) ||
//         (PRIMARYLANGID(Locale) == LANG_CHINESE) ||
//         (PRIMARYLANGID(Locale) == LANG_ARABIC)) {
//
//         if (!SUBLANGID(Locale))
//             type = LOCALE_SENGLANGUAGE; /* suppress country */
//     }
//     else
//     {
//         if (!SUBLANGID(Locale)) {
//             TRACE("SUBLANGID missing in 0x%04x\n", Locale);
//             return E_FAIL;
//         }
//     }
//
//     pRfc1766Info->lcid = Locale;
//     pRfc1766Info->wszRfc1766[0] = 0;
//     pRfc1766Info->wszLocaleName[0] = 0;
//
//     if ((PRIMARYLANGID(LangId) != LANG_ENGLISH) &&
//         (last_lang != LangId)) {
//         FIXME("Only English names supported (requested: 0x%04x)\n", LangId);
//         last_lang = LangId;
//     }
//
//     if ((!lcid_to_rfc1766W(Locale, pRfc1766Info->wszRfc1766, MAX_RFC1766_NAME)) &&
//         (GetLocaleInfoW(Locale, type, pRfc1766Info->wszLocaleName, MAX_LOCALE_NAME) > 0))
//             return S_OK;
//
//     /* Locale not supported */
//     return E_INVALIDARG;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_CreateConvertCharset(
//     IMultiLanguage3* iface,
//     UINT src_cp,
//     UINT dst_cp,
//     DWORD prop,
//     IMLangConvertCharset** convert_charset)
// {
//     HRESULT hr;
//
//     TRACE("(%u %u 0x%08x %p)\n", src_cp, dst_cp, prop, convert_charset);
//
//     hr = MLangConvertCharset_create(NULL, (void**)convert_charset);
//     if (FAILED(hr)) return hr;
//
//     return IMLangConvertCharset_Initialize(*convert_charset, src_cp, dst_cp, prop);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringInIStream(
//     IMultiLanguage3* iface,
//     DWORD* pdwMode,
//     DWORD dwFlag,
//     WCHAR* lpFallBack,
//     DWORD dwSrcEncoding,
//     DWORD dwDstEncoding,
//     IStream* pstmIn,
//     IStream* pstmOut)
// {
//     char *src, *dst = NULL;
//     INT srclen, dstlen;
//     STATSTG stat;
//     HRESULT hr;
//
//     TRACE("%p %0x8 %s %u %u %p %p\n",
//           pdwMode, dwFlag, debugstr_w(lpFallBack), dwSrcEncoding, dwDstEncoding, pstmIn, pstmOut);
//
//     FIXME("dwFlag and lpFallBack not handled\n");
//
//     hr = IStream_Stat(pstmIn, &stat, STATFLAG_NONAME);
//     if (FAILED(hr)) return hr;
//
//     if (stat.cbSize.QuadPart > MAXLONG) return E_INVALIDARG;
//     if (!(src = HeapAlloc(GetProcessHeap(), 0, stat.cbSize.QuadPart))) return E_OUTOFMEMORY;
//
//     hr = IStream_Read(pstmIn, src, stat.cbSize.QuadPart, (ULONG *)&srclen);
//     if (FAILED(hr)) goto exit;
//
//     hr = ConvertINetString(pdwMode, dwSrcEncoding, dwDstEncoding, src, &srclen, NULL, &dstlen);
//     if (FAILED(hr)) goto exit;
//
//     if (!(dst = HeapAlloc(GetProcessHeap(), 0, dstlen)))
//     {
//         hr = E_OUTOFMEMORY;
//         goto exit;
//     }
//     hr = ConvertINetString(pdwMode, dwSrcEncoding, dwDstEncoding, src, &srclen, dst, &dstlen);
//     if (FAILED(hr)) goto exit;
//
//     hr = IStream_Write(pstmOut, dst, dstlen, NULL);
//
// exit:
//     HeapFree(GetProcessHeap(), 0, src);
//     HeapFree(GetProcessHeap(), 0, dst);
//     return hr;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringToUnicodeEx(
//     IMultiLanguage3* iface,
//     DWORD* pdwMode,
//     DWORD dwEncoding,
//     CHAR* pSrcStr,
//     UINT* pcSrcSize,
//     WCHAR* pDstStr,
//     UINT* pcDstSize,
//     DWORD dwFlag,
//     WCHAR* lpFallBack)
// {
//     if (dwFlag || lpFallBack)
//         FIXME("Ignoring dwFlag (0x%x/%d) and lpFallBack (%p)\n",
//                 dwFlag, dwFlag, lpFallBack);
//
//     return ConvertINetMultiByteToUnicode(pdwMode, dwEncoding,
//         pSrcStr, (LPINT)pcSrcSize, pDstStr, (LPINT)pcDstSize);
// }
//
// /*****************************************************************************
//  * MultiLanguage2::ConvertStringToUnicodeEx
//  *
//  * Translates the multibyte string from the specified code page to Unicode.
//  *
//  * PARAMS
//  *   see ConvertStringToUnicode
//  *   dwFlag
//  *   lpFallBack if dwFlag contains MLCONVCHARF_USEDEFCHAR, lpFallBack string used
//  *              instead unconvertible characters.
//  *
//  * RETURNS
//  *   S_OK     Success.
//  *   S_FALSE  The conversion is not supported.
//  *   E_FAIL   Some error has occurred.
//  *
//  * TODO: handle dwFlag and lpFallBack
// */
// static HRESULT WINAPI fnIMultiLanguage3_ConvertStringFromUnicodeEx(
//     IMultiLanguage3* This,
//     DWORD* pdwMode,
//     DWORD dwEncoding,
//     WCHAR* pSrcStr,
//     UINT* pcSrcSize,
//     CHAR* pDstStr,
//     UINT* pcDstSize,
//     DWORD dwFlag,
//     WCHAR* lpFallBack)
// {
//     FIXME("\n");
//     return ConvertINetUnicodeToMultiByte(pdwMode, dwEncoding,
//         pSrcStr, (LPINT)pcSrcSize, pDstStr, (LPINT)pcDstSize);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_DetectCodepageInIStream(
//     IMultiLanguage3* iface,
//     DWORD dwFlag,
//     DWORD dwPrefWinCodePage,
//     IStream* pstmIn,
//     DetectEncodingInfo* lpEncoding,
//     INT* pnScores)
// {
//     FIXME("\n");
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_DetectInputCodepage(
//     IMultiLanguage3* iface,
//     DWORD dwFlag,
//     DWORD dwPrefWinCodePage,
//     CHAR* pSrcStr,
//     INT* pcSrcSize,
//     DetectEncodingInfo* lpEncoding,
//     INT* pnScores)
// {
//     FIXME("\n");
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ValidateCodePage(
//     IMultiLanguage3* iface,
//     UINT uiCodePage,
//     HWND hwnd)
// {
//     return IMultiLanguage3_ValidateCodePageEx(iface,uiCodePage,hwnd,0);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetCodePageDescription(
//     IMultiLanguage3* iface,
//     UINT uiCodePage,
//     LCID lcid,
//     LPWSTR lpWideCharStr,
//     int cchWideChar)
// {
//     /* Find first instance */
//     unsigned int i,n;
//
//     TRACE ("%u, %04x, %p, %d\n", uiCodePage, lcid, lpWideCharStr, cchWideChar);
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].cp == uiCodePage)
//             {
//                 MultiByteToWideChar(CP_ACP, 0,
//                                     mlang_data[i].mime_cp_info[n].description,
//                                     -1, lpWideCharStr, cchWideChar);
//                 return S_OK;
//             }
//         }
//     }
//
//     return S_FALSE;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_IsCodePageInstallable(
//     IMultiLanguage3* iface,
//     UINT uiCodePage)
// {
//     TRACE("%u\n", uiCodePage);
//
//     /* FIXME: the installable set is usually larger than the set of valid codepages */
//     return IMultiLanguage3_ValidateCodePageEx(iface, uiCodePage, NULL, CPIOD_PEEK);
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_SetMimeDBSource(
//     IMultiLanguage3* iface,
//     MIMECONTF dwSource)
// {
//     FIXME("0x%08x\n", dwSource);
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_GetNumberOfScripts(
//     IMultiLanguage3* iface,
//     UINT* pnScripts)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %p\n", This, pnScripts);
//
//     if (!pnScripts) return S_FALSE;
//
//     *pnScripts = This->total_scripts;
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_EnumScripts(
//     IMultiLanguage3* iface,
//     DWORD dwFlags,
//     LANGID LangId,
//     IEnumScript** ppEnumScript)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %08x %04x %p\n", This, dwFlags, LangId, ppEnumScript);
//
//     return EnumScript_create( This, dwFlags, LangId, ppEnumScript );
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_ValidateCodePageEx(
//     IMultiLanguage3* iface,
//     UINT uiCodePage,
//     HWND hwnd,
//     DWORD dwfIODControl)
// {
//     unsigned int i;
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     TRACE("%p %u %p %08x\n", This, uiCodePage, hwnd, dwfIODControl);
//
//     /* quick check for kernel32 supported code pages */
//     if (IsValidCodePage(uiCodePage))
//         return S_OK;
//
//     /* check for mlang supported code pages */
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         UINT n;
//         for (n = 0; n < mlang_data[i].number_of_cp; n++)
//         {
//             if (mlang_data[i].mime_cp_info[n].cp == uiCodePage)
//                 return S_OK;
//         }
//     }
//
//     if (dwfIODControl != CPIOD_PEEK)
//         FIXME("Request to install codepage language pack not handled\n");
//
//     return S_FALSE;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_DetectOutboundCodePage(
//     IMultiLanguage3 *iface,
//     DWORD dwFlags,
//     LPCWSTR lpWideCharStr,
//     UINT cchWideChar,
//     UINT *puiPreferredCodePages,
//     UINT nPreferredCodePages,
//     UINT *puiDetectedCodePages,
//     UINT *pnDetectedCodePages,
//     WCHAR *lpSpecialChar)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     FIXME("(%p)->(%08x %s %p %u %p %p(%u) %s)\n", This, dwFlags, debugstr_w(lpWideCharStr),
//           puiPreferredCodePages, nPreferredCodePages, puiDetectedCodePages,
//           pnDetectedCodePages, pnDetectedCodePages ? *pnDetectedCodePages : 0,
//           debugstr_w(lpSpecialChar));
//
//     if (!puiDetectedCodePages || !pnDetectedCodePages || !*pnDetectedCodePages)
//         return E_INVALIDARG;
//
//     puiDetectedCodePages[0] = CP_UTF8;
//     *pnDetectedCodePages = 1;
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMultiLanguage3_DetectOutboundCodePageInIStream(
//     IMultiLanguage3 *iface,
//     DWORD dwFlags,
//     IStream *pStrIn,
//     UINT *puiPreferredCodePages,
//     UINT nPreferredCodePages,
//     UINT *puiDetectedCodePages,
//     UINT *pnDetectedCodePages,
//     WCHAR *lpSpecialChar)
// {
//     MLang_impl *This = impl_from_IMultiLanguage3( iface );
//
//     FIXME("(%p)->(%08x %p %p %u %p %p(%u) %s)\n", This, dwFlags, pStrIn,
//           puiPreferredCodePages, nPreferredCodePages, puiDetectedCodePages,
//           pnDetectedCodePages, pnDetectedCodePages ? *pnDetectedCodePages : 0,
//           debugstr_w(lpSpecialChar));
//
//     if (!puiDetectedCodePages || !pnDetectedCodePages || !*pnDetectedCodePages)
//         return E_INVALIDARG;
//
//     puiDetectedCodePages[0] = CP_UTF8;
//     *pnDetectedCodePages = 1;
//     return S_OK;
// }
//
// static const IMultiLanguage3Vtbl IMultiLanguage3_vtbl =
// {
//     fnIMultiLanguage3_QueryInterface,
//     fnIMultiLanguage3_AddRef,
//     fnIMultiLanguage3_Release,
//     fnIMultiLanguage3_GetNumberOfCodePageInfo,
//     fnIMultiLanguage3_GetCodePageInfo,
//     fnIMultiLanguage3_GetFamilyCodePage,
//     fnIMultiLanguage3_EnumCodePages,
//     fnIMultiLanguage3_GetCharsetInfo,
//     fnIMultiLanguage3_IsConvertible,
//     fnIMultiLanguage3_ConvertString,
//     fnIMultiLanguage3_ConvertStringToUnicode,
//     fnIMultiLanguage3_ConvertStringFromUnicode,
//     fnIMultiLanguage3_ConvertStringReset,
//     fnIMultiLanguage3_GetRfc1766FromLcid,
//     fnIMultiLanguage3_GetLcidFromRfc1766,
//     fnIMultiLanguage3_EnumRfc1766,
//     fnIMultiLanguage3_GetRfc1766Info,
//     fnIMultiLanguage3_CreateConvertCharset,
//     fnIMultiLanguage3_ConvertStringInIStream,
//     fnIMultiLanguage3_ConvertStringToUnicodeEx,
//     fnIMultiLanguage3_ConvertStringFromUnicodeEx,
//     fnIMultiLanguage3_DetectCodepageInIStream,
//     fnIMultiLanguage3_DetectInputCodepage,
//     fnIMultiLanguage3_ValidateCodePage,
//     fnIMultiLanguage3_GetCodePageDescription,
//     fnIMultiLanguage3_IsCodePageInstallable,
//     fnIMultiLanguage3_SetMimeDBSource,
//     fnIMultiLanguage3_GetNumberOfScripts,
//     fnIMultiLanguage3_EnumScripts,
//     fnIMultiLanguage3_ValidateCodePageEx,
//     fnIMultiLanguage3_DetectOutboundCodePage,
//     fnIMultiLanguage3_DetectOutboundCodePageInIStream
// };
//
// /******************************************************************************/
//
// static inline MLang_impl *impl_from_IMLangFontLink2( IMLangFontLink2 *iface )
// {
//     return CONTAINING_RECORD( iface, MLang_impl, IMLangFontLink2_iface );
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_QueryInterface(
//     IMLangFontLink2 * iface,
//     REFIID riid,
//     void** ppvObject)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2( iface );
//     return IMultiLanguage3_QueryInterface( &This->IMultiLanguage3_iface, riid, ppvObject );
// }
//
// static ULONG WINAPI fnIMLangFontLink2_AddRef( IMLangFontLink2* iface )
// {
//     MLang_impl *This = impl_from_IMLangFontLink2( iface );
//     return IMultiLanguage3_AddRef( &This->IMultiLanguage3_iface );
// }
//
// static ULONG WINAPI fnIMLangFontLink2_Release( IMLangFontLink2* iface )
// {
//     MLang_impl *This = impl_from_IMLangFontLink2( iface );
//     return IMultiLanguage3_Release( &This->IMultiLanguage3_iface );
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_GetCharCodePages( IMLangFontLink2* iface,
//         WCHAR ch_src, DWORD *ret_codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2(iface);
//     unsigned int i;
//
//     TRACE("(%p)->(%s %p)\n", This, debugstr_wn(&ch_src, 1), ret_codepages);
//
//     *ret_codepages = 0;
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         BOOL used_dc;
//         CHAR buf;
//
//         WideCharToMultiByte(mlang_data[i].family_codepage, WC_NO_BEST_FIT_CHARS,
//             &ch_src, 1, &buf, 1, NULL, &used_dc);
//
//         /* If default char is not used, current codepage include the given symbol */
//         if (!used_dc)
//         {
//             DWORD codepages;
//
//             IMLangFontLink2_CodePageToCodePages(iface,
//                 mlang_data[i].family_codepage, &codepages);
//             *ret_codepages |= codepages;
//         }
//     }
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_GetStrCodePages( IMLangFontLink2* iface,
//         const WCHAR *src, LONG src_len, DWORD priority_cp,
//         DWORD *codepages, LONG *ret_len)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2(iface);
//     LONG i;
//     DWORD cps = 0;
//
//     TRACE("(%p)->(%s:%d %x %p %p)\n", This, debugstr_wn(src, src_len), src_len, priority_cp,
//         codepages, ret_len);
//
//     if (codepages) *codepages = 0;
//     if (ret_len) *ret_len = 0;
//
//     if (!src || !src_len || src_len < 0)
//         return E_INVALIDARG;
//
//     for (i = 0; i < src_len; i++)
//     {
//         DWORD cp;
//         HRESULT ret;
//
//         ret = IMLangFontLink2_GetCharCodePages(iface, src[i], &cp);
//         if (ret != S_OK) return E_FAIL;
//
//         if (!cps) cps = cp;
//         else cps &= cp;
//
//         /* FIXME: not tested */
//         if (priority_cp & cps) break;
//     }
//
//     if (codepages) *codepages = cps;
//     if (ret_len) *ret_len = min( i + 1, src_len );
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_CodePageToCodePages(IMLangFontLink2* iface,
//         UINT codepage,
//         DWORD *codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2(iface);
//     CHARSETINFO cs;
//     BOOL rc;
//
//     TRACE("(%p)->(%u %p)\n", This, codepage, codepages);
//
//     rc = TranslateCharsetInfo((DWORD*)(DWORD_PTR)codepage, &cs, TCI_SRCCODEPAGE);
//     if (rc)
//     {
//         *codepages = cs.fs.fsCsb[0];
//         TRACE("resulting codepages 0x%x\n", *codepages);
//         return S_OK;
//     }
//
//     TRACE("codepage not found\n");
//     *codepages = 0;
//     return E_FAIL;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_CodePagesToCodePage(IMLangFontLink2* iface,
//         DWORD codepages, UINT def_codepage, UINT *codepage)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2(iface);
//     DWORD mask = 0;
//     CHARSETINFO cs;
//     BOOL rc;
//     UINT i;
//
//     TRACE("(%p)->(0x%x %u %p)\n", This, codepages, def_codepage, codepage);
//
//     *codepage = 0;
//
//     rc = TranslateCharsetInfo((DWORD*)(DWORD_PTR)def_codepage, &cs, TCI_SRCCODEPAGE);
//     if (rc && (codepages & cs.fs.fsCsb[0]))
//     {
//         TRACE("Found Default Codepage\n");
//         *codepage = def_codepage;
//         return S_OK;
//     }
//
//     for (i = 0; i < 32; i++)
//     {
//         mask = 1 << i;
//         if (codepages & mask)
//         {
//             DWORD Csb[2];
//             Csb[0] = mask;
//             Csb[1] = 0x0;
//             rc = TranslateCharsetInfo(Csb, &cs, TCI_SRCFONTSIG);
//             if (!rc)
//                 continue;
//
//             TRACE("Falling back to least significant found CodePage %u\n",
//                     cs.ciACP);
//             *codepage = cs.ciACP;
//             return S_OK;
//         }
//     }
//
//     TRACE("no codepage found\n");
//     return E_FAIL;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_GetFontCodePages(IMLangFontLink2 *iface,
//         HDC hdc, HFONT hfont, DWORD *codepages)
// {
//     MLang_impl *This = impl_from_IMLangFontLink2(iface);
//     FONTSIGNATURE fontsig;
//     HFONT old_font;
//
//     TRACE("(%p)->(%p %p %p)\n", This, hdc, hfont, codepages);
//
//     old_font = SelectObject(hdc, hfont);
//     GetTextCharsetInfo(hdc, &fontsig, 0);
//     SelectObject(hdc, old_font);
//
//     *codepages = fontsig.fsCsb[0];
//     TRACE("ret 0x%x\n", fontsig.fsCsb[0]);
//
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_ReleaseFont(IMLangFontLink2* This,
//         HFONT hFont)
// {
//     TRACE("(%p)->%p\n",This, hFont);
//
//     return release_font(hFont);
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_ResetFontMapping(IMLangFontLink2* This)
// {
//     TRACE("(%p)\n",This);
//
//     return clear_font_cache();
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_MapFont(IMLangFontLink2* This,
//         HDC hDC, DWORD dwCodePages, WCHAR chSrc, HFONT *pFont)
// {
//     HFONT old_font;
//
//     TRACE("(%p)->%p %08x %04x %p\n",This, hDC, dwCodePages, chSrc, pFont);
//
//     if (!hDC) return E_FAIL;
//
//     if (dwCodePages != 0)
//     {
//         old_font = GetCurrentObject(hDC, OBJ_FONT);
//         return map_font(hDC, dwCodePages, old_font, pFont);
//     }
//     else
//     {
//         if (pFont == NULL) return E_INVALIDARG;
//         FIXME("the situation where dwCodepages is set to zero is not implemented\n");
//         return E_FAIL;
//     }
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_GetFontUnicodeRanges(IMLangFontLink2* This,
//         HDC hDC, UINT *puiRanges, UNICODERANGE *pUranges)
// {
//     DWORD size;
//     GLYPHSET *gs;
//
//     TRACE("(%p)->%p %p %p\n", This, hDC, puiRanges, pUranges);
//
//     if (!puiRanges) return E_INVALIDARG;
//     if (!(size = GetFontUnicodeRanges(hDC, NULL))) return E_FAIL;
//     if (!(gs = HeapAlloc(GetProcessHeap(), 0, size))) return E_OUTOFMEMORY;
//
//     GetFontUnicodeRanges(hDC, gs);
//     *puiRanges = gs->cRanges;
//     if (pUranges)
//     {
//         UINT i;
//         for (i = 0; i < gs->cRanges; i++)
//         {
//             if (i >= *puiRanges) break;
//             pUranges[i].wcFrom = gs->ranges[i].wcLow;
//             pUranges[i].wcTo   = gs->ranges[i].wcLow + gs->ranges[i].cGlyphs;
//         }
//         *puiRanges = i;
//     }
//     HeapFree(GetProcessHeap(), 0, gs);
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_GetScriptFontInfo(IMLangFontLink2* This,
//         SCRIPT_ID sid, DWORD dwFlags, UINT *puiFonts,
//         SCRIPTFONTINFO *pScriptFont)
// {
//     UINT i, j;
//
//     TRACE("(%p)->%u %x %p %p\n", This, sid, dwFlags, puiFonts, pScriptFont);
//
//     if (!dwFlags) dwFlags = SCRIPTCONTF_PROPORTIONAL_FONT;
//
//     for (i = 0, j = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         if (sid == mlang_data[i].sid)
//         {
//             if (pScriptFont)
//             {
//                 if (j >= *puiFonts) break;
//
//                 pScriptFont[j].scripts = (SCRIPT_IDS)1 << mlang_data[i].sid;
//                 if (dwFlags == SCRIPTCONTF_FIXED_FONT)
//                 {
//                     MultiByteToWideChar(CP_ACP, 0, mlang_data[i].fixed_font, -1,
//                         pScriptFont[j].wszFont, MAX_MIMEFACE_NAME);
//                 }
//                 else if (dwFlags == SCRIPTCONTF_PROPORTIONAL_FONT)
//                 {
//                     MultiByteToWideChar(CP_ACP, 0, mlang_data[i].proportional_font, -1,
//                         pScriptFont[j].wszFont, MAX_MIMEFACE_NAME);
//                 }
//             }
//             j++;
//         }
//     }
//     *puiFonts = j;
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangFontLink2_CodePageToScriptID(IMLangFontLink2* This,
//         UINT uiCodePage, SCRIPT_ID *pSid)
// {
//     UINT i;
//
//     TRACE("(%p)->%i %p\n", This, uiCodePage, pSid);
//
//     if (uiCodePage == CP_UNICODE) return E_FAIL;
//
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//     {
//         if (uiCodePage == mlang_data[i].family_codepage)
//         {
//             if (pSid) *pSid = mlang_data[i].sid;
//             return S_OK;
//         }
//     }
//     return E_FAIL;
// }
//
// static const IMLangFontLink2Vtbl IMLangFontLink2_vtbl =
// {
//     fnIMLangFontLink2_QueryInterface,
//     fnIMLangFontLink2_AddRef,
//     fnIMLangFontLink2_Release,
//     fnIMLangFontLink2_GetCharCodePages,
//     fnIMLangFontLink2_GetStrCodePages,
//     fnIMLangFontLink2_CodePageToCodePages,
//     fnIMLangFontLink2_CodePagesToCodePage,
//     fnIMLangFontLink2_GetFontCodePages,
//     fnIMLangFontLink2_ReleaseFont,
//     fnIMLangFontLink2_ResetFontMapping,
//     fnIMLangFontLink2_MapFont,
//     fnIMLangFontLink2_GetFontUnicodeRanges,
//     fnIMLangFontLink2_GetScriptFontInfo,
//     fnIMLangFontLink2_CodePageToScriptID
// };
//
// /******************************************************************************/
//
// static inline MLang_impl *impl_from_IMLangLineBreakConsole( IMLangLineBreakConsole *iface )
// {
//     return CONTAINING_RECORD( iface, MLang_impl, IMLangLineBreakConsole_iface );
// }
//
// static HRESULT WINAPI fnIMLangLineBreakConsole_QueryInterface(
//     IMLangLineBreakConsole* iface,
//     REFIID riid,
//     void** ppvObject)
// {
//     MLang_impl *This = impl_from_IMLangLineBreakConsole( iface );
//     return IMultiLanguage3_QueryInterface( &This->IMultiLanguage3_iface, riid, ppvObject );
// }
//
// static ULONG WINAPI fnIMLangLineBreakConsole_AddRef(
//     IMLangLineBreakConsole* iface )
// {
//     MLang_impl *This = impl_from_IMLangLineBreakConsole( iface );
//     return IMultiLanguage3_AddRef( &This->IMultiLanguage3_iface );
// }
//
// static ULONG WINAPI fnIMLangLineBreakConsole_Release(
//     IMLangLineBreakConsole* iface )
// {
//     MLang_impl *This = impl_from_IMLangLineBreakConsole( iface );
//     return IMultiLanguage3_Release( &This->IMultiLanguage3_iface );
// }
//
// static HRESULT WINAPI fnIMLangLineBreakConsole_BreakLineML(
//     IMLangLineBreakConsole* iface,
//     IMLangString* pSrcMLStr,
//     LONG lSrcPos,
//     LONG lSrcLen,
//     LONG cMinColumns,
//     LONG cMaxColumns,
//     LONG* plLineLen,
//     LONG* plSkipLen)
// {
//     FIXME("(%p)->%p %i %i %i %i %p %p\n", iface, pSrcMLStr, lSrcPos, lSrcLen, cMinColumns, cMaxColumns, plLineLen, plSkipLen);
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI fnIMLangLineBreakConsole_BreakLineW(
//     IMLangLineBreakConsole* iface,
//     LCID locale,
//     const WCHAR* pszSrc,
//     LONG cchSrc,
//     LONG cMaxColumns,
//     LONG* pcchLine,
//     LONG* pcchSkip )
// {
//     FIXME("(%p)->%i %s %i %i %p %p\n", iface, locale, debugstr_wn(pszSrc,cchSrc), cchSrc, cMaxColumns, pcchLine, pcchSkip);
//
//     *pcchLine = cchSrc;
//     *pcchSkip = 0;
//     return S_OK;
// }
//
// static HRESULT WINAPI fnIMLangLineBreakConsole_BreakLineA(
//     IMLangLineBreakConsole* iface,
//     LCID locale,
//     UINT uCodePage,
//     const CHAR* pszSrc,
//     LONG cchSrc,
//     LONG cMaxColumns,
//     LONG* pcchLine,
//     LONG* pcchSkip)
// {
//     LONG i, line = cchSrc, skip = 0;
//
//     FIXME("(%p)->%i %i %s %i %i %p %p\n", iface, locale, uCodePage, debugstr_an(pszSrc,cchSrc), cchSrc, cMaxColumns, pcchLine, pcchSkip);
//
//     if (uCodePage == CP_USASCII && cchSrc > cMaxColumns)
//     {
//         for (line = cMaxColumns, i = cMaxColumns - 1; i >= 0; i--)
//         {
//             if (pszSrc[i] == ' ')
//             {
//                 while (i >= 0 && pszSrc[i] == ' ')
//                 {
//                     i--;
//                     line--;
//                     skip++;
//                 }
//                 break;
//             }
//         }
//     }
//     *pcchLine = line;
//     *pcchSkip = skip;
//     return S_OK;
// }
//
// static const IMLangLineBreakConsoleVtbl IMLangLineBreakConsole_vtbl =
// {
//     fnIMLangLineBreakConsole_QueryInterface,
//     fnIMLangLineBreakConsole_AddRef,
//     fnIMLangLineBreakConsole_Release,
//     fnIMLangLineBreakConsole_BreakLineML,
//     fnIMLangLineBreakConsole_BreakLineW,
//     fnIMLangLineBreakConsole_BreakLineA
// };
//
// struct convert_charset {
//     IMLangConvertCharset IMLangConvertCharset_iface;
//     LONG ref;
//
//     UINT src_cp;
//     UINT dst_cp;
// };
//
// static inline struct convert_charset *impl_from_IMLangConvertCharset(IMLangConvertCharset *iface)
// {
//     return CONTAINING_RECORD(iface, struct convert_charset, IMLangConvertCharset_iface);
// }
//
// static HRESULT WINAPI MLangConvertCharset_QueryInterface(IMLangConvertCharset *iface, REFIID riid, void **obj)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//
//     TRACE("(%p)->(%s %p)\n", This, debugstr_guid(riid), obj);
//
//     if (IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IMLangConvertCharset))
//     {
//         *obj = &This->IMLangConvertCharset_iface;
//         IMLangConvertCharset_AddRef(iface);
//         return S_OK;
//     }
//
//     *obj = NULL;
//     return E_NOINTERFACE;
// }
//
// static ULONG WINAPI MLangConvertCharset_AddRef(IMLangConvertCharset *iface)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     ULONG ref = InterlockedIncrement(&This->ref);
//     TRACE("(%p)->(%u)\n", This, ref);
//     return ref;
// }
//
// static ULONG WINAPI MLangConvertCharset_Release(IMLangConvertCharset *iface)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     ULONG ref = InterlockedDecrement(&This->ref);
//
//     TRACE("(%p)->(%u)\n", This, ref);
//     if (!ref)
//     {
//         HeapFree(GetProcessHeap(), 0, This);
//         UnlockModule();
//     }
//
//     return ref;
// }
//
// static HRESULT WINAPI MLangConvertCharset_Initialize(IMLangConvertCharset *iface,
//     UINT src_cp, UINT dst_cp, DWORD prop)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//
//     TRACE("(%p)->(%u %u 0x%08x)\n", This, src_cp, dst_cp, prop);
//
//     prop &= ~MLCONVCHARF_USEDEFCHAR;
//     if (prop)
//         FIXME("property 0x%08x not supported\n", prop);
//
//     This->src_cp = src_cp;
//     This->dst_cp = dst_cp;
//
//     return S_OK;
// }
//
// static HRESULT WINAPI MLangConvertCharset_GetSourceCodePage(IMLangConvertCharset *iface, UINT *src_cp)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//
//     TRACE("(%p)->(%p)\n", This, src_cp);
//
//     if (!src_cp) return E_INVALIDARG;
//     *src_cp = This->src_cp;
//     return S_OK;
// }
//
// static HRESULT WINAPI MLangConvertCharset_GetDestinationCodePage(IMLangConvertCharset *iface, UINT *dst_cp)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//
//     TRACE("(%p)->(%p)\n", This, dst_cp);
//
//     if (!dst_cp) return E_INVALIDARG;
//     *dst_cp = This->dst_cp;
//     return S_OK;
// }
//
// static HRESULT WINAPI MLangConvertCharset_GetProperty(IMLangConvertCharset *iface, DWORD *prop)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     FIXME("(%p)->(%p): stub\n", This, prop);
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI MLangConvertCharset_DoConversion(IMLangConvertCharset *iface, BYTE *src,
//     UINT *src_size, BYTE *dest, UINT *dest_size)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     FIXME("(%p)->(%p %p %p %p): stub\n", This, src, src_size, dest, dest_size);
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI MLangConvertCharset_DoConversionToUnicode(IMLangConvertCharset *iface, CHAR *src,
//     UINT *src_size, WCHAR *dest, UINT *dest_size)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     TRACE("(%p)->(%p %p %p %p)\n", This, src, src_size, dest, dest_size);
//     return ConvertINetMultiByteToUnicode(NULL, This->src_cp, src, (INT*)src_size, dest, (INT*)dest_size);
// }
//
// static HRESULT WINAPI MLangConvertCharset_DoConversionFromUnicode(IMLangConvertCharset *iface,
//     WCHAR *src, UINT *src_size, CHAR *dest, UINT *dest_size)
// {
//     struct convert_charset *This = impl_from_IMLangConvertCharset(iface);
//     TRACE("(%p)->(%p %p %p %p)\n", This, src, src_size, dest, dest_size);
//     return ConvertINetUnicodeToMultiByte(NULL, This->dst_cp, src, (INT*)src_size, dest, (INT*)dest_size);
// }
//
// static const IMLangConvertCharsetVtbl MLangConvertCharsetVtbl =
// {
//     MLangConvertCharset_QueryInterface,
//     MLangConvertCharset_AddRef,
//     MLangConvertCharset_Release,
//     MLangConvertCharset_Initialize,
//     MLangConvertCharset_GetSourceCodePage,
//     MLangConvertCharset_GetDestinationCodePage,
//     MLangConvertCharset_GetProperty,
//     MLangConvertCharset_DoConversion,
//     MLangConvertCharset_DoConversionToUnicode,
//     MLangConvertCharset_DoConversionFromUnicode
// };
//
// static HRESULT MultiLanguage_create(IUnknown *pUnkOuter, LPVOID *ppObj)
// {
//     MLang_impl *mlang;
//     UINT i;
//
//     TRACE("Creating MultiLanguage object\n");
//
//     if( pUnkOuter )
//         return CLASS_E_NOAGGREGATION;
//
//     mlang = HeapAlloc( GetProcessHeap(), 0, sizeof (MLang_impl) );
//     mlang->IMLangFontLink_iface.lpVtbl = &IMLangFontLink_vtbl;
//     mlang->IMultiLanguage_iface.lpVtbl = &IMultiLanguage_vtbl;
//     mlang->IMultiLanguage3_iface.lpVtbl = &IMultiLanguage3_vtbl;
//     mlang->IMLangFontLink2_iface.lpVtbl = &IMLangFontLink2_vtbl;
//     mlang->IMLangLineBreakConsole_iface.lpVtbl = &IMLangLineBreakConsole_vtbl;
//
//     mlang->total_cp = 0;
//     for (i = 0; i < ARRAY_SIZE(mlang_data); i++)
//         mlang->total_cp += mlang_data[i].number_of_cp;
//
//     /* do not enumerate unicode flavours */
//     mlang->total_scripts = ARRAY_SIZE(mlang_data) - 1;
//
//     mlang->ref = 1;
//     *ppObj = &mlang->IMultiLanguage_iface;
//     TRACE("returning %p\n", mlang);
//
//     LockModule();
//
//     return S_OK;
// }
//
// static HRESULT MLangConvertCharset_create(IUnknown *outer, void **obj)
// {
//     struct convert_charset *convert;
//
//     if (outer)
//         return CLASS_E_NOAGGREGATION;
//
//     *obj = NULL;
//
//     convert = HeapAlloc(GetProcessHeap(), 0, sizeof(struct convert_charset));
//     if (!convert) return E_OUTOFMEMORY;
//
//     convert->IMLangConvertCharset_iface.lpVtbl = &MLangConvertCharsetVtbl;
//     convert->ref = 1;
//
//     *obj = &convert->IMLangConvertCharset_iface;
//
//     LockModule();
//
//     return S_OK;
// }
//
// /******************************************************************************/
//
// HRESULT WINAPI DllCanUnloadNow(void)
// {
//     return dll_count == 0 ? S_OK : S_FALSE;
// }
//
// static BOOL register_codepages(void)
// {
//     const struct mlang_data *family;
//     const MIME_CP_INFO *info;
//     HKEY db_key, key;
//     WCHAR buf[32];
//     LSTATUS status;
//
//     static const WCHAR db_key_nameW[] = {
//         'M','I','M','E',
//         '\\','D','a','t','a','b','a','s','e',
//         '\\','C','o','d','e','p','a','g','e',0};
//     static const WCHAR familyW[] = {'F','a','m','i','l','y',0};
//     static const WCHAR formatW[] = {'%','u',0};
//
//     status = RegCreateKeyW(HKEY_CLASSES_ROOT, db_key_nameW, &db_key);
//     if (status != ERROR_SUCCESS)
//         return FALSE;
//
//     for (family = mlang_data; family < mlang_data + ARRAY_SIZE(mlang_data); family++)
//     {
//         for (info = family->mime_cp_info; info < family->mime_cp_info + family->number_of_cp; info++)
//         {
//             swprintf(buf, formatW, info->cp);
//             status = RegCreateKeyW(db_key, buf, &key);
//             if (status != ERROR_SUCCESS)
//                 continue;
//
//             RegSetValueExA(key, "BodyCharset", 0, REG_SZ, (BYTE*)info->body_charset,
//                            strlen(info->body_charset) + 1);
//
//             if (info->cp == family->family_codepage)
//             {
//                 RegSetValueExA(key, "FixedWidthFont", 0, REG_SZ, (BYTE*)family->fixed_font,
//                                strlen(family->fixed_font) + 1);
//                 RegSetValueExA(key, "ProportionalFont", 0, REG_SZ, (BYTE*)family->proportional_font,
//                                strlen(family->proportional_font) + 1);
//             }
//             else
//             {
//                 RegSetValueExW(key, familyW, 0, REG_DWORD, (BYTE*)&family->family_codepage,
//                                sizeof(family->family_codepage));
//             }
//
//             RegCloseKey(key);
//         }
//     }
//
//     RegCloseKey(db_key);
//     return TRUE;
// }
//
// /***********************************************************************
//  *		DllRegisterServer (MLANG.@)
//  */
// HRESULT WINAPI DllRegisterServer(void)
// {
//     if(!register_codepages())
//         return E_FAIL;
//     return __wine_register_resources( instance );
// }
//
// /***********************************************************************
//  *		DllUnregisterServer (MLANG.@)
//  */
// HRESULT WINAPI DllUnregisterServer(void)
// {
//     return __wine_unregister_resources( instance );
// }
//
// HRESULT WINAPI GetGlobalFontLinkObject(void **unknown)
// {
//     if (!unknown) return E_INVALIDARG;
//
//     FIXME("%p: stub\n", unknown);
//
//     return S_FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
