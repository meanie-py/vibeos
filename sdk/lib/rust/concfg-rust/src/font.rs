use winsrv_sys::winsrv::*;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// RtlGetProcessHeap is a PEB inline (TEB->Peb->ProcessHeap), not an export.
// NtCurrentTeb is exported by both the nt and winsrv globs — qualify it.
#[inline]
unsafe fn RtlGetProcessHeap() -> *mut core::ffi::c_void {
    (*(*winsrv_sys::nt::NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

// ---- concfg/font.h + shared-header helpers (macros/inlines, not exports) ----
pub const INVALID_CP: UINT = 0xFFFFFFFF; // (UINT)-1
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// CJK codepages / charsets (sdk/include/reactos/cjkcode.h).
const CP_SHIFTJIS: UINT = 932;
const CP_HANGUL: UINT = 949;
const CP_GB2312: UINT = 936;
const CP_BIG5: UINT = 950;

#[inline]
unsafe fn IsCJKCodePage(cp: UINT) -> BOOL {
    (cp == CP_SHIFTJIS || cp == CP_HANGUL || cp == CP_BIG5 || cp == CP_GB2312) as BOOL
}
#[inline]
unsafe fn IsCJKCharSet(cs: BYTE) -> BOOL {
    // SHIFTJIS=128, HANGUL=129, GB2312=134, CHINESEBIG5=136
    (cs == 128 || cs == 129 || cs == 134 || cs == 136) as BOOL
}
#[inline]
unsafe fn IsAdditionalTTFont(face: *const WCHAR) -> BOOL {
    (!FindCachedTTFont(face, INVALID_CP).is_null()) as BOOL
}
#[inline]
unsafe fn IsAdditionalTTFontCP(face: *const WCHAR, cp: UINT) -> BOOL {
    (!FindCachedTTFont(face, cp).is_null()) as BOOL
}

// ARRAYSIZE!(ptr-to-array) — element count of the pointed-to fixed array.
macro_rules! ARRAYSIZE {
    ($p:expr) => {
        (core::mem::size_of_val(&*($p)) / core::mem::size_of_val(&(*($p))[0]))
    };
}
// DBGFNT1 == DPRINT1 (debug trace) — no-op.
macro_rules! DBGFNT1 {
    ($($t:tt)*) => {
        ()
    };
}

#[inline]
unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// SINGLE_LIST_ENTRY ops (ntddk FORCEINLINE).
#[inline]
unsafe fn PushEntryList(head: *mut SINGLE_LIST_ENTRY, entry: *mut SINGLE_LIST_ENTRY) {
    (*entry).Next = (*head).Next;
    (*head).Next = entry;
}
#[inline]
unsafe fn PopEntryList(head: *mut SINGLE_LIST_ENTRY) -> *mut SINGLE_LIST_ENTRY {
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    first
}

extern "C" {
    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32;
}

// ntstrsafe.h `static __inline NTAPI` — not exports. Bounded wide copies.
#[inline]
unsafe fn StringCchCopyW(dest: *mut u16, cch_dest: usize, src: *const u16) -> HRESULT {
    if cch_dest == 0 {
        return 0x8007007Au32 as HRESULT;
    }
    let mut i = 0usize;
    while i + 1 < cch_dest && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
    0
}
#[inline]
unsafe fn StringCchCopyNW(
    dest: *mut u16,
    cch_dest: usize,
    src: *const u16,
    cch_src: usize,
) -> HRESULT {
    if cch_dest == 0 {
        return 0x8007007Au32 as HRESULT;
    }
    let mut i = 0usize;
    while i + 1 < cch_dest && i < cch_src && *src.add(i) != 0 {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    *dest.add(i) = 0;
    0
}

// concfg/font.h private types (not in winsrv-sys bindgen — NT API surface only).
pub const LF_FACESIZE: usize = 32;

#[repr(C)]
pub struct _TT_FONT_ENTRY {
    pub Entry: SINGLE_LIST_ENTRY,
    pub CodePage: UINT,
    pub DisableBold: BOOL,
    pub FaceName: [WCHAR; LF_FACESIZE],
    pub FaceNameAlt: [WCHAR; LF_FACESIZE],
}
pub type TT_FONT_ENTRY = _TT_FONT_ENTRY;
pub type PTT_FONT_ENTRY = *mut _TT_FONT_ENTRY;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct _FONT_DATA {
    pub FaceName: PWSTR,
    pub Weight: ULONG,
    pub Family: ULONG,
    pub Size: COORD,
    pub CharSet: BYTE,
}
pub type FONT_DATA = _FONT_DATA;
pub type PFONT_DATA = *mut _FONT_DATA;

// ---- file-scope preamble (translated) ----
pub const TERMINAL_FACENAME: &[u16] = &[
    'T' as u16, 'e' as u16, 'r' as u16, 'm' as u16, 'i' as u16, 'n' as u16, 'a' as u16, 'l' as u16, 0,
];

pub const DEFAULT_NON_DBCS_FONTFACE: &[u16] = &[
    'L' as u16, 'u' as u16, 'c' as u16, 'i' as u16, 'd' as u16, 'a' as u16, ' ' as u16,
    'C' as u16, 'o' as u16, 'n' as u16, 's' as u16, 'o' as u16, 'l' as u16, 'e' as u16, 0,
];

pub const DEFAULT_TT_FONT_FACENAME: &[u16] = &[
    '_' as u16, '_' as u16, 'D' as u16, 'e' as u16, 'f' as u16, 'a' as u16, 'u' as u16, 'l' as u16,
    't' as u16, 'T' as u16, 'T' as u16, 'F' as u16, 'o' as u16, 'n' as u16, 't' as u16,
    '_' as u16, '_' as u16, 0,
];

pub static mut TTFontCache: SINGLE_LIST_ENTRY = SINGLE_LIST_ENTRY {
    Next: core::ptr::null_mut(),
};

#[repr(C)]
pub struct _FIND_SUITABLE_FONT_PROC_PARAM {
    pub AltFaceName: PCWSTR,
    pub SearchFont: FONT_DATA,
    pub CodePage: UINT,
    pub StrictSearch: BOOL,
    pub FontFound: BOOL,
    pub CandidateFont: FONT_DATA,
    pub CandidateFaceName: [WCHAR; LF_FACESIZE as usize],
}
pub type FIND_SUITABLE_FONT_PROC_PARAM = _FIND_SUITABLE_FONT_PROC_PARAM;
pub type PFIND_SUITABLE_FONT_PROC_PARAM = *mut _FIND_SUITABLE_FONT_PROC_PARAM;

#[repr(C)]
pub struct _IS_VALID_CONSOLE_FONT_PARAM {
    pub IsValidFont: BOOL,
    pub CodePage: UINT,
}
pub type IS_VALID_CONSOLE_FONT_PARAM = _IS_VALID_CONSOLE_FONT_PARAM;
pub type PIS_VALID_CONSOLE_FONT_PARAM = *mut _IS_VALID_CONSOLE_FONT_PARAM;

#[no_mangle]
pub unsafe extern "C" fn CodePageToCharSet(CodePage: UINT) -> BYTE {
    let mut CharInfo: CHARSETINFO = core::mem::zeroed();
    if TranslateCharsetInfo(CodePage as usize as *mut _, &mut CharInfo, TCI_SRCCODEPAGE) != 0 {
        CharInfo.ciCharset as BYTE
    } else {
        DEFAULT_CHARSET as BYTE
    }
}

unsafe extern "system" fn FindSuitableFontProc(
    lplf: PLOGFONTW,
    lpntm: PNEWTEXTMETRICW,
    FontType: DWORD,
    lParam: LPARAM,
) -> BOOL {
    let Param = lParam as PFIND_SUITABLE_FONT_PROC_PARAM;
    let SearchFont: PFONT_DATA = core::ptr::addr_of_mut!((*Param).SearchFont);

    if IsValidConsoleFont2(lplf, lpntm, FontType, (*Param).CodePage) == 0 {
        return 1;
    }

    if FontType == TRUETYPE_FONTTYPE {
        (*lplf).lfHeight = (*SearchFont).Size.Y as i32;
        (*lplf).lfWidth = 0;
        (*lplf).lfWeight = FW_NORMAL as i32;
    }

    if (*Param).StrictSearch != 0 {
        if ((*SearchFont).Family != 0)
            && ((*SearchFont).Family as u8 != ((*lplf).lfPitchAndFamily & 0xF0))
        {
            return 1;
        }

        if ((*lplf).lfHeight != (*SearchFont).Size.Y as i32)
            && !((*lplf).lfWidth == (*SearchFont).Size.X as i32
                && (*lplf).lfHeight == (*SearchFont).Size.Y as i32)
        {
            return 1;
        }

        if ((*SearchFont).Weight != 0) && ((*SearchFont).Weight != (*lplf).lfWeight as u32) {
            return 1;
        }

        if (FontType != TRUETYPE_FONTTYPE)
            && ((*lplf).lfCharSet != (*SearchFont).CharSet)
            && !((*lplf).lfCharSet == OEM_CHARSET as u8
                && IsCJKCodePage((*Param).CodePage) != 0)
        {
            return 1;
        }

        if (*SearchFont).FaceName.is_null()
            || *((*SearchFont).FaceName) == 0
            || wcscmp((*lplf).lfFaceName.as_ptr(), (*SearchFont).FaceName) == 0
            || wcscmp((*lplf).lfFaceName.as_ptr(), (*Param).AltFaceName) == 0
        {
            let CandidateFont: PFONT_DATA = core::ptr::addr_of_mut!((*Param).CandidateFont);

            (*CandidateFont).FaceName = (*Param).CandidateFaceName.as_mut_ptr();
            StringCchCopyNW(
                (*Param).CandidateFaceName.as_mut_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*Param).CandidateFaceName)),
                (*lplf).lfFaceName.as_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*lplf).lfFaceName)),
            );

            (*CandidateFont).Weight = (*lplf).lfWeight as u32;
            (*CandidateFont).Family = ((*lplf).lfPitchAndFamily & 0xF0) as u32;

            (*CandidateFont).Size.X = (*lplf).lfWidth as i16;
            (*CandidateFont).Size.Y = (*lplf).lfHeight as i16;

            (*CandidateFont).CharSet = (*lplf).lfCharSet;

            (*Param).FontFound = 1;
            return 0;
        } else if FontType != TRUETYPE_FONTTYPE {
            let CandidateFont: PFONT_DATA = core::ptr::addr_of_mut!((*Param).CandidateFont);

            (*CandidateFont).FaceName = (*Param).CandidateFaceName.as_mut_ptr();
            StringCchCopyNW(
                (*Param).CandidateFaceName.as_mut_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*Param).CandidateFaceName)),
                (*lplf).lfFaceName.as_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*lplf).lfFaceName)),
            );

            (*CandidateFont).Weight = (*lplf).lfWeight as u32;
            (*CandidateFont).Family = ((*lplf).lfPitchAndFamily & 0xF0) as u32;

            (*CandidateFont).Size.X = (*lplf).lfWidth as i16;
            (*CandidateFont).Size.Y = (*lplf).lfHeight as i16;

            (*CandidateFont).CharSet = (*lplf).lfCharSet;

            (*Param).FontFound = 1;
        }
    } else {
        if IsCJKCodePage((*Param).CodePage) != 0 {
            if ((*SearchFont).Family != 0)
                && ((*SearchFont).Family as u8 != ((*lplf).lfPitchAndFamily & 0xF0))
            {
                return 1;
            }

            if (FontType != TRUETYPE_FONTTYPE)
                && ((*lplf).lfCharSet != (*SearchFont).CharSet)
            {
                return 1;
            }
        } else {
            if (*SearchFont).Family as u8 != ((*lplf).lfPitchAndFamily & 0xF0) {
                return 1;
            }
        }

        if ((*lplf).lfHeight >= (*SearchFont).Size.Y as i32)
            && ((*lplf).lfWidth >= (*SearchFont).Size.X as i32)
        {
            let CandidateFont: PFONT_DATA = core::ptr::addr_of_mut!((*Param).CandidateFont);

            (*CandidateFont).FaceName = (*Param).CandidateFaceName.as_mut_ptr();
            StringCchCopyNW(
                (*Param).CandidateFaceName.as_mut_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*Param).CandidateFaceName)),
                (*lplf).lfFaceName.as_ptr(),
                ARRAYSIZE!(core::ptr::addr_of!((*lplf).lfFaceName)),
            );

            (*CandidateFont).Weight = (*lplf).lfWeight as u32;
            (*CandidateFont).Family = ((*lplf).lfPitchAndFamily & 0xF0) as u32;

            (*CandidateFont).Size.X = (*lplf).lfWidth as i16;
            (*CandidateFont).Size.Y = (*lplf).lfHeight as i16;

            (*CandidateFont).CharSet = (*lplf).lfCharSet;

            (*Param).FontFound = 1;
            return 0;
        }
    }

    1
}

unsafe fn FindSuitableFont(
    FontData: PFONT_DATA,
    CodePage: UINT,
) -> BOOL {
    let mut Param: FIND_SUITABLE_FONT_PROC_PARAM = core::mem::zeroed();
    let FaceName: PWSTR;
    let mut hDC: HDC;
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut FontEntry: PTT_FONT_ENTRY;

    /* Save the original FaceName pointer */
    FaceName = (*FontData).FaceName;

    /* Save our current search criteria */
    RtlZeroMemory(
        core::ptr::addr_of_mut!(Param) as *mut _,
        core::mem::size_of::<FIND_SUITABLE_FONT_PROC_PARAM>() as _,
    );
    Param.SearchFont = *FontData;

    Param.SearchFont.CharSet = CodePageToCharSet(CodePage);
    Param.CodePage = CodePage;

    if *FaceName == 0 {
        /* Find and use a default Raster font */

        /* Use "Terminal" as the fallback */
        StringCchCopyW(FaceName, LF_FACESIZE as usize, TERMINAL_FACENAME.as_ptr());
        (*FontData).Family &= !(TMPF_TRUETYPE as u32);
    } else if wcscmp(FaceName, DEFAULT_TT_FONT_FACENAME.as_ptr()) == 0 {
        /* Find and use a default TrueType font */
        FontEntry = FindCachedTTFont(core::ptr::null_mut(), CodePage);
        if !FontEntry.is_null() {
            StringCchCopyW(FaceName, LF_FACESIZE as usize, (*FontEntry).FaceName.as_ptr());
        } else {
            StringCchCopyW(FaceName, LF_FACESIZE as usize, DEFAULT_NON_DBCS_FONTFACE.as_ptr());
        }
        (*FontData).Family |= TMPF_TRUETYPE as u32;
    }

    /* Search for a TrueType alternative face name */
    FontEntry = FindCachedTTFont(FaceName, CodePage);
    if !FontEntry.is_null() {
        /* NOTE: The font face names are case-sensitive */
        if wcscmp((*FontEntry).FaceName.as_ptr(), FaceName) == 0 {
            Param.AltFaceName = (*FontEntry).FaceNameAlt.as_ptr();
        } else if wcscmp((*FontEntry).FaceNameAlt.as_ptr(), FaceName) == 0 {
            Param.AltFaceName = (*FontEntry).FaceName.as_ptr();
        }
    } else {
        Param.AltFaceName = FaceName;
    }

    /* Initialize the search: start with a strict search, then a relaxed one */
    Param.FontFound = 0;

    Param.StrictSearch = 1;

    loop {
        /*
         * Enumerate all fonts with the given character set.
         * We will match them with the search criteria.
         */
        RtlZeroMemory(
            core::ptr::addr_of_mut!(lf) as *mut _,
            core::mem::size_of::<LOGFONTW>() as _,
        );
        lf.lfCharSet = Param.SearchFont.CharSet;

        hDC = GetDC(core::ptr::null_mut());
        EnumFontFamiliesExW(
            hDC,
            &mut lf,
            core::mem::transmute::<_, FONTENUMPROCW>(FindSuitableFontProc as *const ()),
            core::ptr::addr_of_mut!(Param) as LPARAM,
            0,
        );
        ReleaseDC(core::ptr::null_mut(), hDC);

        /* If we failed to find any font, search again with relaxed criteria */
        if Param.StrictSearch != 0 && Param.FontFound == 0 {
            Param.StrictSearch = 0;
            continue;
        }
        break;
    }

    /* If no font was found again, return failure */
    if Param.FontFound == 0 {
        return 0;
    }

    /* Return the font details */
    *FontData = Param.CandidateFont;
    (*FontData).FaceName = FaceName; // Restore the original FaceName pointer.
    StringCchCopyNW(
        FaceName,
        LF_FACESIZE as usize,
        Param.CandidateFaceName.as_ptr(),
        Param.CandidateFaceName.len(),
    );

    1
}

unsafe fn CreateConsoleFontWorker(
    FontData: PFONT_DATA,
    CodePage: UINT,
) -> HFONT {
    let mut lf: LOGFONTW = core::mem::zeroed();

    RtlZeroMemory(core::ptr::addr_of_mut!(lf) as *mut _, core::mem::size_of::<LOGFONTW>() as _);

    lf.lfHeight = (*FontData).Size.Y as ULONG as LONG;
    lf.lfWidth = (*FontData).Size.X as ULONG as LONG;

    lf.lfEscapement = 0;
    lf.lfOrientation = 0;
    lf.lfWeight = (*FontData).Weight as LONG;
    lf.lfCharSet = CodePageToCharSet(CodePage) as _;
    lf.lfOutPrecision = OUT_DEFAULT_PRECIS as _;
    lf.lfClipPrecision = CLIP_DEFAULT_PRECIS as _;
    lf.lfQuality = DEFAULT_QUALITY as _;

    lf.lfPitchAndFamily = (((FIXED_PITCH | FF_MODERN) as u32 | (*FontData).Family as u32)
        & !((VARIABLE_PITCH | FF_DECORATIVE | FF_ROMAN | FF_SCRIPT | FF_SWISS) as u32))
        as BYTE;

    if IsValidConsoleFont((*FontData).FaceName, CodePage) == 0 {
        return core::ptr::null_mut();
    }

    StringCchCopyNW(
        lf.lfFaceName.as_mut_ptr(),
        lf.lfFaceName.len(),
        (*FontData).FaceName,
        LF_FACESIZE as _,
    );

    CreateFontIndirectW(&mut lf)
}

#[no_mangle]
pub unsafe extern "C" fn CreateConsoleFontEx(
    Height: LONG,
    Width: LONG,
    FaceName: PWSTR,
    FontWeight: ULONG,
    FontFamily: ULONG,
    CodePage: UINT,
    UseDefaultFallback: BOOL,
    FontData: PFONT_DATA,
) -> HFONT {
    let mut hFont: HFONT;

    (*FontData).FaceName = FaceName;
    (*FontData).Weight = FontWeight;
    (*FontData).Family = FontFamily;
    /* NOTE: FontSize is always in cell height/width units (pixels) */
    (*FontData).Size.X = Width as SHORT;
    (*FontData).Size.Y = Height as SHORT;
    (*FontData).CharSet = 0; // CodePageToCharSet(CodePage);

    if *FaceName == 0
        || wcscmp(FaceName, DEFAULT_TT_FONT_FACENAME.as_ptr()) == 0
    {
        /* We do not have an actual font face name yet and should find one.
         * Call FindSuitableFont() to determine the default font to use. */
    } else {
        hFont = CreateConsoleFontWorker(FontData, CodePage);
        if !hFont.is_null() {
            return hFont;
        }

        DBGFNT1!(
            b"CreateConsoleFont('%S') failed - Try to find a suitable font...\n\0".as_ptr()
                as *const i8,
            FaceName,
        );
    }

    /*
     * We could not create a font with the default settings.
     * Try to find a suitable font and retry.
     */
    if FindSuitableFont(FontData, CodePage) == 0 {
        /* We could not find any suitable font, fall back
         * to some default one if required to do so. */
        DBGFNT1!(
            b"FindSuitableFont could not find anything - %s\n\0".as_ptr() as *const i8,
            if UseDefaultFallback != 0 {
                b"Falling back to 'Terminal'\0".as_ptr() as *const i8
            } else {
                b"Bailing out\0".as_ptr() as *const i8
            },
        );

        /* No fallback: no font! */
        if UseDefaultFallback == 0 {
            return core::ptr::null_mut();
        }

        //
        // FIXME: See also !*FaceName case in FindSuitableFont().
        //
        /* Use "Terminal" as the fallback */
        StringCchCopyW(FaceName, LF_FACESIZE as _, TERMINAL_FACENAME.as_ptr());
        (*FontData).Family &= !(TMPF_TRUETYPE as ULONG);
    } else {
        DBGFNT1!(
            b"FindSuitableFont found: '%S', size (%d x %d)\n\0".as_ptr() as *const i8,
            FaceName,
            (*FontData).Size.X as i32,
            (*FontData).Size.Y as i32,
        );
    }

    /* Retry creating the font */
    hFont = CreateConsoleFontWorker(FontData, CodePage);
    if hFont.is_null() {
        DBGFNT1!(
            b"CreateConsoleFont('%S') failed\n\0".as_ptr() as *const i8,
            FaceName,
        );
    }

    hFont
}

#[no_mangle]
pub unsafe extern "C" fn CreateConsoleFont2(
    Height: LONG,
    Width: LONG,
    ConsoleInfo: PCONSOLE_STATE_INFO,
) -> HFONT {
    let mut FontData: FONT_DATA = core::mem::zeroed();
    let hFont: HFONT;

    hFont = CreateConsoleFontEx(
        Height,
        Width,
        (*ConsoleInfo).FaceName.as_mut_ptr(),
        (*ConsoleInfo).FontWeight,
        (*ConsoleInfo).FontFamily,
        (*ConsoleInfo).CodePage,
        1,
        &mut FontData,
    );
    if !hFont.is_null() {
        (*ConsoleInfo).FontWeight = FontData.Weight as ULONG;
        (*ConsoleInfo).FontFamily = FontData.Family as ULONG;
    }

    hFont
}

#[no_mangle]
pub unsafe extern "C" fn CreateConsoleFont(
    ConsoleInfo: PCONSOLE_STATE_INFO,
) -> HFONT {
    CreateConsoleFont2(
        (*ConsoleInfo).FontSize.Y as ULONG as LONG,
        (*ConsoleInfo).FontSize.X as ULONG as LONG,
        ConsoleInfo,
    )
}

#[no_mangle]
pub unsafe extern "C" fn GetFontCellSize(
    mut hDC: HDC,
    hFont: HFONT,
    Height: PUINT,
    Width: PUINT,
) -> BOOL {
    let mut Success: BOOL = 0;
    let hOrgDC: HDC = hDC;
    let hOldFont: HFONT;
    let CharWidth: LONG;
    let CharHeight: LONG;
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    if hDC.is_null() {
        hDC = GetDC(core::ptr::null_mut());
    }

    hOldFont = SelectObject(hDC, hFont as _) as HFONT;
    'quit: {
        if hOldFont.is_null() {
            DBGFNT1!(b"GetFontCellSize: SelectObject failed\n\0".as_ptr());
            break 'quit;
        }

        'cleanup: {
            if GetTextMetricsW(hDC, &mut tm) == 0 {
                DBGFNT1!(b"GetFontCellSize: GetTextMetrics failed\n\0".as_ptr());
                break 'cleanup;
            }

            CharHeight = tm.tmHeight + tm.tmExternalLeading;
            CharWidth = tm.tmAveCharWidth;

            *Height = CharHeight as UINT;
            *Width = CharWidth as UINT;
            Success = 1;
        }

        SelectObject(hDC, hOldFont as _);
    }

    if hOrgDC.is_null() {
        ReleaseDC(core::ptr::null_mut(), hDC);
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn IsValidConsoleFont2(
    lplf: PLOGFONTW,
    lpntm: PNEWTEXTMETRICW,
    FontType: DWORD,
    CodePage: UINT,
) -> BOOL {
    let _ = lpntm;
    let FaceName: LPCWSTR = (*lplf).lfFaceName.as_ptr();

    if (((*lplf).lfPitchAndFamily & 0x03) != FIXED_PITCH as u8)
        && IsAdditionalTTFont(FaceName) == 0
    {
        return 0;
    }

    if (FontType == TRUETYPE_FONTTYPE) && (((*lplf).lfPitchAndFamily & 0xF0) != FF_MODERN as u8) {
        return 0;
    }

    if *FaceName == ('@' as u16) {
        return 0;
    }

    if IsCJKCodePage(CodePage) != 0 {
        if FontType == TRUETYPE_FONTTYPE {
            if IsCJKCharSet((*lplf).lfCharSet) == 0 {
                return 0;
            }

            if IsAdditionalTTFont(FaceName) != 0
                && IsAdditionalTTFontCP(FaceName, 0) == 0
                && IsCJKCharSet((*lplf).lfCharSet) == 0
            {
                return 0;
            }
        } else {
            if IsCJKCharSet((*lplf).lfCharSet) == 0 && (*lplf).lfCharSet != OEM_CHARSET as u8 {
                return 0;
            }

            if wcscmp(FaceName, TERMINAL_FACENAME.as_ptr()) != 0 {
                return 0;
            }
        }
    } else {
        if (FontType != TRUETYPE_FONTTYPE)
            && ((*lplf).lfCharSet != ANSI_CHARSET as u8)
            && ((*lplf).lfCharSet != DEFAULT_CHARSET as u8)
            && ((*lplf).lfCharSet != OEM_CHARSET as u8)
        {
            return 0;
        }
    }

    1
}

unsafe extern "system" fn IsValidConsoleFontProc(
    lplf: PLOGFONTW,
    lpntm: PNEWTEXTMETRICW,
    FontType: DWORD,
    lParam: LPARAM,
) -> BOOL {
    let Param = lParam as PIS_VALID_CONSOLE_FONT_PARAM;
    (*Param).IsValidFont = IsValidConsoleFont2(lplf, lpntm, FontType, (*Param).CodePage);

    0
}

#[no_mangle]
pub unsafe extern "C" fn IsValidConsoleFont(FaceName: PCWSTR, CodePage: UINT) -> BOOL {
    let mut Param: IS_VALID_CONSOLE_FONT_PARAM = core::mem::zeroed();
    let hDC: HDC;
    let mut lf: LOGFONTW = core::mem::zeroed();

    Param.IsValidFont = 0;
    Param.CodePage = CodePage;

    RtlZeroMemory(core::ptr::addr_of_mut!(lf) as *mut _, core::mem::size_of::<LOGFONTW>() as _);
    lf.lfCharSet = CodePageToCharSet(CodePage) as _;
    StringCchCopyW(
        lf.lfFaceName.as_mut_ptr(),
        lf.lfFaceName.len(),
        FaceName,
    );

    hDC = GetDC(core::ptr::null_mut());
    EnumFontFamiliesExW(
        hDC,
        &mut lf,
        core::mem::transmute::<_, FONTENUMPROCW>(IsValidConsoleFontProc as *const ()),
        core::ptr::addr_of_mut!(Param) as LPARAM,
        0,
    );
    ReleaseDC(core::ptr::null_mut(), hDC);

    Param.IsValidFont
}

#[no_mangle]
pub unsafe extern "C" fn InitTTFontCache() {
    let _lResult: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut dwType: DWORD = 0;
    let mut szValueName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut cchValueName: DWORD;
    let mut szValue: [WCHAR; LF_FACESIZE as usize] = [0; LF_FACESIZE as usize];
    let mut cbValue: DWORD;
    let _CodePage: UINT;
    let _FontEntry: PTT_FONT_ENTRY;
    let mut pszNext: PWCHAR;

    if TTFontCache.Next != core::ptr::null_mut() {
        return;
    }

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("Software\\Microsoft\\Windows NT\\CurrentVersion\\Console\\TrueTypeFont").as_ptr() as *const u16,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != ERROR_SUCCESS as LONG
    {
        return;
    }

    dwIndex = 0;
    cchValueName = szValueName.len() as DWORD;
    cbValue = core::mem::size_of_val(&szValue) as DWORD;

    loop {
        let lResult = RegEnumValueW(
            hKey,
            dwIndex,
            szValueName.as_mut_ptr(),
            &mut cchValueName,
            core::ptr::null_mut(),
            &mut dwType,
            szValue.as_mut_ptr() as PBYTE,
            &mut cbValue,
        );

        if lResult == ERROR_NO_MORE_ITEMS as LONG {
            break;
        }

        'cont: {
            if lResult != ERROR_SUCCESS as LONG {
                break 'cont;
            }

            if cchValueName == 0 || szValueName[0] == 0 {
                break 'cont;
            }

            if dwType != REG_SZ && dwType != REG_MULTI_SZ {
                break 'cont;
            }

            let mut pNext: PWCHAR = core::ptr::null_mut();
            let CodePage = wcstoul(szValueName.as_ptr(), &mut pNext, 10) as UINT;
            if *pNext != 0 {
                break 'cont;
            }

            let FontEntry = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<_TT_FONT_ENTRY>() as _,
            ) as PTT_FONT_ENTRY;
            if FontEntry.is_null() {
                DBGFNT1!(b"InitTTFontCache: Failed to allocate memory, continuing...\n\0".as_ptr());
                break 'cont;
            }

            (*FontEntry).CodePage = CodePage;

            pszNext = szValue.as_mut_ptr();

            if *pszNext == BOLD_MARK as WCHAR {
                (*FontEntry).DisableBold = 1;
                pszNext = pszNext.add(1);
            } else {
                (*FontEntry).DisableBold = 0;
            }

            StringCchCopyNW(
                (*FontEntry).FaceName.as_mut_ptr(),
                (*FontEntry).FaceName.len(),
                pszNext,
                wcslen(pszNext),
            );

            if dwType == REG_MULTI_SZ {
                pszNext = pszNext.add(wcslen(pszNext) + 1);

                if *pszNext == BOLD_MARK as WCHAR {
                    (*FontEntry).DisableBold = 1;
                    pszNext = pszNext.add(1);
                }

                StringCchCopyNW(
                    (*FontEntry).FaceNameAlt.as_mut_ptr(),
                    (*FontEntry).FaceNameAlt.len(),
                    pszNext,
                    wcslen(pszNext),
                );
            }

            PushEntryList(&raw mut TTFontCache, core::ptr::addr_of_mut!((*FontEntry).Entry));
        }

        dwIndex += 1;
        cchValueName = szValueName.len() as DWORD;
        cbValue = core::mem::size_of_val(&szValue) as DWORD;
    }

    RegCloseKey(hKey);
}

#[no_mangle]
pub unsafe extern "C" fn ClearTTFontCache() {
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut FontEntry: PTT_FONT_ENTRY;

    while !TTFontCache.Next.is_null() {
        Entry = PopEntryList(core::ptr::addr_of_mut!(TTFontCache));
        FontEntry = (Entry as *mut u8)
            .sub(core::mem::offset_of!(TT_FONT_ENTRY, Entry)) as PTT_FONT_ENTRY;
        RtlFreeHeap(RtlGetProcessHeap(), 0, FontEntry as *mut core::ffi::c_void);
    }
    TTFontCache.Next = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn RefreshTTFontCache() {
    ClearTTFontCache();
    InitTTFontCache();
}

#[no_mangle]
pub unsafe extern "C" fn FindCachedTTFont(
    FaceName: PCWSTR,
    CodePage: UINT,
) -> PTT_FONT_ENTRY {
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut FontEntry: PTT_FONT_ENTRY;

    if !FaceName.is_null() {
        Entry = TTFontCache.Next;
        while !Entry.is_null() {
            FontEntry = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(TT_FONT_ENTRY, Entry) as isize))
                as PTT_FONT_ENTRY;

            if (wcscmp((*FontEntry).FaceName.as_ptr(), FaceName) == 0)
                || (wcscmp((*FontEntry).FaceNameAlt.as_ptr(), FaceName) == 0)
            {
                if (CodePage == INVALID_CP) || (CodePage == (*FontEntry).CodePage) {
                    return FontEntry;
                }
            }

            Entry = (*Entry).Next;
        }
    } else if CodePage != INVALID_CP {
        Entry = TTFontCache.Next;
        while !Entry.is_null() {
            FontEntry = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(TT_FONT_ENTRY, Entry) as isize))
                as PTT_FONT_ENTRY;

            if CodePage == (*FontEntry).CodePage {
                return FontEntry;
            }

            Entry = (*Entry).Next;
        }
    }

    core::ptr::null_mut()
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Console Server DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Console GDI Fonts Management.
//  * COPYRIGHT:   Copyright 2017-2022 Hermès Bélusca-Maïto
//  *              Copyright 2017 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "precomp.h"
// #include <winuser.h>
//
// #include "settings.h"
// #include "font.h"
// // #include "concfg.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define DBGFNT  DPRINT
// #define DBGFNT1 DPRINT1
//
//
// /* GLOBALS ********************************************************************/
//
// #define TERMINAL_FACENAME           L"Terminal"
// #define DEFAULT_NON_DBCS_FONTFACE   L"Lucida Console" // L"Consolas"
// #define DEFAULT_TT_FONT_FACENAME    L"__DefaultTTFont__"
//
// /* TrueType font list cache */
// SINGLE_LIST_ENTRY TTFontCache = { NULL };
//
// // NOTE: Used to tag code that makes sense only with a font cache.
// // #define FONT_CACHE_PRESENT
//
//
// /* FUNCTIONS ******************************************************************/
//
// /**
//  * @brief
//  * Retrieves the character set associated with a given code page.
//  *
//  * @param[in]   CodePage
//  * The code page to convert.
//  *
//  * @return
//  * The character set corresponding to the code page, or @b DEFAULT_CHARSET.
//  **/
// BYTE
// CodePageToCharSet(
//      UINT CodePage)
// {
//     CHARSETINFO CharInfo;
//     if (TranslateCharsetInfo(UlongToPtr(CodePage), &CharInfo, TCI_SRCCODEPAGE))
//         return (BYTE)CharInfo.ciCharset;
//     else
//         return DEFAULT_CHARSET;
// }
//
// /*****************************************************************************/
//
// typedef struct _FIND_SUITABLE_FONT_PROC_PARAM
// {
//     /* Search criteria */
//      PCWSTR AltFaceName;
//     FONT_DATA SearchFont;
//     UINT CodePage;
//     BOOL StrictSearch; // TRUE to do strict search; FALSE for relaxed criteria.
//
//     /* Candidate font data */
//     BOOL FontFound;    // TRUE/FALSE if we have/haven't found a suitable font.
//     FONT_DATA CandidateFont;
//     WCHAR CandidateFaceName[LF_FACESIZE];
// } FIND_SUITABLE_FONT_PROC_PARAM, *PFIND_SUITABLE_FONT_PROC_PARAM;
//
// #define TM_IS_TT_FONT(x)    (((x) & TMPF_TRUETYPE) == TMPF_TRUETYPE)
// #define SIZE_EQUAL(s1, s2)  (((s1).X == (s2).X) && ((s1).Y == (s2).Y))
//
// /**
//  * @brief   EnumFontFamiliesEx() callback helper for FindSuitableFont().
//  *
//  * @remark
//  * It implements a nearly-identical console-suitable font search
//  * algorithm based on the one from FindCreateFont()
//  * https://github.com/microsoft/terminal/blob/main/src/propsheet/fontdlg.cpp#L1113
//  * excepting that for now, it does not support an internal font cache.
//  **/
// static BOOL __stdcall
// FindSuitableFontProc(
//      PLOGFONTW lplf,
//      PNEWTEXTMETRICW lpntm,
//      DWORD  FontType,
//      LPARAM lParam)
// {
//     PFIND_SUITABLE_FONT_PROC_PARAM Param = (PFIND_SUITABLE_FONT_PROC_PARAM)lParam;
//     PFONT_DATA SearchFont = &Param->SearchFont;
//
//     if (!IsValidConsoleFont2(lplf, lpntm, FontType, Param->CodePage))
//     {
//         /* This font does not suit us; continue enumeration */
//         return TRUE;
//     }
//
//     /*
//      * Since we don't cache all the possible font sizes for TrueType fonts,
//      * we cannot check our requested size (and weight) against the enumerated
//      * one; therefore reset the enumerated values to the requested ones.
//      * On the contrary, Raster fonts get their specific font sizes (and weights)
//      * enumerated separately, so for them we can keep the enumerated values.
//      */
//     if (FontType == TRUETYPE_FONTTYPE)
//     {
//         lplf->lfHeight = SearchFont->Size.Y;
//         lplf->lfWidth  = 0; // SearchFont->Size.X;
//         lplf->lfWeight = FW_NORMAL;
//     }
//
//     if (Param->StrictSearch)
//     {
//         /*
//          * Find whether this is an exact match.
//          */
//
//         /* If looking for a particular family, skip non-matches */
//         if ((SearchFont->Family != 0) &&
//             ((BYTE)SearchFont->Family != (lplf->lfPitchAndFamily & 0xF0)))
//         {
//             /* Continue enumeration */
//             return TRUE;
//         }
//
//         /* Skip non-matching sizes */
//         if ((lplf->lfHeight != SearchFont->Size.Y) &&
//             !(lplf->lfWidth == SearchFont->Size.X &&
//               lplf->lfHeight == SearchFont->Size.Y))
//         {
//             /* Continue enumeration */
//             return TRUE;
//         }
//
//         /* Skip non-matching weights */
//         if ((SearchFont->Weight != 0) &&
//             (SearchFont->Weight != lplf->lfWeight))
//         {
//             /* Continue enumeration */
//             return TRUE;
//         }
//
//         /* NOTE: We are making the font enumeration at fixed CharSet,
//          * with the one specified in the parameter block. */
//         ASSERT(lplf->lfCharSet == SearchFont->CharSet);
//
//         if ((FontType != TRUETYPE_FONTTYPE) && // !TM_IS_TT_FONT(lpntm->tmPitchAndFamily)
//             (lplf->lfCharSet != SearchFont->CharSet) &&
//             !(lplf->lfCharSet == OEM_CHARSET && IsCJKCodePage(Param->CodePage))) // g_fEastAsianSystem
//         {
//             /* Continue enumeration */
//             return TRUE;
//         }
//
//         /*
//          * Size (and maybe family) match. If we don't care about the name or
//          * if it matches, use this font. Otherwise, if name doesn't match and
//          * it is a raster font, consider it.
//          *
//          * NOTE: The font face names are case-sensitive.
//          */
//         if (!SearchFont->FaceName || !*(SearchFont->FaceName) ||
//             (wcscmp(lplf->lfFaceName, SearchFont->FaceName) == 0) ||
//             (wcscmp(lplf->lfFaceName, Param->AltFaceName) == 0))
//         {
//             // FontIndex = i;
//
//             PFONT_DATA CandidateFont = &Param->CandidateFont;
//
//             CandidateFont->FaceName = Param->CandidateFaceName;
//             StringCchCopyNW(Param->CandidateFaceName,
//                             ARRAYSIZE!(Param->CandidateFaceName),
//                             lplf->lfFaceName, ARRAYSIZE!(lplf->lfFaceName));
//
//             CandidateFont->Weight = lplf->lfWeight;
//             CandidateFont->Family = (lplf->lfPitchAndFamily & 0xF0);
//
//             CandidateFont->Size.X = lplf->lfWidth;
//             CandidateFont->Size.Y = lplf->lfHeight;
//
//             CandidateFont->CharSet = lplf->lfCharSet;
//
//             /* The font is found, stop enumeration */
//             Param->FontFound = TRUE;
//             return FALSE;
//         }
//         else if (FontType != TRUETYPE_FONTTYPE) // !TM_IS_TT_FONT(lpntm->tmPitchAndFamily)
//         {
//             // FontIndex = i;
//
//             PFONT_DATA CandidateFont = &Param->CandidateFont;
//
//             CandidateFont->FaceName = Param->CandidateFaceName;
//             StringCchCopyNW(Param->CandidateFaceName,
//                             ARRAYSIZE!(Param->CandidateFaceName),
//                             lplf->lfFaceName, ARRAYSIZE!(lplf->lfFaceName));
//
//             CandidateFont->Weight = lplf->lfWeight;
//             CandidateFont->Family = (lplf->lfPitchAndFamily & 0xF0);
//
//             CandidateFont->Size.X = lplf->lfWidth;
//             CandidateFont->Size.Y = lplf->lfHeight;
//
//             CandidateFont->CharSet = lplf->lfCharSet;
//
//             /* A close Raster Font fit was found; only the name doesn't match.
//              * Continue enumeration to see whether we can find better. */
//             Param->FontFound = TRUE;
//         }
//     }
//     else // !Param->StrictSearch
//     {
//         /*
//          * Failed to find exact match, even after enumeration, so now
//          * try to find a font of same family and same size or bigger.
//          */
//
//         if (IsCJKCodePage(Param->CodePage)) // g_fEastAsianSystem
//         {
//             if ((SearchFont->Family != 0) &&
//                 ((BYTE)SearchFont->Family != (lplf->lfPitchAndFamily & 0xF0)))
//             {
//                 /* Continue enumeration */
//                 return TRUE;
//             }
//
//             if ((FontType != TRUETYPE_FONTTYPE) && // !TM_IS_TT_FONT(lpntm->tmPitchAndFamily)
//                 (lplf->lfCharSet != SearchFont->CharSet))
//             {
//                 /* Continue enumeration */
//                 return TRUE;
//             }
//         }
//         else
//         {
//             if (// (SearchFont->Family != 0) &&
//                 ((BYTE)SearchFont->Family != (lplf->lfPitchAndFamily & 0xF0)))
//             {
//                 /* Continue enumeration */
//                 return TRUE;
//             }
//         }
//
//         if ((lplf->lfHeight >= SearchFont->Size.Y) &&
//             (lplf->lfWidth  >= SearchFont->Size.X))
//         {
//             /* Same family, size >= desired */
//             // FontIndex = i;
//
//             PFONT_DATA CandidateFont = &Param->CandidateFont;
//
//             CandidateFont->FaceName = Param->CandidateFaceName;
//             StringCchCopyNW(Param->CandidateFaceName,
//                             ARRAYSIZE!(Param->CandidateFaceName),
//                             lplf->lfFaceName, ARRAYSIZE!(lplf->lfFaceName));
//
//             CandidateFont->Weight = lplf->lfWeight;
//             CandidateFont->Family = (lplf->lfPitchAndFamily & 0xF0);
//
//             CandidateFont->Size.X = lplf->lfWidth;
//             CandidateFont->Size.Y = lplf->lfHeight;
//
//             CandidateFont->CharSet = lplf->lfCharSet;
//
//             /* The font is found, stop enumeration */
//             Param->FontFound = TRUE;
//             return FALSE;
//         }
//     }
//
//     /* Continue enumeration */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Finds a font suitable for the given code page, based on the current font
//  * and its characteristics provided in input.
//  *
//  * @param[in,out]   FontData
//  * In input: The face name and characteristics of the font to search for,
//  * possibly getting a best match.
//  * In output: The face name and characteristics of the suitable font,
//  * in case of success.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support.
//  *
//  * @return
//  * @b TRUE in case a suitable font has been found. Its name and characteristics
//  * are returned in @b FontData. @b FALSE if no suitable font has been found.
//  **/
// static BOOL
// FindSuitableFont(
//      PFONT_DATA FontData,
//      UINT CodePage)
// {
//     FIND_SUITABLE_FONT_PROC_PARAM Param;
//      PWSTR FaceName;
//     HDC hDC;
//     LOGFONTW lf;
//     PTT_FONT_ENTRY FontEntry;
//
//     /* Save the original FaceName pointer */
//     FaceName = FontData->FaceName;
//
//     /* Save our current search criteria */
//     RtlZeroMemory(&Param, sizeof(Param));
//     Param.SearchFont = *FontData;
//
//     Param.SearchFont.CharSet = CodePageToCharSet(CodePage);
//     Param.CodePage = CodePage;
//
//     if (/* !FaceName || */ !*FaceName)
//     {
//         /* Find and use a default Raster font */
//
//         /* Use "Terminal" as the fallback */
//         StringCchCopyW(FaceName, LF_FACESIZE, TERMINAL_FACENAME);
//         FontData->Family &= ~TMPF_TRUETYPE;
//     }
//     else if (wcscmp(FaceName, DEFAULT_TT_FONT_FACENAME) == 0)
//     {
//         /* Find and use a default TrueType font */
//         FontEntry = FindCachedTTFont(NULL, CodePage);
//         if (FontEntry)
//         {
//             StringCchCopyW(FaceName, LF_FACESIZE, FontEntry->FaceName);
//         }
//         else
//         {
//             StringCchCopyW(FaceName, LF_FACESIZE, DEFAULT_NON_DBCS_FONTFACE);
//         }
//         FontData->Family |= TMPF_TRUETYPE;
//     }
//
//     /* Search for a TrueType alternative face name */
//     FontEntry = FindCachedTTFont(FaceName, CodePage);
//     if (FontEntry)
//     {
//         /* NOTE: The font face names are case-sensitive */
//         if (wcscmp(FontEntry->FaceName, FaceName) == 0)
//             Param.AltFaceName = FontEntry->FaceNameAlt;
//         else if (wcscmp(FontEntry->FaceNameAlt, FaceName) == 0)
//             Param.AltFaceName = FontEntry->FaceName;
//     }
//     else
//     {
//         Param.AltFaceName = FaceName;
//     }
//
//     /* Initialize the search: start with a strict search, then a relaxed one */
//     Param.FontFound = FALSE;
//
//     Param.StrictSearch = TRUE;
// SearchAgain:
//     /*
//      * Enumerate all fonts with the given character set.
//      * We will match them with the search criteria.
//      */
//     RtlZeroMemory(&lf, sizeof(lf));
//     lf.lfCharSet = Param.SearchFont.CharSet;
//     // lf.lfPitchAndFamily = FIXED_PITCH | FF_MODERN;
//
//     hDC = GetDC(NULL);
//     EnumFontFamiliesExW(hDC, &lf, (FONTENUMPROCW)FindSuitableFontProc, (LPARAM)&Param, 0);
//     ReleaseDC(NULL, hDC);
//
//     /* If we failed to find any font, search again with relaxed criteria */
//     if (Param.StrictSearch && !Param.FontFound)
//     {
//         Param.StrictSearch = FALSE;
//         goto SearchAgain;
//     }
//
//     /* If no font was found again, return failure */
//     if (!Param.FontFound)
//         return FALSE;
//
//     /* Return the font details */
//     *FontData = Param.CandidateFont;
//     FontData->FaceName = FaceName; // Restore the original FaceName pointer.
//     StringCchCopyNW(FaceName, LF_FACESIZE,
//                     Param.CandidateFaceName,
//                     ARRAYSIZE!(Param.CandidateFaceName));
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Validates and creates a suitable console font based on the font
//  * characteristics given in input.
//  *
//  * @param[in]   FontData
//  * The face name and characteristics of the font to create.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support.
//  *
//  * @return
//  * A GDI handle to the created font, or @b NULL in case of failure.
//  **/
// static HFONT
// CreateConsoleFontWorker(
//      PFONT_DATA FontData,
//      UINT CodePage)
// {
//     LOGFONTW lf;
//
//     RtlZeroMemory(&lf, sizeof(lf));
//
//     lf.lfHeight = (LONG)(ULONG)FontData->Size.Y;
//     lf.lfWidth  = (LONG)(ULONG)FontData->Size.X;
//
//     lf.lfEscapement  = 0;
//     lf.lfOrientation = 0; // TA_BASELINE; // TA_RTLREADING; when the console supports RTL?
//     // lf.lfItalic = lf.lfUnderline = lf.lfStrikeOut = FALSE;
//     lf.lfWeight  = FontData->Weight;
//     lf.lfCharSet = CodePageToCharSet(CodePage);
//     lf.lfOutPrecision  = OUT_DEFAULT_PRECIS;
//     lf.lfClipPrecision = CLIP_DEFAULT_PRECIS;
//     lf.lfQuality = DEFAULT_QUALITY;
//
//     /* Set the mandatory flags and remove those that we do not support */
//     lf.lfPitchAndFamily = (BYTE)( (FIXED_PITCH | FF_MODERN | FontData->Family) &
//                                  ~(VARIABLE_PITCH | FF_DECORATIVE | FF_ROMAN | FF_SCRIPT | FF_SWISS));
//
//     if (!IsValidConsoleFont(FontData->FaceName, CodePage))
//         return NULL;
//
//     StringCchCopyNW(lf.lfFaceName, ARRAYSIZE!(lf.lfFaceName),
//                     FontData->FaceName, LF_FACESIZE);
//
//     return CreateFontIndirectW(&lf);
// }
//
// /*****************************************************************************/
//
// /**
//  * @brief
//  * Validates and creates a suitable console font based on the font
//  * characteristics given in input.
//  *
//  * @param[in]   Height
//  * The font height in cell units (pixels).
//  *
//  * @param[in,opt]   Width
//  * The font width in cell units (pixels).
//  *
//  * @param[in,out]   FaceName
//  * A pointer to a maximally @b LF_FACESIZE-sized buffer.
//  * In input: The buffer contains the face name of the font to try to create.
//  * In output: The buffer receives the face name of the font that has been
//  * created, in case of success. It may, or may not be, identical to the face
//  * name provided in input, in case a substitute font has been chosen.
//  *
//  * @param[in]   FontWeight
//  * The font weight.
//  *
//  * @param[in]   FontFamily
//  * The font family.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support.
//  *
//  * @param[in]   UseDefaultFallback
//  * Whether (@b TRUE) or not (@b FALSE) to use a default fallback font in case
//  * neither the specified font nor any substitute font could be found and
//  * created for the specified code page.
//  *
//  * @param[out]  FontData
//  * The face name and characteristics of the created font.
//  *
//  * @return
//  * A GDI handle to the created font, or @b NULL in case of failure.
//  *
//  * @remark
//  * Similar to FindCreateFont()
//  * https://github.com/microsoft/terminal/blob/main/src/propsheet/fontdlg.cpp#L1113
//  * but:
//  * - does not support an internal font cache for now;
//  * - returns a font handle (and not a font index to the cache).
//  **/
// HFONT
// CreateConsoleFontEx(
//          LONG Height,
//      LONG Width,
//     
//          PWSTR FaceName,
//      ULONG FontWeight,
//      ULONG FontFamily,
//      UINT  CodePage,
//      BOOL  UseDefaultFallback,
//      PFONT_DATA FontData)
// {
//     HFONT hFont;
//
//     FontData->FaceName = FaceName;
//     FontData->Weight = FontWeight;
//     FontData->Family = FontFamily;
//     /* NOTE: FontSize is always in cell height/width units (pixels) */
//     FontData->Size.X = Width;
//     FontData->Size.Y = Height;
//     FontData->CharSet = 0; // CodePageToCharSet(CodePage);
//
//     if (/* !FaceName || */ !*FaceName || wcscmp(FaceName, DEFAULT_TT_FONT_FACENAME) == 0)
//     {
//         /* We do not have an actual font face name yet and should find one.
//          * Call FindSuitableFont() to determine the default font to use. */
//     }
//     else
//     {
//         hFont = CreateConsoleFontWorker(FontData, CodePage);
//         if (hFont)
//             return hFont;
//
//         DBGFNT1!("CreateConsoleFont('%S') failed - Try to find a suitable font...\n",
//                 FaceName);
//     }
//
//     /*
//      * We could not create a font with the default settings.
//      * Try to find a suitable font and retry.
//      */
//     if (!FindSuitableFont(FontData, CodePage))
//     {
//         /* We could not find any suitable font, fall back
//          * to some default one if required to do so. */
//         DBGFNT1!("FindSuitableFont could not find anything - %s\n",
//                 UseDefaultFallback ? "Falling back to 'Terminal'"
//                                    : "Bailing out");
//
//         /* No fallback: no font! */
//         if (!UseDefaultFallback)
//             return NULL;
//
//         //
//         // FIXME: See also !*FaceName case in FindSuitableFont().
//         //
//         /* Use "Terminal" as the fallback */
//         StringCchCopyW(FaceName, LF_FACESIZE, TERMINAL_FACENAME);
//         FontData->Family &= ~TMPF_TRUETYPE;
//     }
//     else
//     {
//         DBGFNT1!("FindSuitableFont found: '%S', size (%d x %d)\n",
//                 FaceName, FontData->Size.X, FontData->Size.Y);
//     }
//
//     /* Retry creating the font */
//     hFont = CreateConsoleFontWorker(FontData, CodePage);
//     if (!hFont)
//         DBGFNT1!("CreateConsoleFont('%S') failed\n", FaceName);
//
//     return hFont;
// }
//
// /**
//  * @brief
//  * A wrapper for CreateConsoleFontEx().
//  *
//  * @param[in]   Height
//  * The font height in cell units (pixels).
//  *
//  * @param[in,opt]   Width
//  * The font width in cell units (pixels).
//  *
//  * @param[in,out]   ConsoleInfo
//  * A pointer to console settings information, containing in particular
//  * (in input) the face name and characteristics of the font to create
//  * with the current console code page.
//  * In output, the font information gets updated.
//  * Note that a default fallback font is always being used in case neither
//  * the specified font nor any substitute font could be found and created
//  * for the specified code page.
//  *
//  * @return
//  * A GDI handle to the created font, or @b NULL in case of failure.
//  *
//  * @see CreateConsoleFontEx(), CreateConsoleFont()
//  **/
// HFONT
// CreateConsoleFont2(
//          LONG Height,
//      LONG Width,
//       PCONSOLE_STATE_INFO ConsoleInfo)
// {
//     FONT_DATA FontData;
//     HFONT hFont;
//
//     hFont = CreateConsoleFontEx(Height,
//                                 Width,
//                                 ConsoleInfo->FaceName,
//                                 ConsoleInfo->FontWeight,
//                                 ConsoleInfo->FontFamily,
//                                 ConsoleInfo->CodePage,
//                                 TRUE, // UseDefaultFallback
//                                 &FontData);
//     if (hFont)
//     {
//         ConsoleInfo->FontWeight = FontData.Weight;
//         ConsoleInfo->FontFamily = FontData.Family;
//     }
//
//     return hFont;
// }
//
// /**
//  * @brief
//  * A wrapper for CreateConsoleFontEx().
//  *
//  * @param[in,out]   ConsoleInfo
//  * A pointer to console settings information, containing in particular
//  * (in input) the face name and characteristics of the font to create
//  * with the current console code page.
//  * In output, the font information gets updated.
//  * Note that a default fallback font is always being used in case neither
//  * the specified font nor any substitute font could be found and created
//  * for the specified code page.
//  *
//  * @return
//  * A GDI handle to the created font, or @b NULL in case of failure.
//  *
//  * @see CreateConsoleFontEx(), CreateConsoleFont2()
//  **/
// HFONT
// CreateConsoleFont(
//      PCONSOLE_STATE_INFO ConsoleInfo)
// {
//     /*
//      * Format:
//      * Width  = FontSize.X = LOWORD(FontSize);
//      * Height = FontSize.Y = HIWORD(FontSize);
//      */
//     /* NOTE: FontSize is always in cell height/width units (pixels) */
//     return CreateConsoleFont2((LONG)(ULONG)ConsoleInfo->FontSize.Y,
//                               (LONG)(ULONG)ConsoleInfo->FontSize.X,
//                               ConsoleInfo);
// }
//
// /**
//  * @brief
//  * Retrieves the cell size for a console font.
//  *
//  * @param[in,opt]   hDC
//  * An optional GDI device context handle.
//  *
//  * @param[in]   hFont
//  * The GDI handle to the font.
//  *
//  * @param[out]  Height
//  * In case of success, receives the cell height size (in pixels).
//  *
//  * @param[out]  Width
//  * In case of success, receives the cell height size (in pixels).
//  *
//  * @return
//  * @b TRUE if success, @b FALSE in case of failure.
//  **/
//
// BOOL
// GetFontCellSize(
//      HDC hDC,
//       HFONT hFont,
//      PUINT Height,
//      PUINT Width)
// {
//     BOOL Success = FALSE;
//     HDC hOrgDC = hDC;
//     HFONT hOldFont;
//     // LONG LogSize, PointSize;
//     LONG CharWidth, CharHeight;
//     TEXTMETRICW tm;
//     // SIZE CharSize;
//
//     if (!hDC)
//         hDC = GetDC(NULL);
//
//     hOldFont = SelectObject(hDC, hFont);
//     if (hOldFont == NULL)
//     {
//         DBGFNT1!("GetFontCellSize: SelectObject failed\n");
//         goto Quit;
//     }
//
// /*
//  * See also: Display_SetTypeFace in applications/fontview/display.c
//  */
//
//     /*
//      * Note that the method with GetObjectW just returns
//      * the original parameters with which the font was created.
//      */
//     if (!GetTextMetricsW(hDC, &tm))
//     {
//         DBGFNT1!("GetFontCellSize: GetTextMetrics failed\n");
//         goto Cleanup;
//     }
//
//     CharHeight = tm.tmHeight + tm.tmExternalLeading;
//
//     CharWidth = tm.tmAveCharWidth; // tm.tmMaxCharWidth;
//
//
//     *Height = (UINT)CharHeight;
//     *Width  = (UINT)CharWidth;
//     Success = TRUE;
//
// Cleanup:
//     SelectObject(hDC, hOldFont);
// Quit:
//     if (!hOrgDC)
//         ReleaseDC(NULL, hDC);
//
//     return Success;
// }
//
// /**
//  * @brief
//  * Validates whether a given font can be supported in the console,
//  * under the specified code page.
//  *
//  * @param[in]   lplf
//  * @param[in]   lpntm
//  * @param[in]   FontType
//  * The GDI font characteristics of the font to validate.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support.
//  *
//  * @return
//  * @b TRUE if the font is valid and supported in the console,
//  * @b FALSE if not.
//  *
//  * @remark
//  * Equivalent of the font validation tests in FontEnumForV2Console()
//  * (or the more restrictive ones in FontEnum())
//  * https://github.com/microsoft/terminal/blob/main/src/propsheet/misc.cpp#L465
//  * https://github.com/microsoft/terminal/blob/main/src/propsheet/misc.cpp#L607
//  *
//  * @see IsValidConsoleFont()
//  **/
// BOOL
// IsValidConsoleFont2(
//      PLOGFONTW lplf,
//      PNEWTEXTMETRICW lpntm,
//      DWORD FontType,
//      UINT  CodePage)
// {
//     LPCWSTR FaceName = lplf->lfFaceName;
//
//     /*
//      * According to: https://web.archive.org/web/20140901124501/http://support.microsoft.com/kb/247815
//      * "Necessary criteria for fonts to be available in a command window",
//      * the criteria for console-eligible fonts are as follows:
//      * - The font must be a fixed-pitch font.
//      * - The font cannot be an italic font.
//      * - The font cannot have a negative A or C space.
//      * - If it is a TrueType font, it must be FF_MODERN.
//      * - If it is not a TrueType font, it must be OEM_CHARSET.
//      *
//      * Non documented: vertical fonts are forbidden (their name start with a '@').
//      *
//      * Additional criteria for Asian installations:
//      * - If it is not a TrueType font, the face name must be "Terminal".
//      * - If it is an Asian TrueType font, it must also be an Asian character set.
//      *
//      * See also Raymond Chen's blog: https://devblogs.microsoft.com/oldnewthing/?p=26843
//      * and MIT-licensed Microsoft Terminal source code: https://github.com/microsoft/terminal/blob/main/src/propsheet/misc.cpp
//      * for other details.
//      *
//      * To install additional TrueType fonts to be available for the console,
//      * add entries of type REG_SZ named "0", "00" etc... in:
//      * HKEY_LOCAL_MACHINE\Software\Microsoft\Windows NT\CurrentVersion\Console\TrueTypeFont
//      * The names of the fonts listed there should match those in:
//      * HKEY_LOCAL_MACHINE\Software\Microsoft\Windows NT\CurrentVersion\Fonts
//      */
//
//     /*
//      * In ReactOS we relax some of the criteria:
//      * - We allow fixed-pitch FF_MODERN (Monospace) TrueType fonts
//      *   that can be italic or have negative A or C space.
//      * - If it is not a TrueType font, it can be from another character set
//      *   than OEM_CHARSET. When an Asian codepage is active however, we require
//      *   that this non-TrueType font has an Asian character set.
//      */
//
//     /* Reject variable-width fonts ... */
//     if ( ( ((lplf->lfPitchAndFamily & 0x03) != FIXED_PITCH)
//          ) &&
//         /* ... if they are not in the list of additional TrueType fonts to include */
//          !IsAdditionalTTFont(FaceName) )
//     {
//         DBGFNT("Font '%S' rejected because it%s (lfPitchAndFamily = %d)\n",
//                FaceName,
//                !(lplf->lfPitchAndFamily & FIXED_PITCH) ? "'s not FIXED_PITCH"
//                    : (!(lpntm->ntmFlags & NTM_NONNEGATIVE_AC) ? " has negative A or C space"
//                                                               : " is broken"),
//                lplf->lfPitchAndFamily);
//         return FALSE;
//     }
//
//     /* Reject TrueType fonts that are not FF_MODERN */
//     if ((FontType == TRUETYPE_FONTTYPE) && ((lplf->lfPitchAndFamily & 0xF0) != FF_MODERN))
//     {
//         DBGFNT("TrueType font '%S' rejected because it's not FF_MODERN (lfPitchAndFamily = %d)\n",
//                FaceName, lplf->lfPitchAndFamily);
//         return FALSE;
//     }
//
//     /* Reject vertical fonts (tategaki) */
//     if (FaceName[0] == L'@')
//     {
//         DBGFNT("Font '%S' rejected because it's vertical\n", FaceName);
//         return FALSE;
//     }
//
//     /* Is the current code page Chinese, Japanese or Korean? */
//     if (IsCJKCodePage(CodePage))
//     {
//         /* It's CJK */
//
//         if (FontType == TRUETYPE_FONTTYPE)
//         {
//             /*
//              * Here we are inclusive and check for any CJK character set,
//              * instead of looking just at the current one via CodePageToCharSet().
//              */
//             if (!IsCJKCharSet(lplf->lfCharSet))
//             {
//                 DBGFNT("TrueType font '%S' rejected because it's not Asian charset (lfCharSet = %d)\n",
//                        FaceName, lplf->lfCharSet);
//                 return FALSE;
//             }
//
//             /*
//              * If this is a cached TrueType font that is used only for certain
//              * code pages, verify that the charset it claims is the correct one.
//              *
//              * Since there may be multiple entries for a cached TrueType font,
//              * a general one (code page == 0) and one or more for explicit
//              * code pages, we need to perform two search queries instead of
//              * just one and retrieving the code page for this entry.
//              */
//             if (IsAdditionalTTFont(FaceName) && !IsAdditionalTTFontCP(FaceName, 0) &&
//                 !IsCJKCharSet(lplf->lfCharSet))
//             {
//                 DBGFNT("Cached TrueType font '%S' rejected because it claims a code page that is not Asian charset (lfCharSet = %d)\n",
//                        FaceName, lplf->lfCharSet);
//                 return FALSE;
//             }
//         }
//         else
//         {
//             /* Reject non-TrueType fonts that do not have an Asian character set */
//             if (!IsCJKCharSet(lplf->lfCharSet) && (lplf->lfCharSet != OEM_CHARSET))
//             {
//                 DBGFNT("Non-TrueType font '%S' rejected because it's not Asian charset or OEM_CHARSET (lfCharSet = %d)\n",
//                        FaceName, lplf->lfCharSet);
//                 return FALSE;
//             }
//
//             /* Reject non-TrueType fonts that are not Terminal */
//             if (wcscmp(FaceName, TERMINAL_FACENAME.as_ptr()) != 0)
//             {
//                 DBGFNT("Non-TrueType font '%S' rejected because it's not 'Terminal'\n", FaceName);
//                 return FALSE;
//             }
//         }
//     }
//     else
//     {
//         /* Not CJK */
//
//         /* Reject non-TrueType fonts that are not OEM or similar */
//         if ((FontType != TRUETYPE_FONTTYPE) &&
//             (lplf->lfCharSet != ANSI_CHARSET) &&
//             (lplf->lfCharSet != DEFAULT_CHARSET) &&
//             (lplf->lfCharSet != OEM_CHARSET))
//         {
//             DBGFNT("Non-TrueType font '%S' rejected because it's not ANSI_CHARSET or DEFAULT_CHARSET or OEM_CHARSET (lfCharSet = %d)\n",
//                    FaceName, lplf->lfCharSet);
//             return FALSE;
//         }
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// typedef struct _IS_VALID_CONSOLE_FONT_PARAM
// {
//     BOOL IsValidFont;
//     UINT CodePage;
// } IS_VALID_CONSOLE_FONT_PARAM, *PIS_VALID_CONSOLE_FONT_PARAM;
//
// /**
//  * @brief   EnumFontFamiliesEx() callback helper for IsValidConsoleFont().
//  **/
// static BOOL __stdcall
// IsValidConsoleFontProc(
//      PLOGFONTW lplf,
//      PNEWTEXTMETRICW lpntm,
//      DWORD  FontType,
//      LPARAM lParam)
// {
//     PIS_VALID_CONSOLE_FONT_PARAM Param = (PIS_VALID_CONSOLE_FONT_PARAM)lParam;
//     Param->IsValidFont = IsValidConsoleFont2(lplf, lpntm, FontType, Param->CodePage);
//
//     /* Stop the enumeration now */
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Validates whether a given font can be supported in the console,
//  * under the specified code page.
//  *
//  * @param[in]   FaceName
//  * The face name of the font to validate.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support.
//  *
//  * @return
//  * @b TRUE if the font is valid and supported in the console,
//  * @b FALSE if not.
//  *
//  * @see IsValidConsoleFont2()
//  **/
// BOOL
// IsValidConsoleFont(
//     // 
//      PCWSTR FaceName,
//      UINT CodePage)
// {
//     IS_VALID_CONSOLE_FONT_PARAM Param;
//     HDC hDC;
//     LOGFONTW lf;
//
//     Param.IsValidFont = FALSE;
//     Param.CodePage = CodePage;
//
//     RtlZeroMemory(&lf, sizeof(lf));
//     lf.lfCharSet = CodePageToCharSet(CodePage);
//     // lf.lfPitchAndFamily = FIXED_PITCH | FF_MODERN;
//     StringCchCopyW(lf.lfFaceName, ARRAYSIZE!(lf.lfFaceName), FaceName);
//
//     hDC = GetDC(NULL);
//     EnumFontFamiliesExW(hDC, &lf, (FONTENUMPROCW)IsValidConsoleFontProc, (LPARAM)&Param, 0);
//     ReleaseDC(NULL, hDC);
//
//     return Param.IsValidFont;
// }
//
//
// /**
//  * @brief
//  * Initializes the console TrueType font cache.
//  *
//  * @remark
//  * To install additional TrueType fonts to be available for the console,
//  * add entries of type REG_SZ named "0", "00" etc... in:
//  * HKEY_LOCAL_MACHINE\Software\Microsoft\Windows NT\CurrentVersion\Console\TrueTypeFont
//  * The names of the fonts listed there should match those in:
//  * HKEY_LOCAL_MACHINE\Software\Microsoft\Windows NT\CurrentVersion\Fonts
//  *
//  * @return None.
//  **/
// VOID
// InitTTFontCache(VOID)
// {
//     LRESULT lResult;
//     HKEY hKey;
//     DWORD dwIndex, dwType;
//     WCHAR szValueName[MAX_PATH];
//     DWORD cchValueName;
//     WCHAR szValue[LF_FACESIZE] = L"";
//     DWORD cbValue;
//     UINT CodePage;
//     PTT_FONT_ENTRY FontEntry;
//     PWCHAR pszNext;
//
//     if (TTFontCache.Next != NULL)
//         return;
//     // TTFontCache.Next = NULL;
//
//     /* Open the Console\TrueTypeFont key */
//     // "\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Console\\TrueTypeFont"
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"Software\\Microsoft\\Windows NT\\CurrentVersion\\Console\\TrueTypeFont",
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hKey) != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     /* Enumerate all the available TrueType console fonts */
//     for (dwIndex = 0, cchValueName = ARRAYSIZE!(szValueName),
//                       cbValue = sizeof(szValue);
//          (lResult = RegEnumValueW(hKey, dwIndex,
//                                   szValueName, &cchValueName,
//                                   NULL, &dwType,
//                                   (PBYTE)szValue, &cbValue)) != ERROR_NO_MORE_ITEMS;
//          ++dwIndex, cchValueName = ARRAYSIZE!(szValueName),
//                     cbValue = sizeof(szValue))
//     {
//         /* Ignore if we failed for another reason, e.g. because
//          * the value name is too long (and thus, invalid). */
//         if (lResult != ERROR_SUCCESS)
//             continue;
//
//         /* Validate the value name (exclude the unnamed value) */
//         if (!cchValueName || (*szValueName == UNICODE_NULL))
//             continue;
//         /* Too large value names have already been handled with ERROR_MORE_DATA */
//         ASSERT((cchValueName < ARRAYSIZE!(szValueName)) &&
//                (szValueName[cchValueName] == UNICODE_NULL));
//
//         /* Only (multi-)string values are supported */
//         if ((dwType != REG_SZ) && (dwType != REG_MULTI_SZ))
//             continue;
//
//         /* The value name is a code page (in decimal), validate it */
//         CodePage = wcstoul(szValueName, &pszNext, 10);
//         if (*pszNext)
//             continue; // Non-numerical garbage followed...
//         // IsValidCodePage(CodePage);
//
//         FontEntry = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(*FontEntry));
//         if (!FontEntry)
//         {
//             DBGFNT1!("InitTTFontCache: Failed to allocate memory, continuing...\n");
//             continue;
//         }
//
//         FontEntry->CodePage = CodePage;
//
//         pszNext = szValue;
//
//         /* Check whether bold is disabled for this font */
//         if (*pszNext == BOLD_MARK)
//         {
//             FontEntry->DisableBold = TRUE;
//             ++pszNext;
//         }
//         else
//         {
//             FontEntry->DisableBold = FALSE;
//         }
//
//         /* Copy the font name */
//         StringCchCopyNW(FontEntry->FaceName, ARRAYSIZE!(FontEntry->FaceName),
//                         pszNext, wcslen(pszNext));
//
//         if (dwType == REG_MULTI_SZ)
//         {
//             /* There may be an alternate face name as the second string */
//             pszNext += wcslen(pszNext) + 1;
//
//             /* Check whether bold is disabled for this font */
//             if (*pszNext == BOLD_MARK)
//             {
//                 FontEntry->DisableBold = TRUE;
//                 ++pszNext;
//             }
//             // else, keep the original setting.
//
//             /* Copy the alternate font name */
//             StringCchCopyNW(FontEntry->FaceNameAlt, ARRAYSIZE!(FontEntry->FaceNameAlt),
//                             pszNext, wcslen(pszNext));
//         }
//
//         PushEntryList(&TTFontCache, &FontEntry->Entry);
//     }
//
//     /* Close the key and quit */
//     RegCloseKey(hKey);
// }
//
// /**
//  * @brief
//  * Clears the console TrueType font cache.
//  *
//  * @return None.
//  **/
// VOID
// ClearTTFontCache(VOID)
// {
//     PSINGLE_LIST_ENTRY Entry;
//     PTT_FONT_ENTRY FontEntry;
//
//     while (TTFontCache.Next != NULL)
//     {
//         Entry = PopEntryList(&TTFontCache);
//         FontEntry = CONTAINING_RECORD(Entry, TT_FONT_ENTRY, Entry);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, FontEntry);
//     }
//     TTFontCache.Next = NULL;
// }
//
// /**
//  * @brief
//  * Refreshes the console TrueType font cache,
//  * by clearing and re-initializing it.
//  *
//  * @return None.
//  **/
// VOID
// RefreshTTFontCache(VOID)
// {
//     ClearTTFontCache();
//     InitTTFontCache();
// }
//
// /**
//  * @brief
//  * Searches for a font in the console TrueType font cache,
//  * with the specified code page.
//  *
//  * @param[in,opt]   FaceName
//  * An optional pointer to a maximally @b LF_FACESIZE-sized buffer.
//  * The buffer contains the face name of the font to search for.
//  *
//  * - If FaceName != NULL, search for the named font that should
//  *   match the provided code page (when CodePage != INVALID_CP).
//  *
//  * - If FaceName == NULL, search for a font with the provided
//  *   code page. In this case, CodePage cannot be == INVALID_CP,
//  *   otherwise the search fails.
//  *
//  * @param[in]   CodePage
//  * The code page the font has to support, or @b INVALID_CP when
//  * searching a font by face name only.
//  *
//  * @return
//  * A pointer to the cache entry for the font, or @b NULL if not found.
//  **/
// PTT_FONT_ENTRY
// FindCachedTTFont(
//     
//          PCWSTR FaceName,
//      UINT CodePage)
// {
//     PSINGLE_LIST_ENTRY Entry;
//     PTT_FONT_ENTRY FontEntry;
//
//     if (FaceName)
//     {
//         /* Search for the named font */
//         for (Entry = TTFontCache.Next;
//              Entry != NULL;
//              Entry = Entry->Next)
//         {
//             FontEntry = CONTAINING_RECORD(Entry, TT_FONT_ENTRY, Entry);
//
//             /* NOTE: The font face names are case-sensitive */
//             if ((wcscmp(FontEntry->FaceName   , FaceName) == 0) ||
//                 (wcscmp(FontEntry->FaceNameAlt, FaceName) == 0))
//             {
//                 /* Return the font if we don't search by code page, or when they match */
//                 if ((CodePage == INVALID_CP) || (CodePage == FontEntry->CodePage))
//                 {
//                     return FontEntry;
//                 }
//             }
//         }
//     }
//     else if (CodePage != INVALID_CP)
//     {
//         /* Search for a font with the specified code page */
//         for (Entry = TTFontCache.Next;
//              Entry != NULL;
//              Entry = Entry->Next)
//         {
//             FontEntry = CONTAINING_RECORD(Entry, TT_FONT_ENTRY, Entry);
//
//             /* Return the font if the code pages match */
//             if (CodePage == FontEntry->CodePage)
//                 return FontEntry;
//         }
//     }
//
//     return NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
