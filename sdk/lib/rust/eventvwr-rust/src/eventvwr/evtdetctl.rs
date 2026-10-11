use crate::eventvwr::eventvwr::GetEventMessage;
use crate::eventvwr::eventvwr::hwndListView;
use crate::eventvwr::eventvwr::szTitle;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2DD11AA35112: [u16; 4] = [0x0020u16, 0x0020, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_725AEBBAF954: [u16; 7] = [0x0020u16, 0x0025, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_80E09B2F645F: [u16; 3] = [0x000Du16, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_82DB442E7A60: [u16; 5] = [0x006Fu16, 0x0070, 0x0065, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_836DAD312A4A: [u16; 6] = [0x0020u16, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B128C529D488: [u16; 12] = [
    0x0043u16, 0x006F, 0x0075, 0x0072, 0x0069, 0x0065, 0x0072, 0x0020, 0x004E, 0x0065,
    0x0077, 0x0000,
];
pub static RUSTCONV_WSTR_BF73910ADD90: [u16; 7] = [0x0025u16, 0x0030, 0x0034, 0x006C, 0x0078, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_FF65042600CB: [u16; 3] = [0x0025u16, 0x0063, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// COPY_EVTTEXT_SPACE_PADDING_MODE: empty object-like macro, no value to emit.

pub const EVENT_MESSAGE_EVENTTEXT_BUFFER: usize = 1024 * 10;

#[repr(C)]
pub struct _DETAILDATA {
    /* The event record being displayed */
    pub pevlr: PEVENTLOGRECORD,

    /* Data initialized from EVENTDETAIL_INFO */
    pub EventLogFilter: PEVENTLOGFILTER,
    pub iEventItem: INT,

    pub bDisplayWords: BOOL,
    pub hMonospaceFont: HFONT,

    pub cxMin: INT,
    pub cyMin: INT,
    pub cxOld: INT,
    pub cyOld: INT,
    pub scPos: POINT,
}
pub type DETAILDATA = _DETAILDATA;
pub type PDETAILDATA = *mut DETAILDATA;

unsafe extern "C" fn DisplayEvent(hDlg: HWND, pDetailData: PDETAILDATA) {
    static lvColsToDlgItemIDs: [WORD; 8] = [
        (IDC_EVENTTYPESTATIC as u16), (IDC_EVENTDATESTATIC as u16), (IDC_EVENTTIMESTATIC as u16),
        (IDC_EVENTSOURCESTATIC as u16), (IDC_EVENTCATEGORYSTATIC as u16), (IDC_EVENTIDSTATIC as u16),
        (IDC_EVENTUSERSTATIC as u16), (IDC_EVENTCOMPUTERSTATIC as u16),
    ];

    let mut pevlr: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut EventLogFilter: PEVENTLOGFILTER = (*pDetailData).EventLogFilter;
    let mut iItem: INT = (*pDetailData).iEventItem;
    let mut i: USHORT = 0;
    let mut bEventData: BOOL = 0;

    let mut szSource: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szEventText: [WCHAR; EVENT_MESSAGE_EVENTTEXT_BUFFER] = [0; EVENT_MESSAGE_EVENTTEXT_BUFFER];

    let mut li: LVITEMW = core::mem::zeroed();
    li.mask = LVIF_PARAM as UINT;
    li.iItem = iItem;
    li.iSubItem = 0;
    ListView_GetItem!(hwndListView, core::ptr::addr_of_mut!(li));
    (*pDetailData).pevlr = li.lParam as PEVENTLOGRECORD;
    pevlr = (*pDetailData).pevlr;

    i = 0;
    while (i as usize) < (_countof!(lvColsToDlgItemIDs) as usize) {
        let mut pszBuffer: PWSTR = szEventText.as_mut_ptr();
        if lvColsToDlgItemIDs[i as usize] as ::core::ffi::c_int == (IDC_EVENTSOURCESTATIC as i32) {
            /* Use a separate buffer to store the source; used below */
            pszBuffer = szSource.as_mut_ptr();
            ListView_GetItemText!(
                hwndListView,
                iItem,
                i as INT,
                szSource.as_mut_ptr(),
                _countof!(szSource),
            );
        } else {
            ListView_GetItemText!(
                hwndListView,
                iItem,
                i as INT,
                szEventText.as_mut_ptr(),
                _countof!(szEventText),
            );
        }
        SetDlgItemTextW(
            hDlg,
            lvColsToDlgItemIDs[i as usize] as ::core::ffi::c_int,
            pszBuffer as PCWSTR,
        );
        i = i.wrapping_add(1);
    }

    bEventData = ((*pevlr).DataLength > 0) as ::core::ffi::c_int as BOOL;
    EnableDlgItem!(hDlg, (IDC_BYTESRADIO as i32), bEventData);
    EnableDlgItem!(hDlg, (IDC_WORDSRADIO as i32), bEventData);

    // FIXME: At the moment we support only one event log in the filter
    GetEventMessage(
        (*(*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize))).LogName,
        szSource.as_mut_ptr() as PCWSTR,
        pevlr,
        szEventText.as_mut_ptr(),
        (_countof!(szEventText)) as SIZE_T,
    );
    SetDlgItemTextW(
        hDlg,
        (IDC_EVENTTEXTEDIT as i32),
        szEventText.as_mut_ptr() as PCWSTR,
    );

    DisplayEventData(hDlg, pDetailData);
}

unsafe extern "C" fn PrintByteDataLine(pBuffer: PWCHAR, uOffset: UINT, pData: PBYTE, uLength: UINT) -> UINT {
    let mut p: PWCHAR = pBuffer;
    let mut n: UINT;
    let mut i: UINT;
    let mut r: UINT = 0;

    if uOffset != 0 {
        n = (_swprintf(p, RUSTCONV_WSTR_80E09B2F645F.as_ptr())) as u32;
        p = p.add(n as usize);
        r = r.wrapping_add(n);
    }

    n = (_swprintf(p, RUSTCONV_WSTR_BF73910ADD90.as_ptr(), uOffset)) as u32;
    p = p.add(n as usize);
    r = r.wrapping_add(n);

    i = 0;
    while i < uLength {
        n = (_swprintf(p, RUSTCONV_WSTR_836DAD312A4A.as_ptr(), *pData.add(i as usize) as UINT)) as u32;
        p = p.add(n as usize);
        r = r.wrapping_add(n);
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < UINT::wrapping_sub(9, uLength) {
        n = (_swprintf(p, RUSTCONV_WSTR_2DD11AA35112.as_ptr())) as u32;
        p = p.add(n as usize);
        r = r.wrapping_add(n);
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < uLength {
        // NOTE: Normally iswprint should return FALSE for tabs...
        n = (_swprintf(
            p,
            RUSTCONV_WSTR_FF65042600CB.as_ptr(),
            if iswprint((*pData.add(i as usize) as u32) as u16) != 0
                && ((*pData.add(i as usize) as u16) != 0x0009u16)
            {
                (*pData.add(i as usize) as u16)
            } else {
                0x002Eu16
            } as UINT,
        )) as u32;
        p = p.add(n as usize);
        r = r.wrapping_add(n);
        i = i.wrapping_add(1);
    }

    return r;
}

unsafe extern "C" fn PrintWordDataLine(pBuffer: PWCHAR, uOffset: UINT, pData: PULONG, uLength: UINT) -> UINT {
    let mut p: PWCHAR = pBuffer;
    let mut n: UINT;
    let mut i: UINT;
    let mut r: UINT = 0;

    if uOffset != 0 {
        n = _swprintf(p, RUSTCONV_WSTR_80E09B2F645F.as_ptr()) as UINT;
        p = p.add(n as usize);
        r += n;
    }

    n = _swprintf(p, RUSTCONV_WSTR_BF73910ADD90.as_ptr(), uOffset) as UINT;
    p = p.add(n as usize);
    r += n;

    i = 0;
    while i < uLength / (core::mem::size_of::<ULONG>() as UINT) {
        n = _swprintf(p, RUSTCONV_WSTR_725AEBBAF954.as_ptr(), *pData.add(i as usize)) as UINT;
        p = p.add(n as usize);
        r += n;
        i += 1;
    }

    /* Display the remaining bytes if uLength was not a multiple of sizeof(ULONG) */
    i = (uLength / (core::mem::size_of::<ULONG>() as UINT)) * (core::mem::size_of::<ULONG>() as UINT);
    while i < uLength {
        n = _swprintf(p, RUSTCONV_WSTR_836DAD312A4A.as_ptr(), *(pData as PBYTE).add(i as usize) as core::ffi::c_int) as UINT;
        p = p.add(n as usize);
        r += n;
        i += 1;
    }

    return r;
}

unsafe extern "C" fn DisplayEventData(mut hDlg: HWND, mut pDetailData: PDETAILDATA) {
    let mut pevlr: PEVENTLOGRECORD = (*pDetailData).pevlr;
    let mut bDisplayWords: BOOL = (*pDetailData).bDisplayWords;
    let mut pData: LPBYTE = ::core::ptr::null_mut::<BYTE>();
    let mut i: UINT = 0;
    let mut uOffset: UINT = 0;
    let mut uBufferSize: UINT = 0;
    let mut uLineLength: UINT = 0;
    let mut pTextBuffer: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut pLine: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    if pevlr.is_null() {
        return;
    }
    if (*pevlr).DataLength == 0 as DWORD {
        SetDlgItemTextW(
            hDlg,
            IDC_EVENTDATAEDIT as INT,
            RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPCWSTR,
        );
        return;
    }
    if bDisplayWords != 0 {
        uBufferSize = ((*pevlr)
            .DataLength
            .wrapping_div(8 as DWORD)
            .wrapping_add(1 as DWORD)
            .wrapping_mul(26 as DWORD) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as UINT;
    } else {
        uBufferSize = ((*pevlr)
            .DataLength
            .wrapping_div(8 as DWORD)
            .wrapping_add(1 as DWORD)
            .wrapping_mul(43 as DWORD) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as UINT;
    }
    pTextBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        uBufferSize as SIZE_T,
    ) as PWCHAR;
    if pTextBuffer.is_null() {
        return;
    }
    pLine = pTextBuffer;
    uOffset = 0 as UINT;
    i = 0 as UINT;
    while (i as DWORD) < (*pevlr).DataLength.wrapping_div(8 as DWORD) {
        pData = (pevlr as LPBYTE)
            .offset((*pevlr).DataOffset as isize)
            .offset(uOffset as isize);
        if bDisplayWords != 0 {
            uLineLength = PrintWordDataLine(
                pLine,
                uOffset,
                pData as PULONG,
                8 as UINT,
            );
        } else {
            uLineLength = PrintByteDataLine(
                pLine,
                uOffset,
                pData as PBYTE,
                8 as UINT,
            );
        }
        pLine = pLine.offset(uLineLength as isize);
        uOffset = uOffset.wrapping_add(8 as UINT);
        i = i.wrapping_add(1);
    }
    if (*pevlr).DataLength.wrapping_rem(8 as DWORD)
        != 0 as DWORD
    {
        pData = (pevlr as LPBYTE)
            .offset((*pevlr).DataOffset as isize)
            .offset(uOffset as isize);
        if bDisplayWords != 0 {
            PrintWordDataLine(
                pLine,
                uOffset,
                pData as PULONG,
                (*pevlr).DataLength.wrapping_rem(8 as DWORD) as UINT,
            );
        } else {
            PrintByteDataLine(
                pLine,
                uOffset,
                pData as PBYTE,
                (*pevlr).DataLength.wrapping_rem(8 as DWORD) as UINT,
            );
        }
    }
    SetDlgItemTextW(
        hDlg,
        IDC_EVENTDATAEDIT as INT,
        pTextBuffer as PCWSTR,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pTextBuffer as PVOID,
    );
}

unsafe extern "C" fn my_cType3ToWidth(mut wType: WORD, mut ucs: wchar_t) -> ::core::ffi::c_int {
    if wType as ::core::ffi::c_int & (C3_HALFWIDTH as i32) != 0 {
        return 1;
    } else if wType as ::core::ffi::c_int
        & ((C3_FULLWIDTH | C3_KATAKANA | C3_HIRAGANA | C3_IDEOGRAPH) as ::core::ffi::c_int)
        != 0
    {
        return 2;
    }
    // HACK for Wide Hangul characters not recognized by GetStringTypeW(CT_CTYPE3)
    // See:
    // https://unicode.org/reports/tr11/
    // https://www.unicode.org/Public/UCD/latest/ucd/EastAsianWidth.txt
    // https://www.unicode.org/Public/UCD/latest/ucd/extracted/DerivedEastAsianWidth.txt
    // (or the /Public/UNIDATA/ files)
    else if (ucs as ::core::ffi::c_int >= 0x1100 && ucs as ::core::ffi::c_int <= 0x115F)
        || (ucs as ::core::ffi::c_int >= 0x302E && ucs as ::core::ffi::c_int <= 0x302F)
        || (ucs as ::core::ffi::c_int >= 0x3131 && ucs as ::core::ffi::c_int <= 0x318E)
        || (ucs as ::core::ffi::c_int >= 0x3260 && ucs as ::core::ffi::c_int <= 0x327F)
        || (ucs as ::core::ffi::c_int >= 0xA960 && ucs as ::core::ffi::c_int <= 0xA97C)
        || (ucs as ::core::ffi::c_int >= 0xAC00 && ucs as ::core::ffi::c_int <= 0xD7A3)
    {
        return 2;
    } else if wType as ::core::ffi::c_int
        & ((C3_SYMBOL | C3_KASHIDA | C3_LEXICAL | C3_ALPHA) as ::core::ffi::c_int)
        != 0
    {
        return 1;
    } else {
        // if (wType & (C3_NONSPACING | C3_DIACRITIC | C3_VOWELMARK | C3_HIGHSURROGATE | C3_LOWSURROGATE | C3_NOTAPPLICABLE))
        return 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn my_wcwidth(mut ucs: wchar_t) -> ::core::ffi::c_int {
    let mut wType: WORD = 0 as WORD;
    GetStringTypeW(
        CT_CTYPE3 as DWORD,
        core::ptr::addr_of_mut!(ucs) as LPCWSTR,
        (core::mem::size_of::<wchar_t>() as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(wType),
    );
    return my_cType3ToWidth(wType, ucs);
}
#[no_mangle]
pub unsafe extern "C" fn my_wcswidth(mut pwcs: *const wchar_t, mut n: size_t) -> ::core::ffi::c_int {
    let mut width: ::core::ffi::c_int = 0;
    let mut pwType: PWORD;
    let mut pwt: PWORD;

    pwType = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (n as SIZE_T).wrapping_mul(::core::mem::size_of::<WORD>() as SIZE_T),
    ) as PWORD;
    if pwType.is_null() {
        return 0;
    }
    'quit: {
        if GetStringTypeW(
            CT_CTYPE3 as DWORD,
            pwcs as LPCWSTR,
            n as ::core::ffi::c_int,
            pwType as LPWORD,
        ) == 0
        {
            break 'quit;
        }

        pwt = pwType;
        loop {
            let fresh35 = n;
            n = n.wrapping_sub(1);
            if !(fresh35 > 0) {
                break;
            }
            width += my_cType3ToWidth(*pwt, *pwcs);
            pwt = pwt.offset(1);
            pwcs = pwcs.offset(1);
        }
    }
    // Quit:
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pwType as PVOID,
    );
    return width;
}

unsafe extern "C" fn CopyEventEntry(mut hWnd: HWND) {
    static nTabWidth: LONG = 4;
    static szCRLF: [WCHAR; 3] = [0x000Du16, 0x000A, 0x0000];
    // The C's `CopyData[]` element is an anonymous function-local struct (c2rust named it
    // C2RustUnnamed_11); no island carries a name for it. Field order as in the C, with the
    // `WORD bSameLine : 1; WORD bOptional : 1;` pair in one WORD of storage (bit 0, bit 1).
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct COPY_DATA {
        uHdrID: WORD,
        nDlgItemID: WORD,
        bSameLine_bOptional: WORD,
        pchHdrText: PCWCH,
        cchHdrLen: SIZE_T,
        cchInfoLen: SIZE_T,
        nHdrWidth: UINT,
        nSpacesPad: UINT,
    }
    impl COPY_DATA {
        #[inline(always)]
        fn bSameLine(&self) -> WORD {
            self.bSameLine_bOptional & 1
        }
        #[inline(always)]
        fn set_bSameLine(&mut self, v: WORD) {
            self.bSameLine_bOptional = (self.bSameLine_bOptional & !1) | (v & 1);
        }
        #[inline(always)]
        fn bOptional(&self) -> WORD {
            (self.bSameLine_bOptional >> 1) & 1
        }
        #[inline(always)]
        fn set_bOptional(&mut self, v: WORD) {
            self.bSameLine_bOptional = (self.bSameLine_bOptional & !2) | ((v & 1) << 1);
        }
    }
    let mut CopyData: [COPY_DATA; 10] = [
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTTYPE as WORD,
                nDlgItemID: IDC_EVENTTYPESTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTSRC as WORD,
                nDlgItemID: IDC_EVENTSOURCESTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTCAT as WORD,
                nDlgItemID: IDC_EVENTCATEGORYSTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTID as WORD,
                nDlgItemID: IDC_EVENTIDSTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTDATE as WORD,
                nDlgItemID: IDC_EVENTDATESTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTTIME as WORD,
                nDlgItemID: IDC_EVENTTIMESTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTUSER as WORD,
                nDlgItemID: IDC_EVENTUSERSTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTCOMP as WORD,
                nDlgItemID: IDC_EVENTCOMPUTERSTATIC as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(TRUE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTTEXT as WORD,
                nDlgItemID: IDC_EVENTTEXTEDIT as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(FALSE as WORD);
            init.set_bOptional(FALSE as WORD);
            init
        },
        {
            let mut init = COPY_DATA {
                bSameLine_bOptional: 0,
                uHdrID: IDS_COPY_EVTDATA as WORD,
                nDlgItemID: IDC_EVENTDATAEDIT as WORD,
                pchHdrText: ::core::ptr::null::<WCHAR>(),
                cchHdrLen: 0 as SIZE_T,
                cchInfoLen: 0 as SIZE_T,
                nHdrWidth: 0,
                nSpacesPad: 0,
            };
            init.set_bSameLine(FALSE as WORD);
            init.set_bOptional(TRUE as WORD);
            init
        },
    ];
    let mut i: USHORT = 0;
    let mut bUsePad: BOOL = 0;
    let mut nMaxHdrWidth: UINT = 0 as UINT;
    let mut size: SIZE_T = 0 as SIZE_T;
    let mut output: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut pszDestEnd: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut cchRemaining: size_t = 0;
    let mut hMem: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();

    /* Try to open the clipboard */
    if OpenClipboard(hWnd) == 0 {
        return;
    }

    /* Use space padding only if the user presses SHIFT */
    bUsePad = ((GetKeyState((VK_SHIFT as i32)) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0) as ::core::ffi::c_int as BOOL;

    /* Calculate the necessary string buffer size */
    i = 0 as USHORT;
    while (i as u32) < _countof!(CopyData) {
        /* Retrieve the event info string length (without NUL terminator) */
        CopyData[i as usize].cchInfoLen = GetWindowTextLengthW(GetDlgItem(
            hWnd,
            CopyData[i as usize].nDlgItemID as ::core::ffi::c_int,
        )) as SIZE_T;

        /* If no data is present and is optional, ignore it */
        if !(CopyData[i as usize].cchInfoLen == 0 as SIZE_T
            && CopyData[i as usize].bOptional() != 0)
        {
            /* Load the header string from resources */
            CopyData[i as usize].cchHdrLen = LoadStringW(
                hInst,
                CopyData[i as usize].uHdrID as UINT,
                core::ptr::addr_of_mut!(CopyData[i as usize].pchHdrText) as PWSTR,
                0 as ::core::ffi::c_int,
            ) as SIZE_T;
            size = size.wrapping_add(CopyData[i as usize].cchHdrLen);

            if CopyData[i as usize].bSameLine() != 0 {
                /* The header and info are on the same line */
                if bUsePad != 0 {
                    /* Retrieve the maximum header string displayed
                     * width for computing space padding later */
                    CopyData[i as usize].nHdrWidth = my_wcswidth(
                        CopyData[i as usize].pchHdrText as *const wchar_t,
                        CopyData[i as usize].cchHdrLen as size_t,
                    ) as UINT;
                    nMaxHdrWidth = max!(nMaxHdrWidth, CopyData[i as usize].nHdrWidth);
                } else {
                    /* Count a TAB separator */
                    size = size.wrapping_add(1);
                }
            } else {
                /* The data is on a separate line, count a newline */
                size = size.wrapping_add(
                    (_countof!(szCRLF) - 1) as SIZE_T,
                );
            }

            /* Count the event info string and the newline that follows it */
            size = size.wrapping_add(CopyData[i as usize].cchInfoLen);
            size = size.wrapping_add(
                (_countof!(szCRLF) - 1) as SIZE_T,
            );
        }
        i = i.wrapping_add(1);
    }

    if bUsePad != 0 {
        /* Round nMaxHdrWidth to the next TAB width, and
         * compute the space padding for each field */
        let mut nSpaceWidth: UINT = 1 as UINT; // my_wcwidth(L' ');
        nMaxHdrWidth = ((nMaxHdrWidth / nTabWidth as UINT)
            .wrapping_add(1 as UINT))
            .wrapping_mul(nTabWidth as UINT);

        i = 0 as USHORT;
        while (i as u32) < _countof!(CopyData) {
            /* If no data is present and is optional, ignore it */
            if !(CopyData[i as usize].cchInfoLen == 0 as SIZE_T
                && CopyData[i as usize].bOptional() != 0)
            {
                /* If the data is on a separate line, ignore padding */
                if CopyData[i as usize].bSameLine() != 0 {
                    /* Compute the padding */
                    CopyData[i as usize].nSpacesPad = nMaxHdrWidth
                        .wrapping_sub(CopyData[i as usize].nHdrWidth)
                        .wrapping_div(nSpaceWidth);
                    size = size.wrapping_add(CopyData[i as usize].nSpacesPad as SIZE_T);
                }
            }
            i = i.wrapping_add(1);
        }
    }
    /* Add NUL-termination */
    size = size.wrapping_add(1);

    /*
     * Consolidate the information into a single buffer to copy in the clipboard.
     */
    hMem = GlobalAlloc(
        (GMEM_MOVEABLE | GMEM_SHARE) as UINT,
        size.wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    );

    'quit: {
        if hMem.is_null() {
            break 'quit;
        }

        output = GlobalLock(hMem) as PWSTR;
        if output.is_null() {
            GlobalFree(hMem);
            break 'quit;
        }

        /* Build the string */
        pszDestEnd = output;
        cchRemaining = size as size_t;
        i = 0 as USHORT;
        while (i as u32) < _countof!(CopyData) {
            let mut sizeDataStr: SIZE_T = 0;

            /* If no data is present and is optional, ignore it */
            if !(CopyData[i as usize].cchInfoLen == 0 as SIZE_T
                && CopyData[i as usize].bOptional() != 0)
            {
                /* Copy the header string */
                StringCchCopyNExW(
                    pszDestEnd,
                    cchRemaining,
                    CopyData[i as usize].pchHdrText,
                    CopyData[i as usize].cchHdrLen as size_t,
                    core::ptr::addr_of_mut!(pszDestEnd),
                    core::ptr::addr_of_mut!(cchRemaining),
                    0 as STRSAFE_DWORD,
                );

                if CopyData[i as usize].bSameLine() != 0 {
                    /* The header and info are on the same line, add
                     * either the space padding or the TAB separator */
                    if bUsePad != 0 {
                        let mut j: UINT = CopyData[i as usize].nSpacesPad;
                        while j != 0 {
                            j = j.wrapping_sub(1);
                            let fresh = pszDestEnd;
                            pszDestEnd = pszDestEnd.offset(1);
                            *fresh = 0x0020u16;
                            cchRemaining = cchRemaining.wrapping_sub(1);
                        }
                    } else {
                        let fresh = pszDestEnd;
                        pszDestEnd = pszDestEnd.offset(1);
                        *fresh = 0x0009u16;
                        cchRemaining = cchRemaining.wrapping_sub(1);
                    }
                } else {
                    /* The data is on a separate line, add a newline */
                    StringCchCopyExW(
                        pszDestEnd,
                        cchRemaining,
                        core::ptr::addr_of!(szCRLF) as PCWSTR,
                        core::ptr::addr_of_mut!(pszDestEnd),
                        core::ptr::addr_of_mut!(cchRemaining),
                        0 as STRSAFE_DWORD,
                    );
                }

                /* Copy the event info */
                sizeDataStr = min!(
                    cchRemaining as SIZE_T,
                    CopyData[i as usize]
                        .cchInfoLen
                        .wrapping_add(1 as SIZE_T),
                );
                sizeDataStr = GetDlgItemTextW(
                    hWnd,
                    CopyData[i as usize].nDlgItemID as ::core::ffi::c_int,
                    pszDestEnd,
                    sizeDataStr as ::core::ffi::c_int,
                ) as SIZE_T;
                pszDestEnd = pszDestEnd.offset(sizeDataStr as isize);
                cchRemaining = cchRemaining.wrapping_sub(sizeDataStr as size_t);

                /* A newline follows the data */
                StringCchCopyExW(
                    pszDestEnd,
                    cchRemaining,
                    core::ptr::addr_of!(szCRLF) as PCWSTR,
                    core::ptr::addr_of_mut!(pszDestEnd),
                    core::ptr::addr_of_mut!(cchRemaining),
                    0 as STRSAFE_DWORD,
                );
            }
            i = i.wrapping_add(1);
        }
        /* NUL-terminate the buffer */
        let fresh = pszDestEnd;
        pszDestEnd = pszDestEnd.offset(1);
        *fresh = UNICODE_NULL as WCHAR;
        cchRemaining = cchRemaining.wrapping_sub(1);

        GlobalUnlock(hMem);

        /* We succeeded, empty the clipboard and write the data in it */
        EmptyClipboard();
        SetClipboardData(CF_UNICODETEXT as UINT, hMem as HANDLE);
    }

    /* Close the clipboard once we are done with it */
    CloseClipboard();
}

unsafe extern "C" fn OnLink(hDlg: HWND, penLink: *mut ENLINK) {
    let mut pLink: LPWSTR = core::ptr::null_mut();
    let mut txtRange: TEXTRANGE = core::mem::zeroed();

    ASSERT!((*penLink).nmhdr.idFrom == (IDC_EVENTTEXTEDIT as _));

    /* Only act on left button up events */
    if (*penLink).msg != WM_LBUTTONUP {
        return;
    }

    /* If the range is empty, do nothing */
    if (*penLink).chrg.cpMin == (*penLink).chrg.cpMax {
        return;
    }

    /* Allocate memory for the text link */
    pLink = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        ((max!((*penLink).chrg.cpMin, (*penLink).chrg.cpMax)
            - min!((*penLink).chrg.cpMin, (*penLink).chrg.cpMax)
            + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if pLink.is_null() {
        /* Not enough memory, bail out */
        return;
    }

    txtRange.chrg = (*penLink).chrg;
    txtRange.lpstrText = pLink;
    SendDlgItemMessageW(
        hDlg,
        (IDC_EVENTTEXTEDIT as i32),
        EM_GETTEXTRANGE,
        0,
        core::ptr::addr_of_mut!(txtRange) as LPARAM,
    );

    /* Open the link */
    ShellExecuteW(
        hDlg,
        RUSTCONV_WSTR_82DB442E7A60.as_ptr(),
        pLink,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (SW_SHOWNOACTIVATE) as INT,
    );

    /* Free the buffer */
    HeapFree(
        GetProcessHeap(),
        0,
        pLink as PVOID,
    );
}

unsafe extern "C" fn OnScroll(hDlg: HWND, pData: PDETAILDATA, nBar: INT, sbCode: WORD) {
    let mut rect: RECT = core::mem::zeroed();

    let mut sInfo: SCROLLINFO = core::mem::zeroed();
    let mut oldPos: INT;
    let mut Maximum: INT;
    let mut pOriginXY: PLONG = core::ptr::null_mut();

    ASSERT!(nBar == (SB_HORZ as i32) || nBar == (SB_VERT as i32));

    GetClientRect(hDlg, core::ptr::addr_of_mut!(rect));

    if nBar == (SB_HORZ as i32) {
        Maximum = (*pData).cxMin - (rect.right - rect.left);
        pOriginXY = core::ptr::addr_of_mut!((*pData).scPos.x);
    } else {
        Maximum = (*pData).cyMin - (rect.bottom - rect.top);
        pOriginXY = core::ptr::addr_of_mut!((*pData).scPos.y);
    }

    /* Set scrollbar sizes */
    sInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    sInfo.fMask = (SIF_RANGE | SIF_POS | SIF_PAGE | SIF_TRACKPOS) as UINT;

    if GetScrollInfo(hDlg, nBar, core::ptr::addr_of_mut!(sInfo)) == 0 {
        return;
    }

    oldPos = sInfo.nPos;

    match (sbCode as INT) as u32 {
        SB_LINEUP => {
            // SB_LINELEFT
            sInfo.nPos -= 1;
        }
        SB_LINEDOWN => {
            // SB_LINERIGHT
            sInfo.nPos += 1;
        }
        SB_PAGEUP => {
            // SB_PAGELEFT
            sInfo.nPos -= sInfo.nPage as INT;
        }
        SB_PAGEDOWN => {
            // SB_PAGERIGHT
            sInfo.nPos += sInfo.nPage as INT;
        }
        SB_THUMBTRACK => {
            sInfo.nPos = sInfo.nTrackPos as INT;
        }
        SB_THUMBPOSITION => {
            sInfo.nPos = sInfo.nTrackPos as INT;
        }
        SB_TOP => {
            // SB_LEFT
            sInfo.nPos = sInfo.nMin;
        }
        SB_BOTTOM => {
            // SB_RIGHT
            sInfo.nPos = sInfo.nMax;
        }
        _ => {}
    }

    sInfo.nPos = min!(max!(sInfo.nPos, 0), Maximum);

    if oldPos != sInfo.nPos {
        let scOldPos: POINT = (*pData).scPos;

        /* We now modify pData->scPos */
        *pOriginXY = sInfo.nPos;

        ScrollWindowEx(
            hDlg,
            scOldPos.x - (*pData).scPos.x,
            scOldPos.y - (*pData).scPos.y,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE | SW_ERASE | SW_SCROLLCHILDREN) as UINT,
        );

        sInfo.fMask = SIF_POS as UINT;
        SetScrollInfo(hDlg, nBar, core::ptr::addr_of_mut!(sInfo), (TRUE) as BOOL);

        // UpdateWindow(hDlg);
    }
}

unsafe extern "C" fn OnSize(hDlg: HWND, pData: PDETAILDATA, mut cx: INT, mut cy: INT) {
    let mut dwStyle: LONG_PTR = 0;
    let mut sbVXSize: INT = 0;
    let mut sbHYSize: INT = 0;
    let mut sInfo: SCROLLINFO = SCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    let mut scOldPos: POINT = POINT { x: 0, y: 0 };
    let mut hdwp: HDWP = core::ptr::null_mut();
    let mut hItemWnd: HWND = core::ptr::null_mut();
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut y: INT = 0 as INT;

    if pData.is_null() {
        return;
    }

    dwStyle = GetWindowLongPtrW(hDlg, GWL_STYLE);
    sbVXSize = GetSystemMetrics((SM_CXVSCROLL as i32)) as INT;
    sbHYSize = GetSystemMetrics((SM_CYHSCROLL as i32)) as INT;

    /* Compensate for existing scroll bars (because lParam values do not accommodate scroll bar) */
    if (dwStyle & (WS_HSCROLL as LONG_PTR)) != 0 {
        cy += sbHYSize; // Window currently has a horizontal scrollbar
    }
    if (dwStyle & (WS_VSCROLL as LONG_PTR)) != 0 {
        cx += sbVXSize; // Window currently has a vertical scrollbar
    }

    /* Compensate for added scroll bars in window */
    if cx < (*pData).cxMin {
        cy -= sbHYSize; // Window will have a horizontal scroll bar
    }
    if cy < (*pData).cyMin {
        cx -= sbVXSize; // Window will have a vertical scroll bar
    }

    /* Set scrollbar sizes */
    sInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;

    sInfo.fMask = SIF_POS as UINT;
    if GetScrollInfo(hDlg, (SB_VERT as i32), core::ptr::addr_of_mut!(sInfo)) != 0 {
        scOldPos.y = sInfo.nPos as LONG;
    } else {
        scOldPos.y = (*pData).scPos.y;
    }

    sInfo.fMask = (SIF_RANGE as UINT) | (SIF_PAGE as UINT) | (SIF_POS as UINT);
    sInfo.nMin = 0 as INT;
    if (*pData).cyMin > cy {
        sInfo.nMax = (*pData).cyMin - (1 as INT);
        sInfo.nPage = cy as UINT;
        sInfo.nPos = (*pData).scPos.y as INT;
        SetScrollInfo(hDlg, (SB_VERT as i32), core::ptr::addr_of_mut!(sInfo), (TRUE) as BOOL);

        /* Display the scrollbar if needed */
        if (dwStyle & (WS_VSCROLL as LONG_PTR)) == 0 {
            ShowScrollBar(hDlg, (SB_VERT as i32), (TRUE) as BOOL);
        }
    } else {
        scOldPos.y = 0 as LONG;

        sInfo.nMax = (*pData).cyMin - (1 as INT);
        sInfo.nPage = cy as UINT;
        sInfo.nPos = (*pData).scPos.y as INT;
        sInfo.nPos = scOldPos.y as INT;
        SetScrollInfo(hDlg, (SB_VERT as i32), core::ptr::addr_of_mut!(sInfo), (TRUE) as BOOL);

        ShowScrollBar(hDlg, (SB_VERT as i32), (FALSE) as BOOL);

        rect.left = (cx - sbVXSize) as LONG;
        rect.right = cx as LONG;
        rect.top = 0 as LONG;
        rect.bottom = cy as LONG;
        InvalidateRect(hDlg, core::ptr::addr_of_mut!(rect), (TRUE) as BOOL);
    }

    sInfo.fMask = SIF_POS as UINT;
    if GetScrollInfo(hDlg, (SB_HORZ as i32), core::ptr::addr_of_mut!(sInfo)) != 0 {
        scOldPos.x = sInfo.nPos as LONG;
    } else {
        scOldPos.x = (*pData).scPos.x;
    }

    sInfo.fMask = (SIF_RANGE as UINT) | (SIF_PAGE as UINT) | (SIF_POS as UINT);
    sInfo.nMin = 0 as INT;
    if (*pData).cxMin > cx {
        sInfo.nMax = (*pData).cxMin - (1 as INT);
        sInfo.nPage = cx as UINT;
        sInfo.nPos = (*pData).scPos.x as INT;
        SetScrollInfo(hDlg, (SB_HORZ as i32), core::ptr::addr_of_mut!(sInfo), (TRUE) as BOOL);

        /* Display the scrollbar if needed */
        if (dwStyle & (WS_HSCROLL as LONG_PTR)) == 0 {
            ShowScrollBar(hDlg, (SB_HORZ as i32), (TRUE) as BOOL);
        }
    } else {
        scOldPos.x = 0 as LONG;

        sInfo.nMax = (*pData).cxMin - (1 as INT);
        sInfo.nPage = cx as UINT;
        sInfo.nPos = (*pData).scPos.x as INT;
        sInfo.nPos = scOldPos.x as INT;
        SetScrollInfo(hDlg, (SB_HORZ as i32), core::ptr::addr_of_mut!(sInfo), (TRUE) as BOOL);

        ShowScrollBar(hDlg, (SB_HORZ as i32), (FALSE) as BOOL);

        rect.left = 0 as LONG;
        rect.right = cx as LONG;
        rect.top = (cy - sbHYSize) as LONG;
        rect.bottom = cy as LONG;
        InvalidateRect(hDlg, core::ptr::addr_of_mut!(rect), (TRUE) as BOOL);
    }

    if (scOldPos.x != (*pData).scPos.x) || (scOldPos.y != (*pData).scPos.y) {
        ScrollWindowEx(
            hDlg,
            ((*pData).scPos.x - scOldPos.x) as INT,
            ((*pData).scPos.y - scOldPos.y) as INT,
            core::ptr::null::<RECT>(),
            core::ptr::null::<RECT>(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (SW_INVALIDATE as UINT) | (SW_ERASE as UINT) | (SW_SCROLLCHILDREN as UINT),
        );

        (*pData).scPos = scOldPos;
    }

    /* Forbid resizing the control smaller than its minimal size */
    if cx < (*pData).cxMin {
        cx = (*pData).cxMin;
    }
    if cy < (*pData).cyMin {
        cy = (*pData).cyMin;
    }

    if (cx != (*pData).cxOld) || (cy != (*pData).cyOld) {
        hdwp = BeginDeferWindowPos(8) as HDWP;

        /* Move the edit boxes */

        GetWindowRect(hDlg, core::ptr::addr_of_mut!(rect));

        hItemWnd = GetDlgItem(hDlg, (IDC_EVENTTEXTEDIT as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );
        // OffsetRect(&rect, 0, y);
        // y += (cy - pData->cyOld) / 2 ; // + (cy - pData->cyOld) % 2;
        /* y += (cy - pData->cyOld) / 2 ; // + (cy - pData->cyOld) % 2; */
        if cy >= (*pData).cyOld {
            y += (cy - (*pData).cyOld) / (2 as INT)
                + (cy - (*pData).cyOld) % (2 as INT);
        } else {
            y -= ((*pData).cyOld - cy) / (2 as INT)
                + ((*pData).cyOld - cy) % (2 as INT);
        }

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                rect.left as INT,
                rect.top as INT,
                (rect.right - rect.left + (cx - (*pData).cxOld) as LONG) as INT,
                (rect.bottom - rect.top + y as LONG) as INT,
                /* SWP_NOMOVE | */ (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, IDC_DETAILS_STATIC);
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );
        // OffsetRect(&rect, 0, y);

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                rect.left as INT,
                (rect.top + y as LONG) as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, (IDC_BYTESRADIO as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );
        // OffsetRect(&rect, 0, y);

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                rect.left as INT,
                (rect.top + y as LONG) as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, (IDC_WORDSRADIO as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );
        // OffsetRect(&rect, 0, y);

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                rect.left as INT,
                (rect.top + y as LONG) as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, (IDC_EVENTDATAEDIT as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );
        // OffsetRect(&rect, 0, y);
        // // y -= (cy - pData->cyOld) % 2;

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                rect.left as INT,
                (rect.top + y as LONG) as INT,
                (rect.right - rect.left + (cx - (*pData).cxOld) as LONG) as INT,
                (rect.bottom - rect.top + y as LONG) as INT,
                (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        /* Move the buttons */

        hItemWnd = GetDlgItem(hDlg, (IDC_PREVIOUS as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                (rect.left + (cx - (*pData).cxOld) as LONG) as INT,
                rect.top as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, (IDC_NEXT as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                (rect.left + (cx - (*pData).cxOld) as LONG) as INT,
                rect.top as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        hItemWnd = GetDlgItem(hDlg, (IDC_COPY as i32));
        GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
        MapWindowPoints(
            HWND_DESKTOP /*NULL*/,
            hDlg,
            core::ptr::addr_of_mut!(rect) as LPPOINT,
            (core::mem::size_of::<RECT>() as usize).wrapping_div(core::mem::size_of::<POINT>() as usize) as UINT,
        );

        if !hdwp.is_null() {
            hdwp = DeferWindowPos(
                hdwp,
                hItemWnd,
                core::ptr::null_mut(),
                (rect.left + (cx - (*pData).cxOld) as LONG) as INT,
                rect.top as INT,
                0,
                0,
                (SWP_NOSIZE as UINT) | (SWP_NOZORDER as UINT) | (SWP_NOACTIVATE as UINT),
            );
        }

        if !hdwp.is_null() {
            EndDeferWindowPos(hdwp);
        }

        (*pData).cxOld = cx;
        (*pData).cyOld = cy;
    }
}

unsafe extern "C" fn EnableNavigationArrows(hDlg: HWND, mut bEnable: BOOL) {
    /* Enable Previous/Next only if there is more than one item in the list */
    if bEnable != 0 {
        bEnable &= (ListView_GetItemCount!(hwndListView) > 1) as BOOL;
    }
    EnableDlgItem!(hDlg, (IDC_PREVIOUS as i32), bEnable);
    EnableDlgItem!(hDlg, (IDC_NEXT as i32), bEnable);
}

unsafe extern "C" fn ClearContents(hDlg: HWND) {
    /* Disable the Previous/Next and Copy buttons */
    EnableNavigationArrows(hDlg, (FALSE) as BOOL);
    EnableDlgItem!(hDlg, (IDC_COPY as i32), (FALSE as i32));

    /* Disable the Bytes/Words mode buttons */
    EnableDlgItem!(hDlg, (IDC_BYTESRADIO as i32), (FALSE as i32));
    EnableDlgItem!(hDlg, (IDC_WORDSRADIO as i32), (FALSE as i32));

    /* Clear the data fields */
    SetDlgItemTextW(hDlg, (IDC_EVENTDATESTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTTIMESTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTUSERSTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTSOURCESTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTCOMPUTERSTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTCATEGORYSTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTIDSTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTTYPESTATIC as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

    SetDlgItemTextW(hDlg, (IDC_EVENTTEXTEDIT as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hDlg, (IDC_EVENTDATAEDIT as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
}

unsafe extern "C" fn CreateMonospaceFont() -> HFONT {
    let mut tmpFont: LOGFONTW = core::mem::zeroed();
    let mut hFont: HFONT;
    let mut hDC: HDC;

    hDC = GetDC(core::ptr::null_mut());

    tmpFont.lfHeight = -MulDiv(8, GetDeviceCaps(hDC, (LOGPIXELSY as i32)), 72);
    tmpFont.lfWeight = (FW_NORMAL as i32);
    wcscpy(core::ptr::addr_of_mut!(tmpFont.lfFaceName) as *mut u16, RUSTCONV_WSTR_B128C529D488.as_ptr());

    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(tmpFont));

    ReleaseDC(core::ptr::null_mut(), hDC);

    return hFont;
}

unsafe extern "C" fn InitDetailsDlgCtrl(hDlg: HWND, pData: PDETAILDATA) {
    let mut dwMask: DWORD = 0;

    let nextIcon: HANDLE = LoadImageW(hInst, MAKEINTRESOURCEW!(IDI_NEXT), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);
    let prevIcon: HANDLE = LoadImageW(hInst, MAKEINTRESOURCEW!(IDI_PREV), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);
    let copyIcon: HANDLE = LoadImageW(hInst, MAKEINTRESOURCEW!(IDI_COPY), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);

    SendDlgItemMessageW(hDlg, (IDC_NEXT as i32), BM_SETIMAGE, IMAGE_ICON as WPARAM, nextIcon as LPARAM);
    SendDlgItemMessageW(hDlg, (IDC_PREVIOUS as i32), BM_SETIMAGE, IMAGE_ICON as WPARAM, prevIcon as LPARAM);
    SendDlgItemMessageW(hDlg, (IDC_COPY as i32), BM_SETIMAGE, IMAGE_ICON as WPARAM, copyIcon as LPARAM);

    /* Set the default read-only RichEdit color */
    SendDlgItemMessageW(hDlg, (IDC_EVENTTEXTEDIT as i32), EM_SETBKGNDCOLOR, 0, GetSysColor((COLOR_3DFACE as i32)) as LPARAM);

    /* Enable RichEdit coloured and underlined links */
    dwMask = SendDlgItemMessageW(hDlg, (IDC_EVENTTEXTEDIT as i32), EM_GETEVENTMASK, 0, 0) as DWORD;
    SendDlgItemMessageW(hDlg, (IDC_EVENTTEXTEDIT as i32), EM_SETEVENTMASK, 0, (dwMask | ENM_LINK | ENM_MOUSEEVENTS) as LPARAM);

    /*
     * Activate automatic URL recognition by the RichEdit control. For more information, see:
     * https://learn.microsoft.com/en-us/archive/blogs/murrays/automatic-richedit-hyperlinks
     * https://learn.microsoft.com/en-us/archive/blogs/murrays/richedit-friendly-name-hyperlinks
     * https://learn.microsoft.com/en-us/windows/win32/controls/em-autourldetect
     */
    SendDlgItemMessageW(hDlg, (IDC_EVENTTEXTEDIT as i32), EM_AUTOURLDETECT, AURL_ENABLEURL as WPARAM /* | AURL_ENABLEEAURLS */, 0 as LPARAM);

    /* Note that the RichEdit control never gets themed under WinXP+; one would have to write code to simulate Edit-control theming */

    SendDlgItemMessageW(hDlg, if (*pData).bDisplayWords != 0 { (IDC_WORDSRADIO as i32) } else { (IDC_BYTESRADIO as i32) }, BM_SETCHECK, BST_CHECKED as WPARAM, 0 as LPARAM);
    SendDlgItemMessageW(hDlg, (IDC_EVENTDATAEDIT as i32), WM_SETFONT, (*pData).hMonospaceFont as WPARAM, TRUE as LPARAM);

    //ClearContents(hDlg);
    if (*pData).iEventItem != -1 {
        EnableNavigationArrows(hDlg, (TRUE) as BOOL);
        EnableDlgItem!(hDlg, (IDC_COPY as i32), (TRUE as i32));
    } else {
        EnableNavigationArrows(hDlg, (FALSE) as BOOL);
        EnableDlgItem!(hDlg, (IDC_COPY as i32), (FALSE as i32));
    }
    EnableDlgItem!(hDlg, (IDC_BYTESRADIO as i32), (FALSE as i32));
    EnableDlgItem!(hDlg, (IDC_WORDSRADIO as i32), (FALSE as i32));
}

unsafe extern "system" fn EventDetailsCtrl(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pData: PDETAILDATA = core::ptr::null_mut();

    pData = GetWindowLongPtrW(hDlg, DWLP_USER as INT) as PDETAILDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut rect: RECT = core::mem::zeroed();

            pData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, core::mem::size_of::<DETAILDATA>() as SIZE_T) as PDETAILDATA;
            if pData.is_null() {
                EndDialog(hDlg, 0 as INT_PTR);
                return TRUE as INT_PTR;
            }
            SetWindowLongPtrW(hDlg, DWLP_USER as INT, (pData as LONG_PTR) as _);

            if lParam != 0 as LPARAM {
                let DetailInfo: PEVENTDETAIL_INFO = lParam as PEVENTDETAIL_INFO;
                (*pData).EventLogFilter = (*DetailInfo).EventLogFilter;
                (*pData).iEventItem = (*DetailInfo).iEventItem;
            } else {
                (*pData).iEventItem = -(1) as INT;
            }

            (*pData).bDisplayWords = FALSE as BOOL;
            (*pData).hMonospaceFont = CreateMonospaceFont();

            GetClientRect(hDlg, core::ptr::addr_of_mut!(rect));
            (*pData).cxMin = (rect.right - rect.left) as INT;
            (*pData).cxOld = (*pData).cxMin;
            (*pData).cyMin = (rect.bottom - rect.top) as INT;
            (*pData).cyOld = (*pData).cyMin;
            (*pData).scPos.y = 0 as LONG;
            (*pData).scPos.x = (*pData).scPos.y;

            InitDetailsDlgCtrl(hDlg, pData);
            /* NOTE: Showing the event (if any) is currently delayed to later */
            // SendMessageW(hDlg, EVT_DISPLAY, TRUE, (LPARAM)pData->iEventItem);

            // OnSize(hDlg, pData, pData->cxOld, pData->cyOld);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            if !pData.is_null() {
                if !(*pData).hMonospaceFont.is_null() {
                    DeleteObject((*pData).hMonospaceFont as HGDIOBJ);
                }
                HeapFree(GetProcessHeap(), 0 as DWORD, pData as PVOID);
            }
            return TRUE as INT_PTR;
        }

        EVT_SETFILTER => {
            /* Disable the display first, before changing the current filter */
            (*pData).pevlr = core::ptr::null_mut();
            (*pData).iEventItem = -(1) as INT;
            ClearContents(hDlg);
            (*pData).EventLogFilter = lParam as PEVENTLOGFILTER;
            return TRUE as INT_PTR;
        }

        EVT_DISPLAY => {
            if wParam != 0 {
                let iEventItem: INT = lParam as INT;

                /* If no filter is set, don't change anything */
                if (*pData).EventLogFilter.is_null() {
                    return TRUE as INT_PTR;
                }

                /* If we re-enable display from previously disabled state, re-enable the buttons */
                if ((*pData).iEventItem == -(1) as INT) && (iEventItem != -(1) as INT) {
                    EnableNavigationArrows(hDlg, TRUE as BOOL);
                    EnableDlgItem!(hDlg, (IDC_COPY as i32), (TRUE as i32));
                } else
                /* If we disable display from previously enabled state, clear and disable it */
                if ((*pData).iEventItem != -(1) as INT) && (iEventItem == -(1) as INT) {
                    ClearContents(hDlg);
                }

                /* Set the new item */
                (*pData).pevlr = core::ptr::null_mut();
                (*pData).iEventItem = iEventItem;

                /* Display the event info, if there is one */
                if (*pData).iEventItem != -(1) as INT {
                    DisplayEvent(hDlg, pData);
                }
            } else {
                /* Enable or disable the Previous/Next buttons,
                 * but keep the existing contents, if any */
                EnableNavigationArrows(hDlg, (lParam != -(1) as LPARAM) as BOOL);

                // HACK: Disable the Bytes/Words mode buttons
                // because we won't have access anymore to the
                // event data. (This will be fixed in the future.)
                (*pData).pevlr = core::ptr::null_mut(); // Invalidate also the cache.
                EnableDlgItem!(hDlg, (IDC_BYTESRADIO as i32), (FALSE as i32));
                EnableDlgItem!(hDlg, (IDC_WORDSRADIO as i32), (FALSE as i32));
            }
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            'sw: {
                match (LOWORD!(wParam)) as u32 {
                    IDC_PREVIOUS | IDC_NEXT => {
                        let bPrev: BOOL = (LOWORD!(wParam) == (IDC_PREVIOUS as u16)) as BOOL;
                        let nItems: INT = ListView_GetItemCount!(hwndListView);
                        let mut iItem: INT;
                        let mut iSel: INT;

                        if nItems <= 0 {
                            /* No items? */
                            break 'sw;
                        }

                        /* Get the index of the first selected item */
                        iSel = ListView_GetNextItem!(hwndListView, -(1) as INT, LVNI_SELECTED as UINT);

                        /* Select the previous/next item from our current one */
                        iItem = ListView_GetNextItem!(hwndListView, iSel,
                            (if bPrev != 0 { LVNI_ABOVE } else { LVNI_BELOW }) as UINT);
                        if iItem < 0 || iItem >= nItems {
                            /* Confirm selection restart only if an item was previously
                             * selected. If not, just proceed with default selection. */
                            if iSel != -(1) as INT {
                                let mut szText: [WCHAR; 200] = [0 as WCHAR; 200];
                                LoadStringW(hInst,
                                    (if bPrev != 0 { IDS_CONTFROMEND } else { IDS_CONTFROMBEGINNING }) as UINT,
                                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                                    _countof!(szText) as INT);
                                if MessageBoxW(hDlg,
                                    core::ptr::addr_of_mut!(szText) as LPCWSTR,
                                    core::ptr::addr_of_mut!(szTitle) as LPCWSTR,
                                    (MB_YESNO | MB_ICONQUESTION) as UINT) != IDYES as INT
                                {
                                    break 'sw;
                                }
                            }

                            /* Determine where to restart from */
                            iItem = (if bPrev != 0 { nItems - 1 } else { 0 }) as INT;
                        }

                        /*
                         * Deselect the currently selected items in the list view.
                         * (They may be different from our current one, if multiple
                         * event details are being displayed concurrently!)
                         */
                        iSel = -(1) as INT;
                        loop {
                            iSel = ListView_GetNextItem!(hwndListView, iSel, LVNI_SELECTED as UINT);
                            if !(iSel != -(1) as INT) {
                                break;
                            }
                            ListView_SetItemState!(hwndListView, iSel,
                                0, LVIS_FOCUSED | LVIS_SELECTED);
                        }

                        /* Select the new item */
                        ListView_SetItemState!(hwndListView, iItem,
                            LVIS_FOCUSED | LVIS_SELECTED,
                            LVIS_FOCUSED | LVIS_SELECTED);
                        ListView_EnsureVisible!(hwndListView, iItem, FALSE);

                        (*pData).pevlr = core::ptr::null_mut();
                        (*pData).iEventItem = iItem;

                        /* Display the event info */
                        if !(*pData).EventLogFilter.is_null() {
                            DisplayEvent(hDlg, pData);
                        }
                        return TRUE as INT_PTR;
                    }

                    IDC_COPY => {
                        if !(*pData).EventLogFilter.is_null() {
                            CopyEventEntry(hDlg);
                        }
                        return TRUE as INT_PTR;
                    }

                    IDC_BYTESRADIO | IDC_WORDSRADIO => {
                        (*pData).bDisplayWords = (LOWORD!(wParam) == (IDC_WORDSRADIO as u16)) as BOOL;
                        if !(*pData).EventLogFilter.is_null() {
                            DisplayEventData(hDlg, pData);
                        }
                        return TRUE as INT_PTR;
                    }

                    _ => {}
                }
            }
        }

        WM_NOTIFY => {
            let hdr: LPNMHDR = lParam as LPNMHDR;

            if (*hdr).idFrom == IDC_EVENTTEXTEDIT as UINT_PTR {
                match (*hdr).code {
                    EN_LINK => {
                        OnLink(hDlg, lParam as *mut ENLINK);
                    }
                    _ => {}
                }
            }
        }

        WM_HSCROLL | WM_VSCROLL => {
            OnScroll(hDlg, pData,
                (if uMsg == WM_HSCROLL as UINT { SB_HORZ } else { SB_VERT }) as INT,
                LOWORD!(wParam));
            SetWindowLongPtrW(hDlg, DWLP_MSGRESULT as INT, (0 as LONG_PTR) as _);
            return TRUE as INT_PTR;
        }

        WM_SIZE => {
            OnSize(hDlg, pData, LOWORD!(lParam) as INT, HIWORD!(lParam) as INT);
            SetWindowLongPtrW(hDlg, DWLP_MSGRESULT as INT, (0 as LONG_PTR) as _);
            return TRUE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateEventDetailsCtrl(hInstance: HINSTANCE, hParentWnd: HWND, lParam: LPARAM) -> HWND {
    return CreateDialogParamW(
        hInstance,
        MAKEINTRESOURCEW!(IDD_EVENTDETAILS_CTRL),
        hParentWnd,
        Some(EventDetailsCtrl),
        lParam,
    );
}
