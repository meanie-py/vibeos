use crate::winhlp32::r#macro::MACRO_About;
use crate::winhlp32::r#macro::MACRO_Annotate;
use crate::winhlp32::r#macro::MACRO_BookmarkDefine;
use crate::winhlp32::r#macro::MACRO_CopyDialog;
use crate::winhlp32::r#macro::MACRO_CreateButton;
use crate::winhlp32::r#macro::MACRO_DisableButton;
use crate::winhlp32::r#macro::MACRO_Exit;
use crate::winhlp32::r#macro::MACRO_FileOpen;
use crate::winhlp32::r#macro::MACRO_HelpOn;
use crate::winhlp32::r#macro::MACRO_HelpOnTop;
use crate::winhlp32::r#macro::MACRO_History;
use crate::winhlp32::r#macro::MACRO_JumpContents;
use crate::winhlp32::r#macro::MACRO_JumpContext;
use crate::winhlp32::r#macro::MACRO_JumpHash;
use crate::winhlp32::r#macro::MACRO_PopupContext;
use crate::winhlp32::r#macro::MACRO_Print;
use crate::winhlp32::r#macro::MACRO_PrinterSetup;
use crate::winhlp32::r#macro::MACRO_SetContents;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Globals: WINHELP_GLOBALS = WINHELP_GLOBALS {
    wVersion: 3 as UINT,
    hInstance: NULL,
    isBook: (TRUE as i32),
    active_win: ::core::ptr::null::<WINHELP_WINDOW>() as *mut WINHELP_WINDOW,
    active_popup: ::core::ptr::null::<WINHELP_WINDOW>() as *mut WINHELP_WINDOW,
    win_list: ::core::ptr::null::<WINHELP_WINDOW>() as *mut WINHELP_WINDOW,
    button_proc: None,
    dlls: ::core::ptr::null::<WINHELP_DLL>() as *mut WINHELP_DLL,
    history: tagPageSet {
        set: [
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
            WINHELP_WNDPAGE {
                page: ::core::ptr::null::<HLPFILE_PAGE>() as *mut HLPFILE_PAGE,
                wininfo: ::core::ptr::null::<HLPFILE_WINDOWINFO>() as *mut HLPFILE_WINDOWINFO,
                relative: 0,
            },
        ],
        index: 0 as ::core::ffi::c_uint,
    },
    hButtonFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
};

// #define NONAMELESSUNION  (no Rust equivalent needed)

// WINE_DEFAULT_DEBUG_CHANNEL(winhelp): the channel storage is the prelude's generated
// wine-debug-channel block (__wine_dbch_winhelp / __wine_dbch___default).

// WINHELP_GLOBALS Globals = {3, NULL, TRUE, NULL, NULL, NULL, NULL, NULL, {{{NULL,NULL}},0}, NULL};
// Field layout of WINHELP_GLOBALS is defined elsewhere; intended initial values:
// [0]=3, [1]=NULL, [2]=TRUE, [3..=7]=NULL, [8]={{{NULL,NULL}},0}, [9]=NULL

pub const CTL_ID_BUTTON: u32 = 0x700;

pub const CTL_ID_TEXT: u32 = 0x701;

#[repr(C)]
pub struct WINHELP {
    pub size: WORD,
    pub command: WORD,
    pub data: LONG,
    pub reserved: LONG,
    pub ofsFilename: WORD,
    pub ofsData: WORD,
}
pub type LPWINHELP = *mut WINHELP;

#[repr(C)]
pub struct index_data {
    pub hlpfile: *mut HLPFILE,
    pub jump: BOOL,
    pub offset: ULONG,
}

unsafe extern "C" fn WINHELP_InitFonts(hWnd: HWND) {
    let win: *mut WINHELP_WINDOW = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
    let mut logfontlist: [LOGFONTW; 7] = [
        LOGFONTW {
            lfHeight: -(10) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 400 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(12) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 700 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(12) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 700 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(12) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 400 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(12) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 700 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(10) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 700 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
        LOGFONTW {
            lfHeight: -(8) as LONG,
            lfWidth: 0 as LONG,
            lfEscapement: 0 as LONG,
            lfOrientation: 0 as LONG,
            lfWeight: 400 as LONG,
            lfItalic: 0 as BYTE,
            lfUnderline: 0 as BYTE,
            lfStrikeOut: 0 as BYTE,
            lfCharSet: DEFAULT_CHARSET as BYTE,
            lfOutPrecision: 0 as BYTE,
            lfClipPrecision: 0 as BYTE,
            lfQuality: 0 as BYTE,
            lfPitchAndFamily: 32 as BYTE,
            lfFaceName: [
                'H' as WCHAR, 'e' as WCHAR, 'l' as WCHAR, 'v' as WCHAR, 0 as WCHAR,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        },
    ];
    static mut fonts: [HFONT; 7] = [core::ptr::null_mut(); 7];
    static mut init: BOOL = (FALSE as i32);

    (*win).fonts_len = ARRAY_SIZE!(logfontlist) as UINT;
    (*win).fonts = &raw mut fonts as *mut HFONT;

    if init == 0 {
        let mut i: UINT;

        i = 0;
        while i < ARRAY_SIZE!(logfontlist) as UINT {
            fonts[i as usize] = CreateFontIndirectW((&raw mut logfontlist as *mut LOGFONTW).add(i as usize));
            i = i.wrapping_add(1);
        }

        init = (TRUE as i32);
    }
}

unsafe extern "system" fn WINHELP_RtfStreamIn(
    mut cookie: DWORD_PTR,
    mut buff: *mut BYTE,
    mut cb: LONG,
    mut pcb: *mut LONG,
) -> DWORD {
    let mut rd: *mut RtfData = cookie as *mut RtfData;

    if (*rd).where_ >= (*rd).ptr {
        return 1;
    }
    if (*rd).where_.offset(cb as isize) > (*rd).ptr {
        cb = (*rd).ptr.offset_from((*rd).where_) as ::core::ffi::c_longlong as LONG;
    }
    memcpy(
        buff as *mut ::core::ffi::c_void,
        (*rd).where_ as *const ::core::ffi::c_void,
        cb as usize,
    );
    (*rd).where_ = (*rd).where_.offset(cb as isize);
    *pcb = cb;
    return 0;
}

unsafe extern "C" fn WINHELP_SetupText(hTextWnd: HWND, win: *mut WINHELP_WINDOW, relative: ULONG) {
    static mut emptyW: [WCHAR; 1] = [0; 1];
    /* At first clear area - needed by EM_POSFROMCHAR/EM_SETSCROLLPOS */
    SendMessageW(
        hTextWnd,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of!(emptyW) as *const WCHAR as LPARAM,
    );
    SendMessageW(
        hTextWnd,
        WM_SETREDRAW as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );
    SendMessageW(
        hTextWnd,
        EM_SETBKGNDCOLOR as UINT,
        0 as WPARAM,
        (*(*win).info).sr_color as LPARAM,
    );
    /* set word-wrap to window size (undocumented) */
    SendMessageW(
        hTextWnd,
        EM_SETTARGETDEVICE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    if !(*win).page.is_null() {
        let mut rd: RtfData = core::mem::zeroed();
        let mut es: EDITSTREAM = core::mem::zeroed();
        let mut cp: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        let mut ptl: POINTL = core::mem::zeroed();
        let mut pt: POINT = core::mem::zeroed();

        if HLPFILE_BrowsePage(
            (*win).page,
            &raw mut rd,
            (*win).font_scale,
            relative as ::core::ffi::c_uint,
        ) != 0
        {
            rd.where_ = rd.data;
            es.dwCookie = &raw mut rd as DWORD_PTR;
            es.dwError = 0 as DWORD;
            es.pfnCallback = Some(WINHELP_RtfStreamIn);

            SendMessageW(
                hTextWnd,
                EM_STREAMIN as UINT,
                SF_RTF as WPARAM,
                &raw mut es as LPARAM,
            );
            cp = rd.char_pos_rel;
        }
        /* FIXME: else leaking potentially the rd.first_link chain */
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            rd.data as PVOID,
        );
        SendMessageW(
            hTextWnd,
            EM_POSFROMCHAR as UINT,
            &raw mut ptl as WPARAM,
            (if cp != 0 {
                cp.wrapping_sub(1 as ::core::ffi::c_uint)
            } else {
                0 as ::core::ffi::c_uint
            }) as LPARAM,
        );
        pt.x = 0 as LONG;
        pt.y = ptl.y;
        SendMessageW(
            hTextWnd,
            EM_SETSCROLLPOS as UINT,
            0 as WPARAM,
            &raw mut pt as LPARAM,
        );
    }
    SendMessageW(
        hTextWnd,
        WM_SETREDRAW as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );
    RedrawWindow(
        hTextWnd,
        core::ptr::null(),
        core::ptr::null_mut(),
        (RDW_FRAME | RDW_INVALIDATE) as UINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_GetOpenFileName(mut lpszFile: LPSTR, mut len: ::core::ffi::c_int) -> BOOL {
    let mut openfilename: OPENFILENAMEA = core::mem::zeroed();
    let mut szDir: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szzFilter: [CHAR; (2 * MAX_STRING_LEN + 100) as _] =
        [0; (2 * MAX_STRING_LEN + 100) as _];
    let mut p: LPSTR = szzFilter.as_mut_ptr();

    WINE_TRACE!(b"()\n\0".as_ptr() as *const CHAR);

    LoadStringA(
        Globals.hInstance as HINSTANCE,
        STID_HELP_FILES_HLP as UINT,
        p,
        MAX_STRING_LEN as ::core::ffi::c_int,
    );
    p = p.add(strlen(p as *const CHAR) + 1);
    strcpy(p as *mut CHAR, b"*.hlp\0".as_ptr() as *const CHAR);
    p = p.add(strlen(p as *const CHAR) + 1);
    LoadStringA(
        Globals.hInstance as HINSTANCE,
        STID_ALL_FILES as UINT,
        p,
        MAX_STRING_LEN as ::core::ffi::c_int,
    );
    p = p.add(strlen(p as *const CHAR) + 1);
    strcpy(p as *mut CHAR, b"*.*\0".as_ptr() as *const CHAR);
    p = p.add(strlen(p as *const CHAR) + 1);
    *p = 0 as CHAR;

    GetCurrentDirectoryA(
        core::mem::size_of::<[CHAR; (MAX_PATH) as _]>() as DWORD,
        core::ptr::addr_of_mut!(szDir) as LPSTR,
    );

    *lpszFile.offset(0 as isize) = 0 as CHAR;

    openfilename.lStructSize = core::mem::size_of::<OPENFILENAMEA>() as DWORD;
    openfilename.hwndOwner = if !Globals.active_win.is_null() {
        (*Globals.active_win).hMainWnd
    } else {
        core::ptr::null_mut()
    };
    openfilename.hInstance = Globals.hInstance as HINSTANCE;
    openfilename.lpstrFilter = core::ptr::addr_of_mut!(szzFilter) as *mut CHAR as LPCSTR;
    openfilename.lpstrCustomFilter = core::ptr::null_mut();
    openfilename.nMaxCustFilter = 0 as DWORD;
    openfilename.nFilterIndex = 1 as DWORD;
    openfilename.lpstrFile = lpszFile;
    openfilename.nMaxFile = len as DWORD;
    openfilename.lpstrFileTitle = core::ptr::null_mut();
    openfilename.nMaxFileTitle = 0 as DWORD;
    openfilename.lpstrInitialDir = core::ptr::addr_of_mut!(szDir) as *mut CHAR as LPCSTR;
    openfilename.lpstrTitle = core::ptr::null();
    openfilename.Flags = (OFN_EXPLORER | OFN_ENABLESIZING | OFN_HIDEREADONLY | OFN_READONLY) as DWORD;
    openfilename.nFileOffset = 0 as WORD;
    openfilename.nFileExtension = 0 as WORD;
    openfilename.lpstrDefExt = core::ptr::null();
    openfilename.lCustData = 0 as LPARAM;
    openfilename.lpfnHook = None;
    openfilename.lpTemplateName = core::ptr::null();

    return GetOpenFileNameA(core::ptr::addr_of_mut!(openfilename));
}

unsafe extern "C" fn WINHELP_MessageBoxIDS_s(ids_text: UINT, str: LPCSTR, ids_title: UINT, type_: WORD) -> INT {
    let mut text: [CHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut newtext: [CHAR; (MAX_STRING_LEN + MAX_PATH) as _] = [0; (MAX_STRING_LEN + MAX_PATH) as _];

    LoadStringA(
        Globals.hInstance as HINSTANCE,
        ids_text,
        core::ptr::addr_of_mut!(text) as LPSTR,
        core::mem::size_of::<[CHAR; (MAX_STRING_LEN as _)]>() as i32,
    );
    wsprintfA(
        core::ptr::addr_of_mut!(newtext) as LPSTR,
        core::ptr::addr_of_mut!(text) as *mut CHAR as LPCSTR,
        str,
    );

    return MessageBoxA(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(newtext) as *mut CHAR as LPCSTR,
        MAKEINTRESOURCEA!(ids_title),
        type_ as UINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_LookupHelpFile(mut lpszFile: LPCSTR) -> *mut HLPFILE {
    let mut hlpfile: *mut HLPFILE = ::core::ptr::null_mut::<HLPFILE>();
    let mut szFullName: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szAddPath: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !Globals.active_win.is_null()
        && !(*Globals.active_win).page.is_null()
        && !(*(*Globals.active_win).page).file.is_null()
    {
        strcpy(
            &raw mut szAddPath as *mut ::core::ffi::c_char,
            (*(*(*Globals.active_win).page).file).lpszPath as *const ::core::ffi::c_char,
        );
        p = (strrchr((&raw mut szAddPath).cast(), '\\' as i32)).cast();
        if !p.is_null() {
            *p = 0 as ::core::ffi::c_char;
        }
    }
    if SearchPathA(
        ::core::ptr::null::<CHAR>(),
        lpszFile,
        b".hlp\0" as *const u8 as LPCSTR,
        MAX_PATH as DWORD,
        &raw mut szFullName as LPSTR,
        ::core::ptr::null_mut::<LPSTR>(),
    ) == 0
        && SearchPathA(
            &raw mut szAddPath as *mut ::core::ffi::c_char as LPCSTR,
            lpszFile,
            b".hlp\0" as *const u8 as LPCSTR,
            MAX_PATH as DWORD,
            &raw mut szFullName as LPSTR,
            ::core::ptr::null_mut::<LPSTR>(),
        ) == 0
    {
        if WINHELP_MessageBoxIDS_s(
            STID_FILE_NOT_FOUND_s as UINT,
            lpszFile,
            STID_WHERROR as UINT,
            (MB_YESNO | MB_ICONQUESTION) as WORD,
        ) != (IDYES as i32)
        {
            return ::core::ptr::null_mut::<HLPFILE>();
        }
        if WINHELP_GetOpenFileName(&raw mut szFullName as LPSTR, (MAX_PATH as i32)) == 0 {
            return ::core::ptr::null_mut::<HLPFILE>();
        }
    }
    hlpfile = HLPFILE_ReadHlpFile(&raw mut szFullName as *mut ::core::ffi::c_char as LPCSTR);
    if hlpfile.is_null() {
        WINHELP_MessageBoxIDS_s(
            STID_HLPFILE_ERROR_s as UINT,
            lpszFile,
            STID_WHERROR as UINT,
            (MB_OK | MB_ICONSTOP) as WORD,
        );
    }
    return hlpfile;
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_GetWindowInfo(mut hlpfile: *mut HLPFILE, mut name: LPCSTR) -> *mut HLPFILE_WINDOWINFO {
    static mut mwi: HLPFILE_WINDOWINFO = HLPFILE_WINDOWINFO {
        type_: [0; 10],
        name: [0; 9],
        caption: [0; 51],
        origin: tagPOINT { x: 0, y: 0 },
        size: tagSIZE { cx: 0, cy: 0 },
        style: 0,
        win_style: 0,
        sr_color: 0,
        nsr_color: 0,
    };
    let mut i: ::core::ffi::c_uint = 0;
    if false {
        // Non-__REACTOS__ branch of the caption code (preprocessed out in the C build).
        let mut tmp = [0 as ::core::ffi::c_char; 128];
        LoadStringA(
            Globals.hInstance as HINSTANCE,
            STID_WINE_HELP as UINT,
            tmp.as_mut_ptr() as LPSTR,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as ::core::ffi::c_int,
        );
        snprintf(
            &raw mut mwi.caption as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>() as size_t,
            b"%s %s - %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*hlpfile).lpszTitle,
            tmp.as_ptr(),
            (*hlpfile).lpszPath,
        );
    }
    if name.is_null() || *name.offset(0 as isize) == 0 {
        name = &raw mut (*(*Globals.active_win).info).name as *mut ::core::ffi::c_char as LPCSTR;
    }
    if !hlpfile.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < (*hlpfile).numWindows {
            if lstrcmpiA(
                &raw mut (*(*hlpfile).windows.offset(i as isize)).name as *mut ::core::ffi::c_char
                    as LPCSTR,
                name,
            ) == 0
            {
                return (*hlpfile).windows.offset(i as isize) as *mut HLPFILE_WINDOWINFO;
            }
            i = i.wrapping_add(1);
        }
    }
    if strcmp(
        name as *const ::core::ffi::c_char,
        b"main\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        WINE_FIXME!(
            b"Couldn't find window info for %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            debugstr_a(name as *const ::core::ffi::c_char),
        );
        // C: assert(0) -- live only in a debug (non-NDEBUG) build, as debug_assert! is.
        debug_assert!(false, "Couldn't find window info");
        return ::core::ptr::null_mut();
    }
    if mwi.name[0 as usize] == 0 {
        strcpy(
            &raw mut mwi.type_ as *mut ::core::ffi::c_char,
            b"primary\0".as_ptr() as *const ::core::ffi::c_char,
        );
        strcpy(
            &raw mut mwi.name as *mut ::core::ffi::c_char,
            b"main\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !hlpfile.is_null()
            && *(*hlpfile).lpszTitle.offset(0 as isize) != 0
        {
            snprintf(
                &raw mut mwi.caption as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 51]>() as size_t,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*hlpfile).lpszTitle,
            );
        } else {
            LoadStringA(
                Globals.hInstance as HINSTANCE,
                STID_WINE_HELP as UINT,
                &raw mut mwi.caption as LPSTR,
                ::core::mem::size_of::<[::core::ffi::c_char; 51]>() as ::core::ffi::c_int,
            );
        }
        mwi.size.cy = CW_USEDEFAULT as LONG;
        mwi.size.cx = mwi.size.cy;
        mwi.origin.y = mwi.size.cx;
        mwi.origin.x = mwi.origin.y;
        mwi.style = (SW_SHOW as i32);
        mwi.win_style = WS_OVERLAPPEDWINDOW as DWORD;
        mwi.nsr_color = 0xFFFFFF as COLORREF;
        mwi.sr_color = mwi.nsr_color;
    }
    return &raw mut mwi as *mut HLPFILE_WINDOWINFO;
}

unsafe extern "C" fn WINHELP_GetPopupWindowInfo(hlpfile: *mut HLPFILE, parent: *mut WINHELP_WINDOW, mouse: LPARAM) -> *mut HLPFILE_WINDOWINFO {
    static mut wi: HLPFILE_WINDOWINFO = unsafe { core::mem::zeroed() };

    let mut parent_rect: RECT = core::mem::zeroed();

    wi.caption[0 as usize] = 0;
    wi.name[0 as usize] = wi.caption[0 as usize];
    wi.type_[0 as usize] = wi.name[0 as usize];

    /* Calculate horizontal size and position of a popup window */
    GetWindowRect((*parent).hMainWnd, core::ptr::addr_of_mut!(parent_rect));
    wi.size.cx = (parent_rect.right - parent_rect.left) / 2;
    wi.size.cy = 10; /* need a non null value, so that borders are taken into account while computing */

    wi.origin.x = LOWORD!(mouse) as i16 as LONG;
    wi.origin.y = HIWORD!(mouse) as i16 as LONG;
    ClientToScreen((*parent).hMainWnd, core::ptr::addr_of_mut!(wi.origin));
    wi.origin.x -= wi.size.cx / 2;
    wi.origin.x = min!(wi.origin.x, GetSystemMetrics(0 /* SM_CXSCREEN */) - wi.size.cx);
    wi.origin.x = max!(wi.origin.x, 0);

    wi.style = (SW_SHOW as i32);
    wi.win_style = WS_POPUP | WS_BORDER;
    if (*(*(*parent).page).file).has_popup_color != 0 {
        wi.sr_color = (*(*(*parent).page).file).popup_color;
    } else {
        wi.sr_color = (*(*parent).info).sr_color;
    }
    wi.nsr_color = 0xFFFFFF;

    return core::ptr::addr_of_mut!(wi);
}

unsafe extern "C" fn WINHELP_HasWorkingWindow() -> BOOL {
    if Globals.active_win.is_null() {
        return (FALSE) as BOOL;
    }
    if !(*Globals.active_win).next.is_null() || Globals.win_list != Globals.active_win {
        return (TRUE) as BOOL;
    }
    return (!(*Globals.active_win).page.is_null() && !(*(*Globals.active_win).page).file.is_null()) as BOOL;
}

unsafe extern "C" fn WINHELP_HandleCommand(hSrcWnd: HWND, lParam: LPARAM) -> LRESULT {
    let cds: *mut COPYDATASTRUCT = lParam as *mut COPYDATASTRUCT;
    let mut wh: *mut WINHELP;

    if (*cds).dwData != 0xA1DE505 {
        WINE_FIXME!(b"Wrong magic number (%08lx)\n\0".as_ptr() as *const i8, (*cds).dwData);
        return 0;
    }

    wh = (*cds).lpData as *mut WINHELP;

    if !wh.is_null() {
        let ptr: *mut i8 = if (*wh).ofsFilename != 0 {
            (wh as LPSTR).offset((*wh).ofsFilename as isize)
        } else {
            core::ptr::null_mut()
        };

        WINE_TRACE!(
            b"Got[%u]: cmd=%u data=%08x fn=%s\n\0".as_ptr() as *const i8,
            (*wh).size as core::ffi::c_uint,
            (*wh).command as core::ffi::c_uint,
            (*wh).data,
            debugstr_a(ptr),
        );
        match ((*wh).command) as u32 {
            HELP_CONTEXT => {
                if !ptr.is_null() {
                    MACRO_JumpContext(ptr as LPCSTR, b"main\0".as_ptr() as LPCSTR, (*wh).data);
                }
                if WINHELP_HasWorkingWindow() == 0 {
                    MACRO_Exit();
                }
            }
            HELP_QUIT => {
                MACRO_Exit();
            }
            HELP_CONTENTS => {
                if !ptr.is_null() {
                    MACRO_JumpContents(ptr as LPCSTR, b"main\0".as_ptr() as LPCSTR);
                }
                if WINHELP_HasWorkingWindow() == 0 {
                    MACRO_Exit();
                }
            }
            HELP_HELPONHELP => {
                MACRO_HelpOn();
                if WINHELP_HasWorkingWindow() == 0 {
                    MACRO_Exit();
                }
            }
            /* case HELP_SETINDEX: */
            HELP_SETCONTENTS => {
                if !ptr.is_null() {
                    MACRO_SetContents(ptr as LPCSTR, (*wh).data);
                }
            }
            HELP_CONTEXTPOPUP => {
                if !ptr.is_null() {
                    MACRO_PopupContext(ptr as LPCSTR, (*wh).data);
                }
            }
            /* case HELP_FORCEFILE: */
            /* case HELP_CONTEXTMENU: */
            HELP_FINDER => {
                /* in fact, should be the topic dialog box */
                WINE_FIXME!(b"HELP_FINDER: stub\n\0".as_ptr() as *const i8);
                if !ptr.is_null() {
                    MACRO_JumpHash(ptr as LPCSTR, b"main\0".as_ptr() as LPCSTR, 0);
                }
            }
            /* case HELP_WM_HELP: */
            /* case HELP_SETPOPUP_POS: */
            /* case HELP_KEY: */
            /* case HELP_COMMAND: */
            /* case HELP_PARTIALKEY: */
            /* case HELP_MULTIKEY: */
            /* case HELP_SETWINPOS: */
            _ => {
                WINE_FIXME!(
                    b"Unhandled command (%x) for remote winhelp control\n\0".as_ptr() as *const i8,
                    (*wh).command as core::ffi::c_uint,
                );
            }
        }
    }
    /* Always return success for now */
    1
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_LayoutMainWindow(win: *mut WINHELP_WINDOW) {
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut button_box_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut text_top: INT = 0;
    let hButtonBoxWnd: HWND = GetDlgItem((*win).hMainWnd, (CTL_ID_BUTTON as i32));
    let hTextWnd: HWND = GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32));

    GetClientRect((*win).hMainWnd, core::ptr::addr_of_mut!(rect));

    /* Update button box and text Window */
    SetWindowPos(
        hButtonBoxWnd,
        HWND_TOP,
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        0,
    );

    if GetWindowRect(hButtonBoxWnd, core::ptr::addr_of_mut!(button_box_rect)) != 0 {
        text_top = rect.top + button_box_rect.bottom - button_box_rect.top;
    }

    SetWindowPos(
        hTextWnd,
        HWND_TOP,
        rect.left,
        text_top,
        rect.right - rect.left,
        rect.bottom - text_top,
        0,
    );
}

unsafe extern "C" fn WINHELP_DeleteButtons(mut win: *mut WINHELP_WINDOW) {
    let mut b: *mut WINHELP_BUTTON = core::ptr::null_mut::<WINHELP_BUTTON>();
    let mut bp: *mut WINHELP_BUTTON = core::ptr::null_mut::<WINHELP_BUTTON>();
    b = (*win).first_button;
    while !b.is_null() {
        DestroyWindow((*b).hWnd);
        bp = (*b).next;
        HeapFree(GetProcessHeap(), 0 as DWORD, b as PVOID);
        b = bp;
    }
    (*win).first_button = core::ptr::null_mut::<WINHELP_BUTTON>();
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_DeleteBackSet(win: *mut WINHELP_WINDOW) {
    let mut i: ::core::ffi::c_uint = 0;
    while i < (*win).back.index {
        HLPFILE_FreeHlpFile((*(*win).back.set[i as usize].page).file as *mut HLPFILE);
        (*win).back.set[i as usize].page = core::ptr::null_mut::<HLPFILE_PAGE>();
        i = i.wrapping_add(1);
    }
    (*win).back.index = 0;
}

unsafe extern "C" fn WINHELP_DeletePageLinks(page: *mut HLPFILE_PAGE) {
    let mut curr: *mut HLPFILE_LINK = ::core::ptr::null_mut::<HLPFILE_LINK>();
    let mut next: *mut HLPFILE_LINK = ::core::ptr::null_mut::<HLPFILE_LINK>();
    curr = (*page).first_link as *mut HLPFILE_LINK;
    while !curr.is_null() {
        next = (*curr).next as *mut HLPFILE_LINK;
        HeapFree(GetProcessHeap(), 0, curr as PVOID);
        curr = next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_GrabWindow(win: *mut WINHELP_WINDOW) -> *mut WINHELP_WINDOW {
    WINE_TRACE!(b"Grab %p#%d++\n\0".as_ptr() as *const i8, win, (*win).ref_count);
    (*win).ref_count += 1;
    return win;
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_ReleaseWindow(win: *mut WINHELP_WINDOW) -> BOOL {
    WINE_TRACE!(b"Release %p#%d--\n\0".as_ptr() as *const i8, win, (*win).ref_count);

    (*win).ref_count = (*win).ref_count.wrapping_sub(1);
    if (*win).ref_count == 0 {
        DestroyWindow((*win).hMainWnd);
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WINHELP_DeleteWindow(win: *mut WINHELP_WINDOW) {
    let mut w: *mut *mut WINHELP_WINDOW = core::ptr::null_mut();
    let mut bExit: BOOL = 0;
    let mut hTextWnd: HWND = core::ptr::null_mut();

    w = core::ptr::addr_of_mut!(Globals.win_list);
    while !(*w).is_null() {
        if *w == win {
            *w = (*win).next as *mut WINHELP_WINDOW;
            break;
        } else {
            w = core::ptr::addr_of_mut!((**w).next) as *mut *mut WINHELP_WINDOW;
        }
    }
    bExit = (Globals.wVersion >= 4 as UINT
        && lstrcmpiA(
            core::ptr::addr_of_mut!((*(*win).info).name) as *mut ::core::ffi::c_char as LPCSTR,
            b"main\0".as_ptr() as LPCSTR,
        ) == 0) as ::core::ffi::c_int as BOOL;

    if Globals.active_win == win {
        Globals.active_win = Globals.win_list;
        if !Globals.win_list.is_null() {
            SetActiveWindow((*Globals.win_list).hMainWnd);
        }
    }

    if win == Globals.active_popup {
        Globals.active_popup = core::ptr::null_mut();
    }

    hTextWnd = GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32));
    SetWindowLongPtrA(
        hTextWnd,
        GWLP_WNDPROC,
        ::core::mem::transmute::<WNDPROC, LONG_PTR>((*win).origRicheditWndProc) as _,
    );

    WINHELP_DeleteButtons(win);

    if !(*win).page.is_null() {
        WINHELP_DeletePageLinks((*win).page);
    }
    if !(*win).hHistoryWnd.is_null() {
        DestroyWindow((*win).hHistoryWnd);
    }

    DeleteObject((*win).hBrush as HGDIOBJ);

    WINHELP_DeleteBackSet(win);

    if !(*win).page.is_null() {
        HLPFILE_FreeHlpFile((*(*win).page).file as *mut HLPFILE);
    }
    HeapFree(GetProcessHeap(), 0, win as PVOID);

    if bExit != 0 {
        MACRO_Exit();
    }
    if Globals.win_list.is_null() {
        PostQuitMessage(0);
    }
}

unsafe extern "C" fn WINHELP_GetCaption(wpage: *mut WINHELP_WNDPAGE) -> *mut core::ffi::c_char {
    if (*(*wpage).wininfo).caption[0] != 0 {
        return core::ptr::addr_of_mut!((*(*wpage).wininfo).caption) as *mut core::ffi::c_char;
    }
    return (*(*(*wpage).page).file).lpszTitle as *mut core::ffi::c_char;
}

unsafe extern "C" fn WINHELP_RememberPage(win: *mut WINHELP_WINDOW, wpage: *mut WINHELP_WNDPAGE) {
    let mut num: ::core::ffi::c_uint;

    if Globals.history.index == 0
        || Globals.history.set[0 as usize].page != (*wpage).page
    {
        num = ARRAY_SIZE!(Globals.history.set) as ::core::ffi::c_uint;
        /* we're full, remove latest entry */
        if Globals.history.index == num {
            HLPFILE_FreeHlpFile(
                (*Globals.history.set[(num - 1) as usize].page).file
                    as *mut HLPFILE,
            );
            Globals.history.index -= 1;
        }
        memmove(
            core::ptr::addr_of_mut!(Globals.history.set[1 as usize])
                as *mut ::core::ffi::c_void,
            core::ptr::addr_of!(Globals.history.set[0 as usize])
                as *const ::core::ffi::c_void,
            (Globals.history.index as usize)
                * core::mem::size_of_val(
                    &*core::ptr::addr_of!(Globals.history.set[0 as usize]),
                ),
        );
        Globals.history.set[0 as usize] = *wpage;
        Globals.history.index += 1;
        (*(*(*wpage).page).file).wRefCount += 1;
    }
    if !(*win).hHistoryWnd.is_null() {
        InvalidateRect((*win).hHistoryWnd, core::ptr::null(), (TRUE) as BOOL);
    }

    num = ARRAY_SIZE!((*win).back.set) as ::core::ffi::c_uint;
    if (*win).back.index == num {
        /* we're full, remove latest entry */
        HLPFILE_FreeHlpFile(
            (*(*win).back.set[0 as usize].page).file as *mut HLPFILE,
        );
        memmove(
            core::ptr::addr_of_mut!((*win).back.set[0 as usize])
                as *mut ::core::ffi::c_void,
            core::ptr::addr_of!((*win).back.set[1 as usize])
                as *const ::core::ffi::c_void,
            ((num - 1) as usize)
                * core::mem::size_of_val(
                    &*core::ptr::addr_of!((*win).back.set[0 as usize]),
                ),
        );
        (*win).back.index -= 1;
    }
    let fresh = (*win).back.index;
    (*win).back.index += 1;
    (*win).back.set[fresh as usize] = *wpage;
    (*(*(*wpage).page).file).wRefCount += 1;
}

unsafe extern "C" fn WINHELP_FindLink(win: *mut WINHELP_WINDOW, pos: LPARAM) -> *mut HLPFILE_LINK {
    let mut link: *mut HLPFILE_LINK;
    let mut mouse_ptl: POINTL = POINTL { x: 0, y: 0 };
    let mut char_ptl: POINTL = POINTL { x: 0, y: 0 };
    let mut char_next_ptl: POINTL = POINTL { x: 0, y: 0 };
    let mut cp: DWORD;

    if (*win).page.is_null() {
        return core::ptr::null_mut();
    }

    mouse_ptl.x = (LOWORD!(pos as DWORD) as i16) as LONG;
    mouse_ptl.y = (HIWORD!(pos as DWORD) as i16) as LONG;
    cp = SendMessageW(
        GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32)),
        EM_CHARFROMPOS as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(mouse_ptl) as LPARAM,
    ) as DWORD;

    link = (*(*win).page).first_link;
    'scan: while !link.is_null() {
        'body: {
            if (*link).cpMin <= cp && cp <= (*link).cpMax {
                /* check whether we're at end of line */
                SendMessageW(
                    GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32)),
                    EM_POSFROMCHAR as UINT,
                    core::ptr::addr_of_mut!(char_ptl) as LPARAM as WPARAM,
                    cp as LPARAM,
                );
                SendMessageW(
                    GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32)),
                    EM_POSFROMCHAR as UINT,
                    core::ptr::addr_of_mut!(char_next_ptl) as LPARAM as WPARAM,
                    cp.wrapping_add(1 as DWORD) as LPARAM,
                );
                if (*link).bHotSpot() != 0 {
                    let hslink: *mut HLPFILE_HOTSPOTLINK = link as *mut HLPFILE_HOTSPOTLINK;
                    if (mouse_ptl.x as LONG) < char_ptl.x.wrapping_add((*hslink).x as LONG)
                        || (mouse_ptl.x as LONG)
                            >= char_ptl
                                .x
                                .wrapping_add((*hslink).x as LONG)
                                .wrapping_add((*hslink).width as LONG)
                        || (mouse_ptl.y as LONG) < char_ptl.y.wrapping_add((*hslink).y as LONG)
                        || (mouse_ptl.y as LONG)
                            >= char_ptl
                                .y
                                .wrapping_add((*hslink).y as LONG)
                                .wrapping_add((*hslink).height as LONG)
                    {
                        break 'body;
                    }
                    break 'scan;
                }
                if char_next_ptl.y != char_ptl.y || mouse_ptl.x >= char_next_ptl.x {
                    link = core::ptr::null_mut();
                }
                break 'scan;
            }
        }
        link = (*link).next as *mut HLPFILE_LINK;
    }
    return link;
}

unsafe extern "system" fn WINHELP_RicheditWndProc(
    mut hWnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut win: *mut WINHELP_WINDOW = GetWindowLongPtrW(GetParent(hWnd), 0) as *mut WINHELP_WINDOW;
    let mut messagePos: DWORD = 0;
    let mut pt: POINT = core::mem::zeroed();
    match msg {
        WM_SETCURSOR => {
            messagePos = GetMessagePos();
            pt.x = LOWORD!(messagePos) as ::core::ffi::c_short as LONG;
            pt.y = HIWORD!(messagePos) as ::core::ffi::c_short as LONG;
            ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt));
            if !(*win).page.is_null()
                && !WINHELP_FindLink(win, MAKELPARAM!(pt.x as WORD, pt.y as WORD)).is_null()
            {
                SetCursor((*win).hHandCur);
                return 0 as LRESULT;
            }
            /* fall through */
        }
        _ => {}
    }
    return CallWindowProcA((*win).origRicheditWndProc, hWnd, msg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_CreateHelpWindow(mut wpage: *mut WINHELP_WNDPAGE, mut nCmdShow: ::core::ffi::c_int, mut remember: BOOL) -> BOOL {
    let mut win: *mut WINHELP_WINDOW = ::core::ptr::null_mut();
    let mut bPrimary: BOOL = 0;
    let mut bPopup: BOOL = 0;
    let mut bReUsed: BOOL = FALSE as BOOL;
    let mut hIcon: HICON = ::core::ptr::null_mut::<HICON__>();
    let mut hTextWnd: HWND = ::core::ptr::null_mut::<HWND__>();
    bPrimary = (lstrcmpiA(
        &raw mut (*(*wpage).wininfo).name as *mut ::core::ffi::c_char as LPCSTR,
        b"main\0" as *const u8 as LPCSTR,
    ) == 0) as ::core::ffi::c_int as BOOL;
    bPopup = (bPrimary == 0
        && ((*(*wpage).wininfo).win_style & (WS_POPUP) as DWORD) != 0)
        as ::core::ffi::c_int as BOOL;

    if bPopup == 0 {
        win = Globals.win_list;
        while !win.is_null() {
            if lstrcmpiA(
                &raw mut (*(*win).info).name as *mut ::core::ffi::c_char as LPCSTR,
                &raw mut (*(*wpage).wininfo).name as *mut ::core::ffi::c_char as LPCSTR,
            ) == 0
            {
                if (*win).page == (*wpage).page && (*win).info == (*wpage).wininfo {
                    return TRUE as BOOL;
                }
                WINHELP_DeleteButtons(win);
                bReUsed = TRUE as BOOL;
                SetWindowTextA(
                    (*win).hMainWnd,
                    WINHELP_GetCaption(wpage) as LPCSTR,
                );
                if (*win).info != (*wpage).wininfo {
                    let mut pt: POINT = POINT {
                        x: (0) as LONG,
                        y: (0) as LONG,
                    };
                    let mut sz: SIZE = SIZE {
                        cx: (0) as LONG,
                        cy: (0) as LONG,
                    };
                    let mut flags: DWORD =
                        ((SWP_NOSIZE) | (SWP_NOMOVE)) as DWORD;
                    if (*(*wpage).wininfo).origin.x != (CW_USEDEFAULT) as LONG
                        && (*(*wpage).wininfo).origin.y != (CW_USEDEFAULT) as LONG
                    {
                        pt = (*(*wpage).wininfo).origin;
                        flags &= !((SWP_NOSIZE) as DWORD);
                    }
                    if (*(*wpage).wininfo).size.cx != (CW_USEDEFAULT) as LONG
                        && (*(*wpage).wininfo).size.cy != (CW_USEDEFAULT) as LONG
                    {
                        sz = (*(*wpage).wininfo).size;
                        flags &= !((SWP_NOMOVE) as DWORD);
                    }
                    SetWindowPos(
                        (*win).hMainWnd,
                        HWND_TOP,
                        pt.x as ::core::ffi::c_int,
                        pt.y as ::core::ffi::c_int,
                        sz.cx as ::core::ffi::c_int,
                        sz.cy as ::core::ffi::c_int,
                        flags as UINT,
                    );
                }

                if !(*wpage).page.is_null()
                    && !(*win).page.is_null()
                    && (*(*wpage).page).file != (*(*win).page).file
                {
                    WINHELP_DeleteBackSet(win);
                }
                WINHELP_InitFonts((*win).hMainWnd);

                (*win).page = (*wpage).page;
                (*win).info = (*wpage).wininfo;
                hTextWnd = GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32));
                WINHELP_SetupText(hTextWnd, win, (*wpage).relative);

                InvalidateRect(
                    (*win).hMainWnd,
                    ::core::ptr::null::<RECT>(),
                    TRUE as BOOL,
                );
                if !(*win).hHistoryWnd.is_null() {
                    InvalidateRect(
                        (*win).hHistoryWnd,
                        ::core::ptr::null::<RECT>(),
                        TRUE as BOOL,
                    );
                }

                break;
            } else {
                win = (*win).next as *mut WINHELP_WINDOW;
            }
        }
    }

    if win.is_null() {
        /* Initialize WINHELP_WINDOW struct */
        win = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            ::core::mem::size_of::<WINHELP_WINDOW>() as SIZE_T,
        ) as *mut WINHELP_WINDOW;
        if win.is_null() {
            return FALSE as BOOL;
        }
        (*win).next = Globals.win_list as *mut tagWinHelp;
        Globals.win_list = win;

        (*win).hHandCur = LoadCursorW(
            ::core::ptr::null_mut::<HINSTANCE__>(),
            IDC_HAND as LPWSTR as LPCWSTR,
        );
        (*win).back.index = (0) as ::core::ffi::c_uint;
        (*win).font_scale = (1) as ::core::ffi::c_uint;
        WINHELP_GrabWindow(win);
    }
    (*win).page = (*wpage).page;
    (*win).info = (*wpage).wininfo;
    WINHELP_GrabWindow(win);

    if bPopup == 0 && !(*wpage).page.is_null() && remember != 0 {
        WINHELP_RememberPage(win, wpage);
    }

    if bPopup != 0 {
        Globals.active_popup = win;
    } else {
        Globals.active_win = win;
    }

    /* Initialize default pushbuttons */
    if bPrimary != 0 && !(*wpage).page.is_null() {
        let mut buffer: [CHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];

        LoadStringA(
            Globals.hInstance as HINSTANCE,
            STID_CONTENTS as UINT,
            &raw mut buffer as LPSTR,
            ::core::mem::size_of::<[CHAR; (MAX_STRING_LEN as _)]>() as ::core::ffi::c_int,
        );
        MACRO_CreateButton(
            b"BTN_CONTENTS\0" as *const u8 as LPCSTR,
            &raw mut buffer as *mut CHAR as LPCSTR,
            b"Contents()\0" as *const u8 as LPCSTR,
        );
        LoadStringA(
            Globals.hInstance as HINSTANCE,
            STID_INDEX as UINT,
            &raw mut buffer as LPSTR,
            ::core::mem::size_of::<[CHAR; (MAX_STRING_LEN as _)]>() as ::core::ffi::c_int,
        );
        MACRO_CreateButton(
            b"BTN_INDEX\0" as *const u8 as LPCSTR,
            &raw mut buffer as *mut CHAR as LPCSTR,
            b"Finder()\0" as *const u8 as LPCSTR,
        );
        LoadStringA(
            Globals.hInstance as HINSTANCE,
            STID_BACK as UINT,
            &raw mut buffer as LPSTR,
            ::core::mem::size_of::<[CHAR; (MAX_STRING_LEN as _)]>() as ::core::ffi::c_int,
        );
        MACRO_CreateButton(
            b"BTN_BACK\0" as *const u8 as LPCSTR,
            &raw mut buffer as *mut CHAR as LPCSTR,
            b"Back()\0" as *const u8 as LPCSTR,
        );
        if (*win).back.index <= (1) as ::core::ffi::c_uint {
            MACRO_DisableButton(b"BTN_BACK\0" as *const u8 as LPCSTR);
        }
    }

    if bReUsed == 0 {
        (*win).hMainWnd = CreateWindowExA(
            (if bPopup != 0 {
                WS_EX_TOOLWINDOW
            } else {
                0
            }) as DWORD,
            (MAIN_WIN_CLASS_NAME).as_ptr() as LPCSTR,
            WINHELP_GetCaption(wpage) as LPCSTR,
            if bPrimary != 0 {
                (WS_OVERLAPPEDWINDOW) as DWORD
            } else {
                (*(*wpage).wininfo).win_style
            },
            (*(*wpage).wininfo).origin.x as ::core::ffi::c_int,
            (*(*wpage).wininfo).origin.y as ::core::ffi::c_int,
            (*(*wpage).wininfo).size.cx as ::core::ffi::c_int,
            (*(*wpage).wininfo).size.cy as ::core::ffi::c_int,
            if bPopup != 0 {
                (*Globals.active_win).hMainWnd
            } else {
                ::core::ptr::null_mut::<HWND__>()
            },
            if bPrimary != 0 {
                LoadMenuW(
                    Globals.hInstance as HINSTANCE,
                    MAKEINTRESOURCEW!(MAIN_MENU) as LPCWSTR,
                )
            } else {
                0 as HMENU
            },
            Globals.hInstance as HINSTANCE,
            win as LPVOID,
        );
        if bPopup == 0 {
            CreateWindowA!(
                (BUTTON_BOX_WIN_CLASS_NAME).as_ptr() as LPCSTR,
                b"\0" as *const u8 as LPCSTR,
                ((WS_CHILD) | (WS_VISIBLE)) as DWORD,
                (0) as ::core::ffi::c_int,
                (0) as ::core::ffi::c_int,
                (0) as ::core::ffi::c_int,
                (0) as ::core::ffi::c_int,
                (*win).hMainWnd,
                (CTL_ID_BUTTON) as HMENU,
                Globals.hInstance as HINSTANCE,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );
        }

        hTextWnd = CreateWindowA!(
            RICHEDIT_CLASS20A.as_ptr() as LPCSTR,
            ::core::ptr::null::<CHAR>(),
            ((ES_MULTILINE)
                | (ES_READONLY)
                | (WS_CHILD)
                | (WS_HSCROLL)
                | (WS_VSCROLL)
                | (WS_VISIBLE)) as DWORD,
            (0) as ::core::ffi::c_int,
            (0) as ::core::ffi::c_int,
            (0) as ::core::ffi::c_int,
            (0) as ::core::ffi::c_int,
            (*win).hMainWnd,
            (CTL_ID_TEXT) as HMENU,
            Globals.hInstance as HINSTANCE,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
        SendMessageW(
            hTextWnd,
            EM_SETEVENTMASK as UINT,
            0 as WPARAM,
            SendMessageW(
                hTextWnd,
                EM_GETEVENTMASK as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as LPARAM
                | (ENM_MOUSEEVENTS) as LPARAM,
        );
        (*win).origRicheditWndProc =
            ::core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrA(
                hTextWnd,
                GWLP_WNDPROC,
                (::core::mem::transmute::<WNDPROC, LONG_PTR>(Some(WINHELP_RicheditWndProc))) as _,
            ));
    }

    hIcon = if !(*wpage).page.is_null() {
        (*(*(*wpage).page).file).hIcon
    } else {
        ::core::ptr::null_mut::<HICON__>()
    };
    if hIcon.is_null() {
        hIcon = LoadImageW(
            Globals.hInstance as HINSTANCE,
            MAKEINTRESOURCEW!(IDI_WINHELP) as LPCWSTR,
            IMAGE_ICON as UINT,
            GetSystemMetrics((SM_CXSMICON as i32)) as ::core::ffi::c_int,
            GetSystemMetrics((SM_CYSMICON as i32)) as ::core::ffi::c_int,
            LR_SHARED as UINT,
        ) as HICON;
    }
    SendMessageW(
        (*win).hMainWnd,
        WM_SETICON as UINT,
        ICON_SMALL as WPARAM,
        hIcon as DWORD_PTR as LPARAM,
    );

    /* Initialize file specific pushbuttons */
    if (*(*wpage).wininfo).win_style & (WS_POPUP) as DWORD == 0
        && !(*wpage).page.is_null()
    {
        let mut macro_0: *mut HLPFILE_MACRO = ::core::ptr::null_mut::<HLPFILE_MACRO>();
        macro_0 = (*(*(*wpage).page).file).first_macro;
        while !macro_0.is_null() {
            MACRO_ExecuteMacro(win, (*macro_0).lpszMacro);
            macro_0 = (*macro_0).next as *mut HLPFILE_MACRO;
        }

        macro_0 = (*(*wpage).page).first_macro;
        while !macro_0.is_null() {
            MACRO_ExecuteMacro(win, (*macro_0).lpszMacro);
            macro_0 = (*macro_0).next as *mut HLPFILE_MACRO;
        }
    }
    /* See #17681, in some cases, the newly created window is closed by the macros it contains
     * (braindead), so deal with this case
     */
    win = Globals.win_list;
    while !win.is_null() {
        if lstrcmpiA(
            &raw mut (*(*win).info).name as *mut ::core::ffi::c_char as LPCSTR,
            &raw mut (*(*wpage).wininfo).name as *mut ::core::ffi::c_char as LPCSTR,
        ) == 0
        {
            break;
        }
        win = (*win).next as *mut WINHELP_WINDOW;
    }
    if win.is_null() || WINHELP_ReleaseWindow(win) == 0 {
        return TRUE as BOOL;
    }

    if bPopup != 0 {
        let mut mask: DWORD = SendMessageW(
            hTextWnd,
            EM_GETEVENTMASK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as DWORD;

        (*win).font_scale = (*Globals.active_win).font_scale;
        WINHELP_SetupText(hTextWnd, win, (*wpage).relative);

        /* we need the window to be shown for richedit to compute the size */
        ShowWindow((*win).hMainWnd, nCmdShow);
        SendMessageW(
            hTextWnd,
            EM_SETEVENTMASK as UINT,
            0 as WPARAM,
            (mask | (ENM_REQUESTRESIZE) as DWORD) as LPARAM,
        );
        SendMessageW(
            hTextWnd,
            EM_REQUESTRESIZE as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            hTextWnd,
            EM_SETEVENTMASK as UINT,
            0 as WPARAM,
            mask as LPARAM,
        );
    } else {
        WINHELP_SetupText(hTextWnd, win, (*wpage).relative);
        WINHELP_LayoutMainWindow(win);
        ShowWindow((*win).hMainWnd, nCmdShow);
    }

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_OpenHelpWindow(lookup: Option<unsafe extern "C" fn(*mut HLPFILE, LONG, *mut ULONG) -> *mut HLPFILE_PAGE>, hlpfile: *mut HLPFILE, val: LONG, wi: *mut HLPFILE_WINDOWINFO, nCmdShow: ::core::ffi::c_int) -> BOOL {
    let mut wpage: WINHELP_WNDPAGE = WINHELP_WNDPAGE {
        page: core::ptr::null_mut(),
        wininfo: core::ptr::null_mut(),
        relative: 0,
    };

    wpage.page = (lookup.unwrap_unchecked())(hlpfile, val, core::ptr::addr_of_mut!(wpage.relative));
    if !wpage.page.is_null() {
        (*(*wpage.page).file).wRefCount = (*(*wpage.page).file).wRefCount.wrapping_add(1);
    }
    wpage.wininfo = wi;
    return WINHELP_CreateHelpWindow(core::ptr::addr_of_mut!(wpage), nCmdShow, (TRUE) as BOOL);
}

unsafe extern "C" fn WINHELP_HandleTextMouse(
    mut win: *mut WINHELP_WINDOW,
    mut msg: UINT,
    mut lParam: LPARAM,
) -> BOOL {
    let mut hlpfile: *mut HLPFILE = ::core::ptr::null_mut::<HLPFILE>();
    let mut link: *mut HLPFILE_LINK = ::core::ptr::null_mut::<HLPFILE_LINK>();
    let mut ret: BOOL = FALSE as BOOL;
    match msg {
        WM_LBUTTONDOWN => {
            link = WINHELP_FindLink(win, lParam);
            if !link.is_null() {
                let mut wi: *mut HLPFILE_WINDOWINFO = ::core::ptr::null_mut::<HLPFILE_WINDOWINFO>();
                match (*link).cookie as ::core::ffi::c_uint {
                    0 => {
                        hlpfile = WINHELP_LookupHelpFile((*link).string);
                        if !hlpfile.is_null() {
                            let mut skip_open: bool = false;
                            if (*link).window
                                == -(1 as ::core::ffi::c_int) as ::core::ffi::c_uint
                            {
                                wi = (*win).info;
                                if (*wi).win_style & WS_POPUP as DWORD != 0 {
                                    wi = (*Globals.active_win).info;
                                }
                            } else if (*link).window < (*hlpfile).numWindows {
                                wi = (*hlpfile).windows.offset((*link).window as isize)
                                    as *mut HLPFILE_WINDOWINFO;
                            } else {
                                WINE_WARN!(
                                    "link to window %d/%d\n",
                                    (*link).window,
                                    (*hlpfile).numWindows,
                                );
                                skip_open = true;
                            }
                            if !skip_open {
                                WINHELP_OpenHelpWindow(
                                    Some(
                                        HLPFILE_PageByHash
                                            as unsafe extern "C" fn(
                                                *mut HLPFILE,
                                                LONG,
                                                *mut ULONG,
                                            )
                                                -> *mut HLPFILE_PAGE,
                                    ),
                                    hlpfile,
                                    (*link).hash,
                                    wi,
                                    (SW_NORMAL as i32),
                                );
                            }
                        }
                    }
                    1 => {
                        hlpfile = WINHELP_LookupHelpFile((*link).string);
                        if !hlpfile.is_null() {
                            WINHELP_OpenHelpWindow(
                                Some(
                                    HLPFILE_PageByHash
                                        as unsafe extern "C" fn(
                                            *mut HLPFILE,
                                            LONG,
                                            *mut ULONG,
                                        )
                                            -> *mut HLPFILE_PAGE,
                                ),
                                hlpfile,
                                (*link).hash,
                                WINHELP_GetPopupWindowInfo(hlpfile, win, lParam),
                                (SW_NORMAL as i32),
                            );
                        }
                    }
                    2 => {
                        MACRO_ExecuteMacro(win, (*link).string);
                    }
                    _ => {
                        WINE_FIXME!(
                            "Unknown link cookie %d\n",
                            (*link).cookie as ::core::ffi::c_uint,
                        );
                    }
                }
                ret = TRUE as BOOL;
            }
        }
        _ => {}
    }
    return ret;
}

unsafe extern "C" fn WINHELP_CheckPopup(
    mut hWnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    mut lret: *mut LRESULT,
) -> BOOL {
    let mut popup: *mut WINHELP_WINDOW = core::ptr::null_mut();

    if Globals.active_popup.is_null() {
        return (FALSE) as BOOL;
    }

    let mut at: u8 = if msg == WM_NOTIFY {
        0
    } else if msg == WM_ACTIVATE {
        1
    } else if msg == WM_LBUTTONDOWN {
        2
    } else if msg == WM_MBUTTONDOWN
        || msg == WM_RBUTTONDOWN
        || msg == WM_NCLBUTTONDOWN
        || msg == WM_NCMBUTTONDOWN
        || msg == WM_NCRBUTTONDOWN
    {
        3
    } else {
        4
    };

    loop {
        match at {
            0 => {
                let mut msgf: *mut MSGFILTER = lParam as *mut MSGFILTER;
                if (*msgf).nmhdr.code == EN_MSGFILTER as UINT {
                    if WINHELP_CheckPopup(
                        hWnd,
                        (*msgf).msg,
                        (*msgf).wParam,
                        (*msgf).lParam,
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        return (FALSE) as BOOL;
                    }
                    if !lret.is_null() {
                        *lret = 1 as LRESULT;
                    }
                    return (TRUE) as BOOL;
                }
                break;
            }
            1 => {
                if LOWORD!(wParam) as UINT != WA_INACTIVE
                    || lParam as HWND == (*Globals.active_win).hMainWnd
                    || lParam as HWND == (*Globals.active_popup).hMainWnd
                    || GetWindow(lParam as HWND, GW_OWNER as UINT)
                        == (*Globals.active_win).hMainWnd
                {
                    break;
                }
                at = 2;
                continue;
            }
            2 => {
                if msg == WM_LBUTTONDOWN as UINT {
                    WINHELP_HandleTextMouse(Globals.active_popup, msg, lParam);
                }
                at = 3;
                continue;
            }
            3 => {
                popup = Globals.active_popup;
                Globals.active_popup = core::ptr::null_mut();
                WINHELP_ReleaseWindow(popup);
                if !lret.is_null() {
                    *lret = 1 as LRESULT;
                }
                return (TRUE) as BOOL;
            }
            _ => break,
        }
    }

    return (FALSE) as BOOL;
}

unsafe extern "system" fn WINHELP_ButtonWndProc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if WINHELP_CheckPopup(hWnd, msg, wParam, lParam, core::ptr::null_mut()) != 0 {
        return 0 as LRESULT;
    }

    if msg == WM_KEYDOWN as UINT {
        match wParam {
            w if w == VK_UP as WPARAM
                || w == VK_DOWN as WPARAM
                || w == VK_PRIOR as WPARAM
                || w == VK_NEXT as WPARAM
                || w == VK_ESCAPE as WPARAM =>
            {
                return SendMessageA(GetParent(hWnd), msg, wParam, lParam);
            }
            _ => {}
        }
    }

    return CallWindowProcA(Globals.button_proc, hWnd, msg, wParam, lParam);
}
unsafe extern "system" fn WINHELP_ButtonBoxWndProc(
    mut hWnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut winpos: *mut WINDOWPOS = core::ptr::null_mut::<WINDOWPOS>();
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut::<WINHELP_WINDOW>();
    let mut button: *mut WINHELP_BUTTON = core::ptr::null_mut::<WINHELP_BUTTON>();
    let mut button_size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut x: INT = 0;
    let mut y: INT = 0;

    if WINHELP_CheckPopup(hWnd, msg, wParam, lParam, core::ptr::null_mut::<LRESULT>()) != 0 {
        return 0 as LRESULT;
    }

    match msg {
        WM_WINDOWPOSCHANGING => {
            winpos = lParam as *mut WINDOWPOS;
            win = GetWindowLongPtrW(GetParent(hWnd), 0) as *mut WINHELP_WINDOW;

            /* Update buttons */
            button_size.cx = 0 as LONG;
            button_size.cy = 0 as LONG;
            button = (*win).first_button;
            while !button.is_null() {
                let mut hDc: HDC = core::ptr::null_mut::<HDC__>();
                let mut textsize: SIZE = SIZE { cx: 0, cy: 0 };
                if (*button).hWnd.is_null() {
                    (*button).hWnd = CreateWindowA!(
                        (STRING_BUTTON).as_ptr() as LPCSTR,
                        (*button).lpszName,
                        (WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON) as DWORD,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        hWnd,
                        (*button).wParam as HMENU,
                        Globals.hInstance as HINSTANCE,
                        0 as LPVOID,
                    );
                    if !(*button).hWnd.is_null() {
                        if Globals.button_proc.is_none() {
                            let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed::<NONCLIENTMETRICSW>();
                            Globals.button_proc = core::mem::transmute::<LONG_PTR, WNDPROC>(
                                GetWindowLongPtrA((*button).hWnd, GWLP_WNDPROC) as LONG_PTR,
                            );

                            ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
                            SystemParametersInfoW(
                                SPI_GETNONCLIENTMETRICS as UINT,
                                core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
                                &raw mut ncm as PVOID,
                                0 as UINT,
                            );
                            Globals.hButtonFont = CreateFontIndirectW(&raw mut ncm.lfMenuFont);
                        }
                        SetWindowLongPtrA(
                            (*button).hWnd,
                            GWLP_WNDPROC,
                            // C: (LONG_PTR) WINHELP_ButtonWndProc -- the translation had stored
                            // the GWLP_WNDPROC index itself as the new window procedure.
                            ::core::mem::transmute::<WNDPROC, LONG_PTR>(Some(WINHELP_ButtonWndProc)) as _,
                        );
                        if !Globals.hButtonFont.is_null() {
                            SendMessageW(
                                (*button).hWnd,
                                WM_SETFONT as UINT,
                                Globals.hButtonFont as WPARAM,
                                TRUE as LPARAM,
                            );
                        }
                    }
                }
                hDc = GetDC((*button).hWnd);
                GetTextExtentPointA(
                    hDc,
                    (*button).lpszName,
                    strlen((*button).lpszName as *const ::core::ffi::c_char) as ::core::ffi::c_int,
                    &raw mut textsize,
                );
                ReleaseDC((*button).hWnd, hDc);

                button_size.cx = max!(button_size.cx, textsize.cx + BUTTON_CX as LONG);
                button_size.cy = max!(button_size.cy, textsize.cy + BUTTON_CY as LONG);
                button = (*button).next as *mut WINHELP_BUTTON;
            }

            x = 0 as INT;
            y = 0 as INT;
            button = (*win).first_button;
            while !button.is_null() {
                SetWindowPos(
                    (*button).hWnd,
                    HWND_TOP,
                    x as ::core::ffi::c_int,
                    y as ::core::ffi::c_int,
                    button_size.cx as ::core::ffi::c_int,
                    button_size.cy as ::core::ffi::c_int,
                    0 as UINT,
                );

                if x as LONG + 2 as LONG * button_size.cx <= (*winpos).cx as LONG {
                    x = (x as LONG + button_size.cx) as INT;
                } else {
                    x = 0 as INT;
                    y = (y as LONG + button_size.cy) as INT;
                }
                button = (*button).next as *mut WINHELP_BUTTON;
            }
            (*winpos).cy = (y as LONG
                + (if x != 0 {
                    button_size.cy
                } else {
                    0 as LONG
                })) as ::core::ffi::c_int;
        }
        WM_COMMAND => {
            SendMessageW(GetParent(hWnd), msg, wParam, lParam);
        }
        WM_KEYDOWN => {
            match wParam {
                __c if __c == VK_UP as _ || __c == VK_DOWN as _ || __c == VK_PRIOR as _ || __c == VK_NEXT as _ || __c == VK_ESCAPE as _ => {
                    return SendMessageA(GetParent(hWnd), msg, wParam, lParam);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return DefWindowProcA(hWnd, msg, wParam, lParam);
}

unsafe extern "system" fn WINHELP_HistoryWndProc(
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hDc: HDC = core::ptr::null_mut();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut i: ::core::ffi::c_uint = 0;
    let mut r: RECT = core::mem::zeroed();

    match msg {
        WM_NCCREATE => {
            win = (*(lParam as LPCREATESTRUCTA)).lpCreateParams as *mut WINHELP_WINDOW;
            SetWindowLongPtrW(hWnd, 0, (win as ULONG_PTR as LONG_PTR) as _);
            (*win).hHistoryWnd = hWnd;
        }
        WM_CREATE => {
            hDc = GetDC(hWnd);
            GetTextMetricsW(hDc, &raw mut tm);
            GetWindowRect(hWnd, &raw mut r);

            r.right = r.left + 30 * tm.tmAveCharWidth;
            r.bottom = (r.top as usize).wrapping_add(
                (::core::mem::size_of::<[WINHELP_WNDPAGE; 40]>() as usize)
                    .wrapping_div(::core::mem::size_of::<WINHELP_WNDPAGE>() as usize)
                    .wrapping_mul(tm.tmHeight as usize),
            ) as LONG;
            AdjustWindowRect(
                &raw mut r,
                GetWindowLongW(hWnd, GWL_STYLE) as DWORD,
                (FALSE) as BOOL,
            );
            if r.left < 0 {
                r.right -= r.left;
                r.left = 0;
            }
            if r.top < 0 {
                r.bottom -= r.top;
                r.top = 0;
            }

            MoveWindow(
                hWnd,
                r.left as ::core::ffi::c_int,
                r.top as ::core::ffi::c_int,
                r.right as ::core::ffi::c_int,
                r.bottom as ::core::ffi::c_int,
                (TRUE) as BOOL,
            );
            ReleaseDC(hWnd, hDc);
        }
        WM_LBUTTONDOWN => {
            hDc = GetDC(hWnd);
            GetTextMetricsW(hDc, &raw mut tm);
            i = (HIWORD!(lParam) as LONG / tm.tmHeight) as ::core::ffi::c_uint;
            if i < Globals.history.index {
                WINHELP_CreateHelpWindow(
                    (&raw mut Globals.history.set as *mut WINHELP_WNDPAGE).offset(i as isize)
                        as *mut WINHELP_WNDPAGE,
                    (SW_SHOW as i32),
                    (TRUE) as BOOL,
                );
            }
            ReleaseDC(hWnd, hDc);
        }
        WM_PAINT => {
            hDc = BeginPaint(hWnd, &raw mut ps);
            GetTextMetricsW(hDc, &raw mut tm);

            i = 0;
            while i < Globals.history.index {
                if (*Globals.history.set[i as usize].page).file
                    == (*(*Globals.active_win).page).file
                {
                    TextOutA(
                        hDc,
                        0,
                        (i as ::core::ffi::c_ulong)
                            .wrapping_mul(tm.tmHeight as ::core::ffi::c_ulong)
                            as ::core::ffi::c_int,
                        (*Globals.history.set[i as usize].page).lpszTitle as LPCSTR,
                        strlen(
                            (*Globals.history.set[i as usize].page).lpszTitle
                                as *const ::core::ffi::c_char,
                        ) as ::core::ffi::c_int,
                    );
                } else {
                    let mut buffer: [::core::ffi::c_char; 1024] =
                        [0; 1024];
                    let mut ptr1: *const ::core::ffi::c_char =
                        ::core::ptr::null::<::core::ffi::c_char>();
                    let mut ptr2: *const ::core::ffi::c_char =
                        ::core::ptr::null::<::core::ffi::c_char>();
                    let mut len: ::core::ffi::c_uint = 0;

                    ptr1 = strrchr(
                        ((*(*Globals.history.set[i as usize].page).file).lpszPath
                            as *const ::core::ffi::c_char).cast(),
                        '\\' as i32,
                    ).cast();
                    if ptr1.is_null() {
                        ptr1 = (*(*Globals.history.set[i as usize].page).file).lpszPath
                            as *const ::core::ffi::c_char;
                    } else {
                        ptr1 = ptr1.offset(1);
                    }
                    ptr2 = strrchr((ptr1).cast(), '.' as i32).cast();
                    len = (if !ptr2.is_null() {
                        ptr2.offset_from(ptr1) as ::core::ffi::c_longlong as size_t
                    } else {
                        strlen(ptr1)
                    }) as ::core::ffi::c_uint;
                    if len as usize
                        > ::core::mem::size_of::<[::core::ffi::c_char; 1024]>()
                            as usize
                    {
                        len = ::core::mem::size_of::<[::core::ffi::c_char; 1024]>()
                            as ::core::ffi::c_uint;
                    }
                    memcpy(
                        &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        ptr1 as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                    if (len as usize)
                        < ::core::mem::size_of::<[::core::ffi::c_char; 1024]>()
                            as usize
                    {
                        let fresh18 = len;
                        len = len.wrapping_add(1);
                        buffer[fresh18 as usize] = ':' as i32 as ::core::ffi::c_char;
                    }
                    lstrcpynA(
                        (&raw mut buffer as *mut ::core::ffi::c_char).offset(len as isize) as LPSTR,
                        (*Globals.history.set[i as usize].page).lpszTitle as LPCSTR,
                        (::core::mem::size_of::<[::core::ffi::c_char; 1024]>()
                            as usize)
                            .wrapping_sub(len as usize) as ::core::ffi::c_int,
                    );
                    TextOutA(
                        hDc,
                        0,
                        (i as ::core::ffi::c_ulong)
                            .wrapping_mul(tm.tmHeight as ::core::ffi::c_ulong)
                            as ::core::ffi::c_int,
                        &raw mut buffer as *mut ::core::ffi::c_char as LPCSTR,
                        strlen(&raw mut buffer as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int,
                    );
                }
                i = i.wrapping_add(1);
            }
            EndPaint(hWnd, &raw mut ps);
        }
        WM_DESTROY => {
            win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
            if hWnd == (*win).hHistoryWnd {
                (*win).hHistoryWnd = core::ptr::null_mut();
            }
        }
        _ => {}
    }
    return DefWindowProcA(hWnd, msg, wParam, lParam);
}

unsafe extern "C" fn cb_KWBTree(p: *mut ::core::ffi::c_void, next: *mut *mut ::core::ffi::c_void, cookie: *mut ::core::ffi::c_void) {
    let mut hListWnd: HWND = cookie as HWND;
    let mut count: ::core::ffi::c_int = 0;
    WINE_TRACE!(
        b"Adding %s to search list\n\0".as_ptr() as *const ::core::ffi::c_char,
        debugstr_a(p as *mut ::core::ffi::c_char),
    );
    SendMessageA(
        hListWnd,
        LB_INSERTSTRING as UINT,
        (-(1 as ::core::ffi::c_int)) as WPARAM,
        p as LPARAM,
    );
    count = SendMessageW(
        hListWnd,
        LB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    SendMessageW(
        hListWnd,
        LB_SETITEMDATA as UINT,
        (count - 1 as ::core::ffi::c_int) as WPARAM,
        p as LPARAM,
    );
    *next = (p as *mut ::core::ffi::c_char)
        .offset(strlen(p as *mut ::core::ffi::c_char) as isize)
        .offset(7 as isize) as *mut ::core::ffi::c_void;
}

unsafe extern "system" fn WINHELP_IndexDlgProc(
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    static mut id: *mut index_data = core::ptr::null_mut();
    let mut sel: i32 = 0;
    match msg {
        WM_INITDIALOG => {
            id = (*(lParam as *mut PROPSHEETPAGEA)).lParam as *mut index_data;
            HLPFILE_BPTreeEnum(
                (*(*id).hlpfile).kwbtree,
                Some(
                    cb_KWBTree
                        as unsafe extern "C" fn(
                            *mut core::ffi::c_void,
                            *mut *mut core::ffi::c_void,
                            *mut core::ffi::c_void,
                        ) -> (),
                ),
                GetDlgItem(hWnd, IDC_INDEXLIST as i32) as *mut core::ffi::c_void,
            );
            (*id).jump = FALSE as BOOL;
            (*id).offset = 1 as ULONG;
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (HIWORD!(wParam) as i32) as u32 {
                LBN_DBLCLK => {
                    if (LOWORD!(wParam) as i32) == IDC_INDEXLIST as i32 {
                        SendMessageW(
                            GetParent(hWnd),
                            PSM_PRESSBUTTON as UINT,
                            PSBTN_OK as WPARAM,
                            0 as LPARAM,
                        );
                    }
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            match ((*(lParam as *mut NMHDR)).code) as i32 {
                PSN_APPLY => {
                    sel = SendDlgItemMessageW(
                        hWnd,
                        (IDC_INDEXLIST as UINT) as i32,
                        LB_GETCURSEL as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as i32;
                    if sel != LB_ERR as i32 {
                        let mut p: *mut BYTE = core::ptr::null_mut();
                        let mut count: i32 = 0;

                        p = SendDlgItemMessageW(
                            hWnd,
                            (IDC_INDEXLIST as UINT) as i32,
                            LB_GETITEMDATA as UINT,
                            sel as WPARAM,
                            0 as LPARAM,
                        ) as *mut BYTE;
                        // Item data is a packed keyword record: read its fields unaligned.
                        count = ((p as *mut core::ffi::c_char)
                            .offset(strlen(p as *mut core::ffi::c_char) as isize)
                            .offset(1 as isize)
                            as *mut core::ffi::c_short).read_unaligned() as i32;
                        if count > 1 as i32 {
                            MessageBoxA(
                                hWnd,
                                b"count > 1 not supported yet\0" as *const u8 as LPCSTR,
                                b"Error\0" as *const u8 as LPCSTR,
                                (MB_OK | MB_ICONSTOP) as UINT,
                            );
                            SetWindowLongPtrW(
                                hWnd,
                                DWLP_MSGRESULT as i32,
                                (PSNRET_INVALID as LONG_PTR) as _,
                            );
                            return TRUE as INT_PTR;
                        }
                        (*id).offset = ((p as *mut core::ffi::c_char)
                            .offset(strlen(p as *mut core::ffi::c_char) as isize)
                            .offset(3 as isize)
                            as *mut ULONG).read_unaligned();
                        (*id).offset = ((*(*id).hlpfile)
                            .kwdata
                            .offset((*id).offset as isize)
                            .offset(9 as isize)
                            as *mut core::ffi::c_long).read_unaligned() as ULONG;
                        if (*id).offset == 0xFFFFFFFF as ULONG {
                            MessageBoxA(
                                hWnd,
                                b"macro keywords not supported yet\0" as *const u8 as LPCSTR,
                                b"Error\0" as *const u8 as LPCSTR,
                                (MB_OK | MB_ICONSTOP) as UINT,
                            );
                            SetWindowLongPtrW(
                                hWnd,
                                DWLP_MSGRESULT as i32,
                                (PSNRET_INVALID as LONG_PTR) as _,
                            );
                            return TRUE as INT_PTR;
                        }
                        (*id).jump = TRUE as BOOL;
                        SetWindowLongPtrW(
                            hWnd,
                            DWLP_MSGRESULT as i32,
                            (PSNRET_NOERROR as LONG_PTR) as _,
                        );
                    }
                    return TRUE as INT_PTR;
                }
                _ => {
                    return FALSE as INT_PTR;
                }
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn WINHELP_SearchDlgProc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match msg {
        WM_INITDIALOG => return TRUE as INT_PTR,
        WM_NOTIFY => {
            match ((*(lParam as *mut NMHDR)).code) as i32 {
                PSN_APPLY => {
                    SetWindowLongPtrW(hWnd, DWLP_MSGRESULT, (PSNRET_NOERROR as LONG_PTR) as _);
                    return TRUE as INT_PTR;
                }
                _ => return FALSE as INT_PTR,
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
unsafe extern "system" fn WINHELP_MainWndProc(
    mut hWnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut win: *mut WINHELP_WINDOW = core::ptr::null_mut();
    let mut button: *mut WINHELP_BUTTON = core::ptr::null_mut();
    let mut hTextWnd: HWND = core::ptr::null_mut();
    let mut ret: LRESULT = 0;

    if WINHELP_CheckPopup(hWnd, msg, wParam, lParam, &raw mut ret) != 0 {
        return ret;
    }

    match msg {
        WM_NCCREATE => {
            win = (*(lParam as LPCREATESTRUCTA)).lpCreateParams as *mut WINHELP_WINDOW;
            SetWindowLongPtrW(hWnd, 0, (win as ULONG_PTR) as isize);
            if (*win).page.is_null() && Globals.isBook != 0 {
                PostMessageW(
                    hWnd,
                    WM_COMMAND,
                    MNID_FILE_OPEN as WPARAM,
                    0 as LPARAM,
                );
            }
            (*win).hMainWnd = hWnd;
        }

        WM_WINDOWPOSCHANGED => {
            WINHELP_LayoutMainWindow(
                GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW,
            );
        }

        WM_COMMAND => {
            win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
            match (LOWORD!(wParam)) as u32 {
                /* Menu FILE */
                MNID_FILE_OPEN => {
                    MACRO_FileOpen();
                }
                MNID_FILE_PRINT => {
                    MACRO_Print();
                }
                MNID_FILE_SETUP => {
                    MACRO_PrinterSetup();
                }
                MNID_FILE_EXIT => {
                    MACRO_Exit();
                }

                /* Menu EDIT */
                MNID_EDIT_COPYDLG => {
                    SendDlgItemMessageW(
                        hWnd,
                        (CTL_ID_TEXT as i32),
                        WM_COPY,
                        0,
                        0,
                    );
                }
                MNID_EDIT_ANNOTATE => {
                    MACRO_Annotate();
                }

                /* Menu Bookmark */
                MNID_BKMK_DEFINE => {
                    MACRO_BookmarkDefine();
                }

                /* Menu Help */
                MNID_HELP_HELPON => {
                    MACRO_HelpOn();
                }
                MNID_HELP_HELPTOP => {
                    MACRO_HelpOnTop();
                }
                MNID_HELP_ABOUT => {
                    MACRO_About();
                }

                /* Context help */
                MNID_CTXT_ANNOTATE => {
                    MACRO_Annotate();
                }
                MNID_CTXT_COPY => {
                    MACRO_CopyDialog();
                }
                MNID_CTXT_PRINT => {
                    MACRO_Print();
                }
                MNID_OPTS_HISTORY => {
                    MACRO_History();
                }
                MNID_OPTS_FONTS_SMALL | MNID_CTXT_FONTS_SMALL => {
                    win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
                    if (*win).font_scale != 0 {
                        (*win).font_scale = 0;
                        WINHELP_SetupText(
                            GetDlgItem(hWnd, (CTL_ID_TEXT as i32)),
                            win,
                            0, /* FIXME */
                        );
                    }
                }
                MNID_OPTS_FONTS_NORMAL | MNID_CTXT_FONTS_NORMAL => {
                    win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
                    if (*win).font_scale != 1 {
                        (*win).font_scale = 1;
                        WINHELP_SetupText(
                            GetDlgItem(hWnd, (CTL_ID_TEXT as i32)),
                            win,
                            0, /* FIXME */
                        );
                    }
                }
                MNID_OPTS_FONTS_LARGE | MNID_CTXT_FONTS_LARGE => {
                    win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
                    if (*win).font_scale != 2 {
                        (*win).font_scale = 2;
                        WINHELP_SetupText(
                            GetDlgItem(hWnd, (CTL_ID_TEXT as i32)),
                            win,
                            0, /* FIXME */
                        );
                    }
                }

                _ => {
                    /* Buttons */
                    button = (*win).first_button;
                    while !button.is_null() {
                        if wParam == (*button).wParam {
                            break;
                        }
                        button = (*button).next as *mut WINHELP_BUTTON;
                    }
                    if !button.is_null() {
                        MACRO_ExecuteMacro(win, (*button).lpszMacro);
                    } else if HIWORD!(wParam) == 0 {
                        MessageBoxW(
                            core::ptr::null_mut(),
                            MAKEINTRESOURCEW!(STID_NOT_IMPLEMENTED),
                            MAKEINTRESOURCEW!(STID_WHERROR),
                            MB_OK,
                        );
                    }
                }
            }
        }
        /* EPP     case WM_DESTROY: */
        /* EPP         if (Globals.hPopupWnd) DestroyWindow(Globals.hPopupWnd); */
        /* EPP         break; */
        WM_COPYDATA => {
            return WINHELP_HandleCommand(wParam as HWND, lParam);
        }

        WM_CHAR => {
            if wParam == 3 {
                SendDlgItemMessageW(
                    hWnd,
                    (CTL_ID_TEXT as i32),
                    WM_COPY,
                    0,
                    0,
                );
                return 0;
            }
        }

        WM_KEYDOWN => {
            win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
            hTextWnd = GetDlgItem((*win).hMainWnd, (CTL_ID_TEXT as i32));

            match wParam {
                __c if __c == VK_UP as _ => {
                    SendMessageW(
                        hTextWnd,
                        EM_SCROLL,
                        (SB_LINEUP) as WPARAM,
                        0,
                    );
                    return 0;
                }
                __c if __c == VK_DOWN as _ => {
                    SendMessageW(
                        hTextWnd,
                        EM_SCROLL,
                        (SB_LINEDOWN) as WPARAM,
                        0,
                    );
                    return 0;
                }
                __c if __c == VK_PRIOR as _ => {
                    SendMessageW(
                        hTextWnd,
                        EM_SCROLL,
                        (SB_PAGEUP) as WPARAM,
                        0,
                    );
                    return 0;
                }
                __c if __c == VK_NEXT as _ => {
                    SendMessageW(
                        hTextWnd,
                        EM_SCROLL,
                        (SB_PAGEDOWN) as WPARAM,
                        0,
                    );
                    return 0;
                }
                __c if __c == VK_ESCAPE as _ => {
                    MACRO_Exit();
                    return 0;
                }
                _ => {}
            }
        }

        WM_NOTIFY => {
            if wParam == (CTL_ID_TEXT as _) {
                let mut rc: RECT = core::mem::zeroed();

                match (*(lParam as *mut NMHDR)).code {
                    EN_MSGFILTER => {
                        {
                            let msgf: *const MSGFILTER =
                                lParam as *const MSGFILTER;
                            match (*msgf).msg {
                                WM_KEYUP => {
                                    if (*msgf).wParam == (VK_ESCAPE as _) {
                                        WINHELP_ReleaseWindow(
                                            GetWindowLongPtrW(hWnd, 0)
                                                as *mut WINHELP_WINDOW,
                                        );
                                    }
                                }
                                WM_RBUTTONDOWN => {
                                    let mut hMenu: HMENU = core::ptr::null_mut();
                                    let mut pt: POINT = core::mem::zeroed();

                                    win = GetWindowLongPtrW(hWnd, 0)
                                        as *mut WINHELP_WINDOW;
                                    hMenu = LoadMenuW(
                                        (Globals.hInstance).cast(),
                                        MAKEINTRESOURCEW!(CONTEXT_MENU),
                                    );
                                    match (*win).font_scale {
                                        0 => {
                                            CheckMenuItem(
                                                hMenu,
                                                MNID_CTXT_FONTS_SMALL,
                                                MF_BYCOMMAND | MF_CHECKED,
                                            );
                                        }
                                        1 => {
                                            CheckMenuItem(
                                                hMenu,
                                                MNID_CTXT_FONTS_NORMAL,
                                                MF_BYCOMMAND | MF_CHECKED,
                                            );
                                        }
                                        2 => {
                                            CheckMenuItem(
                                                hMenu,
                                                MNID_CTXT_FONTS_LARGE,
                                                MF_BYCOMMAND | MF_CHECKED,
                                            );
                                        }
                                        _ => {
                                            WINE_FIXME!(
                                                b"Unsupported %d\n\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                (*win).font_scale,
                                            );
                                            /* fall through */
                                            CheckMenuItem(
                                                hMenu,
                                                MNID_CTXT_FONTS_NORMAL,
                                                MF_BYCOMMAND | MF_CHECKED,
                                            );
                                        }
                                    }
                                    pt.x = (LOWORD!((*msgf).lParam) as ::core::ffi::c_short)
                                        as ::core::ffi::c_int
                                        as LONG;
                                    pt.y = (HIWORD!((*msgf).lParam) as ::core::ffi::c_short)
                                        as ::core::ffi::c_int
                                        as LONG;
                                    ClientToScreen((*msgf).nmhdr.hwndFrom, &raw mut pt);
                                    TrackPopupMenu(
                                        GetSubMenu(hMenu, 0),
                                        TPM_LEFTALIGN | TPM_TOPALIGN,
                                        pt.x,
                                        pt.y,
                                        0,
                                        hWnd,
                                        core::ptr::null::<RECT>(),
                                    );
                                    DestroyMenu(hMenu);
                                }
                                _ => {
                                    return (WINHELP_HandleTextMouse(
                                        GetWindowLongPtrW(hWnd, 0)
                                            as *mut WINHELP_WINDOW,
                                        (*msgf).msg,
                                        (*msgf).lParam,
                                    )) as LRESULT;
                                }
                            }
                        }
                    }

                    EN_REQUESTRESIZE => {
                        rc = (*(lParam as *mut REQRESIZE)).rc;
                        win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
                        AdjustWindowRect(
                            &raw mut rc,
                            GetWindowLongW((*win).hMainWnd, GWL_STYLE) as DWORD,
                            (FALSE) as BOOL,
                        );
                        SetWindowPos(
                            (*win).hMainWnd,
                            HWND_TOP,
                            0,
                            0,
                            rc.right - rc.left,
                            rc.bottom - rc.top,
                            SWP_NOMOVE | SWP_NOZORDER,
                        );
                        WINHELP_LayoutMainWindow(win);
                    }
                    _ => {}
                }
            }
        }

        WM_INITMENUPOPUP => {
            win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
            CheckMenuItem(
                wParam as HMENU,
                MNID_OPTS_FONTS_SMALL,
                if (*win).font_scale == 0 {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                },
            );
            CheckMenuItem(
                wParam as HMENU,
                MNID_OPTS_FONTS_NORMAL,
                if (*win).font_scale == 1 {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                },
            );
            CheckMenuItem(
                wParam as HMENU,
                MNID_OPTS_FONTS_LARGE,
                if (*win).font_scale == 2 {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                },
            );
        }
        WM_DESTROY => {
            win = GetWindowLongPtrW(hWnd, 0) as *mut WINHELP_WINDOW;
            WINHELP_DeleteWindow(win);
        }
        _ => {}
    }
    return DefWindowProcA(hWnd, msg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn WINHELP_CreateIndexWindow(is_search: BOOL) -> BOOL {
    let mut psPage: [HPROPSHEETPAGE; 3] = core::mem::zeroed();
    let mut psp: PROPSHEETPAGEA = core::mem::zeroed();
    let mut psHead: PROPSHEETHEADERA = core::mem::zeroed();
    let mut id: index_data = core::mem::zeroed();
    let mut buf: [CHAR; 256] = [0 as CHAR; 256];

    if !Globals.active_win.is_null()
        && !(*Globals.active_win).page.is_null()
        && !(*(*Globals.active_win).page).file.is_null()
    {
        id.hlpfile = (*(*Globals.active_win).page).file as *mut HLPFILE;
    } else {
        return (FALSE) as BOOL;
    }

    if (*id.hlpfile).kwbtree.is_null() {
        WINE_TRACE!(b"No index provided\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    InitCommonControls();

    id.jump = (FALSE as i32);
    memset(
        core::ptr::addr_of_mut!(psp) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<PROPSHEETPAGEA>() as size_t,
    );
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGEA>() as DWORD;
    psp.dwFlags = 0 as DWORD;
    psp.hInstance = Globals.hInstance as HINSTANCE;

    psp.u.pszTemplate = MAKEINTRESOURCEA!(IDD_INDEX) as LPCSTR;
    psp.lParam = core::ptr::addr_of_mut!(id) as LPARAM;
    psp.pfnDlgProc = Some(WINHELP_IndexDlgProc);
    psPage[0 as usize] =
        CreatePropertySheetPageA(core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGEA);

    psp.u.pszTemplate = MAKEINTRESOURCEA!(IDD_SEARCH) as LPCSTR;
    psp.lParam = core::ptr::addr_of_mut!(id) as LPARAM;
    psp.pfnDlgProc = Some(WINHELP_SearchDlgProc);
    psPage[1 as usize] =
        CreatePropertySheetPageA(core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGEA);

    memset(
        core::ptr::addr_of_mut!(psHead) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<PROPSHEETHEADERA>() as size_t,
    );
    psHead.dwSize = core::mem::size_of::<PROPSHEETHEADERA>() as DWORD;

    LoadStringA(
        Globals.hInstance as HINSTANCE,
        STID_PSH_INDEX as UINT,
        core::ptr::addr_of_mut!(buf) as *mut CHAR,
        core::mem::size_of::<[CHAR; 256]>() as core::ffi::c_int,
    );
    strcat(
        core::ptr::addr_of_mut!(buf) as *mut CHAR,
        core::ptr::addr_of_mut!((*(*Globals.active_win).info).caption) as *mut CHAR,
    );

    psHead.pszCaption = core::ptr::addr_of_mut!(buf) as *mut CHAR as LPCSTR;
    psHead.nPages = 2 as UINT;
    psHead.u2.nStartPage = (if is_search != 0 {
        1
    } else {
        0
    }) as UINT;
    psHead.hwndParent = (*Globals.active_win).hMainWnd;
    psHead.u3.phpage = core::ptr::addr_of_mut!(psPage) as *mut HPROPSHEETPAGE;
    psHead.dwFlags = PSH_NOAPPLYNOW as DWORD;

    PropertySheetA(core::ptr::addr_of_mut!(psHead) as LPCPROPSHEETHEADERA);
    if id.jump != 0 {
        WINE_TRACE!(
            b"got %d as an offset\n\0".as_ptr() as *const i8,
            id.offset,
        );
        WINHELP_OpenHelpWindow(
            Some(
                HLPFILE_PageByOffset
                    as unsafe extern "C" fn(*mut HLPFILE, LONG, *mut ULONG) -> *mut HLPFILE_PAGE,
            ),
            id.hlpfile,
            id.offset as LONG,
            (*Globals.active_win).info,
            (SW_NORMAL as i32),
        );
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn WINHELP_RegisterWinClasses() -> BOOL {
    let mut class_main: WNDCLASSEXA = core::mem::zeroed::<WNDCLASSEXA>();
    let mut class_button_box: WNDCLASSEXA = core::mem::zeroed::<WNDCLASSEXA>();
    let mut class_history: WNDCLASSEXA = core::mem::zeroed::<WNDCLASSEXA>();

    class_main.cbSize = core::mem::size_of::<WNDCLASSEXA>() as UINT;
    class_main.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    class_main.lpfnWndProc = Some(WINHELP_MainWndProc);
    class_main.cbClsExtra = 0 as ::core::ffi::c_int;
    class_main.cbWndExtra = core::mem::size_of::<*mut WINHELP_WINDOW>() as ::core::ffi::c_int;
    class_main.hInstance = Globals.hInstance as HINSTANCE;
    class_main.hIcon = LoadIconW(
        Globals.hInstance as HINSTANCE,
        MAKEINTRESOURCEW!(IDI_WINHELP),
    );
    class_main.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    );
    class_main.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    class_main.lpszMenuName = core::ptr::null();
    class_main.lpszClassName = (MAIN_WIN_CLASS_NAME).as_ptr();
    class_main.hIconSm = LoadImageW(
        Globals.hInstance as HINSTANCE,
        MAKEINTRESOURCEW!(IDI_WINHELP),
        IMAGE_ICON as UINT,
        GetSystemMetrics((SM_CXSMICON as i32)),
        GetSystemMetrics((SM_CYSMICON as i32)),
        LR_SHARED as UINT,
    ) as HICON;

    class_button_box = class_main;
    class_button_box.lpfnWndProc = Some(WINHELP_ButtonBoxWndProc);
    class_button_box.cbWndExtra = 0 as ::core::ffi::c_int;
    class_button_box.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    class_button_box.lpszClassName = (BUTTON_BOX_WIN_CLASS_NAME).as_ptr();

    class_history = class_main;
    class_history.lpfnWndProc = Some(WINHELP_HistoryWndProc);
    class_history.lpszClassName = (HISTORY_WIN_CLASS_NAME).as_ptr();

    return (RegisterClassExA(core::ptr::addr_of_mut!(class_main)) != 0
        && RegisterClassExA(core::ptr::addr_of_mut!(class_button_box)) != 0
        && RegisterClassExA(core::ptr::addr_of_mut!(class_history)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    mut hInstance: HINSTANCE,
    mut prev: HINSTANCE,
    mut cmdline: LPSTR,
    mut show: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut msg: MSG = core::mem::zeroed();
    let mut lHash: LONG = 0;
    let mut hlpfile: *mut HLPFILE = core::ptr::null_mut();
    static mut default_wndname: [CHAR; 5] = [
        b'm' as CHAR,
        b'a' as CHAR,
        b'i' as CHAR,
        b'n' as CHAR,
        0 as CHAR,
    ];
    let mut wndname: LPSTR = core::ptr::addr_of_mut!(default_wndname) as LPSTR;
    let mut dll: *mut WINHELP_DLL = core::ptr::null_mut();
    let mut hAccel: HACCEL = core::ptr::null_mut();

    Globals.hInstance = hInstance as HANDLE;

    if LoadLibraryA(b"riched20.dll\0".as_ptr() as LPCSTR).is_null() {
        return MessageBoxW(
            core::ptr::null_mut(),
            MAKEINTRESOURCEW!(STID_NO_RICHEDIT),
            MAKEINTRESOURCEW!(STID_WHERROR),
            MB_OK,
        );
    }

    /* Get options */
    while *cmdline as ::core::ffi::c_int != 0
        && (*cmdline as ::core::ffi::c_int == ' ' as i32
            || *cmdline as ::core::ffi::c_int == '-' as i32)
    {
        let mut option: CHAR = 0;
        let mut topic_id: LPCSTR = core::ptr::null();
        let fresh15 = cmdline;
        cmdline = cmdline.add(1);
        if *fresh15 as ::core::ffi::c_int == ' ' as i32 {
            continue;
        }

        option = *cmdline;
        if option != 0 {
            cmdline = cmdline.add(1);
        }
        while *cmdline as ::core::ffi::c_int == ' ' as i32 {
            cmdline = cmdline.add(1);
        }
        match option as ::core::ffi::c_int {
            105 | 73 => {
                // 'i' | 'I'
                topic_id = cmdline as LPCSTR;
                while *cmdline as ::core::ffi::c_int != 0
                    && *cmdline as ::core::ffi::c_int != ' ' as i32
                {
                    cmdline = cmdline.add(1);
                }
                if *cmdline != 0 {
                    let fresh16 = cmdline;
                    cmdline = cmdline.add(1);
                    *fresh16 = '\0' as i32 as CHAR;
                }
                lHash = HLPFILE_Hash(topic_id);
            }
            51 | 52 => {
                // '3' | '4'
                Globals.wVersion = (option as ::core::ffi::c_int - '0' as i32) as UINT;
            }
            120 => {
                // 'x'
                show = (SW_HIDE as i32);
                Globals.isBook = (FALSE as i32);
            }
            _ => {
                WINE_FIXME!(
                    b"Unsupported cmd line: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    debugstr_a(cmdline as *const ::core::ffi::c_char),
                );
            }
        }
    }

    /* Create primary window */
    if WINHELP_RegisterWinClasses() == 0 {
        WINE_FIXME!(
            b"Couldn't register classes\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0;
    }

    if *cmdline != 0 {
        let mut ptr: *mut ::core::ffi::c_char = core::ptr::null_mut();
        if *cmdline as ::core::ffi::c_int == '"' as i32 && {
            ptr = strchr(
                cmdline.add(1 as usize) as *const ::core::ffi::c_char,
                '"' as i32,
            );
            !ptr.is_null()
        } {
            cmdline = cmdline.add(1);
            *ptr = '\0' as i32 as ::core::ffi::c_char;
        }
        ptr = strchr(cmdline as *const ::core::ffi::c_char, '>' as i32);
        if !ptr.is_null() {
            *ptr = '\0' as i32 as ::core::ffi::c_char;
            wndname = ptr.add(1 as usize) as LPSTR;
        }
        hlpfile = WINHELP_LookupHelpFile(cmdline as LPCSTR);
        if hlpfile.is_null() {
            return 0;
        }
    } else {
        hlpfile = core::ptr::null_mut();
    }

    WINHELP_OpenHelpWindow(
        Some(HLPFILE_PageByHash),
        hlpfile,
        lHash,
        WINHELP_GetWindowInfo(hlpfile, wndname),
        show,
    );

    /* Message loop */
    hAccel = LoadAcceleratorsW(hInstance, MAKEINTRESOURCEW!(MAIN_ACCEL));
    while (!Globals.win_list.is_null() || !Globals.active_popup.is_null())
        && GetMessageW(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0,
            0,
        ) != 0
    {
        let mut hWnd: HWND = if !Globals.active_win.is_null() {
            (*Globals.active_win).hMainWnd
        } else {
            core::ptr::null_mut()
        };
        if TranslateAcceleratorW(hWnd, hAccel, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    dll = Globals.dlls;
    while !dll.is_null() {
        if ((*dll).class & DC_INITTERM) != 0 {
            (*dll).handler.unwrap_unchecked()(
                (DW_TERM as u16),
                0,
                0,
            );
        }
        dll = (*dll).next as *mut WINHELP_DLL;
    }
    return 0;
}
