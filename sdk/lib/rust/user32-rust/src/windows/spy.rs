//! Translation of win32ss/user/user32/windows/spy.c — the message SPY debugger.
//!
//! `#define DEBUG_SPY 0` and the WINE message debug channel is off in this build, so
//! the spy is effectively compiled out: `spy_init()` returns FALSE (TRACE_ON(message)
//! is false), so SPY_EnterMessage / SPY_ExitMessage early-return, and the dump helpers
//! (`SPY_DumpMem`, `SPY_DumpStructure`) are dead at runtime. The structure of every
//! function and every name table is translated faithfully regardless: the `TRACE!`
//! macro is a no-op (the debug channel is off), so the dump bodies are inert but present.
//!
//! Cross-TU exports (called by other user32 TUs, plain cdecl — NOT WINAPI):
//!   SPY_GetClassLongOffsetName, SPY_GetMsgName, SPY_GetVKeyName,
//!   SPY_EnterMessage, SPY_ExitMessage
#![allow(non_snake_case, non_upper_case_globals, unused_variables, unused_assignments)]
#![allow(dead_code, unused_mut)]

use core::ffi::{c_char, c_void};
use win32k_sys::win32k::*;

// ── message-range bases (psdk/commctrl.h). Not in the win32k-sys substrate, so
//   defined locally with their canonical Win32 values.
const LVM_FIRST: UINT = 0x1000;
const TV_FIRST: UINT = 0x1100;
const HDM_FIRST: UINT = 0x1200;
const TCM_FIRST: UINT = 0x1300;
const PGM_FIRST: UINT = 0x1400;
const CCM_FIRST: UINT = 0x2000;

// SPY_MAX_MSGNUM == WM_USER (0x0400). WM_USER/WM_APP/GCW_ATOM come from win32k_sys.
const SPY_MAX_MSGNUM: UINT = WM_USER;
const SPY_INDENT_UNIT: INT = 4;

// SPY_EnterMessage iFlag values (spy.h)
const SPY_DISPATCHMESSAGE: INT = 0x0100;
const SPY_SENDMESSAGE: INT = 0x0103; // user32 spy.c: 0x0103; was 0x102
const SPY_DEFWNDPROC: INT = 0x0105; // user32 spy.c: 0x0105; was 0x104
// SPY_ExitMessage iFlag values
const SPY_RESULT_DEFWND: INT = 0x0005; // user32 spy.c: 0x0005; was 0
const SPY_RESULT_OK: INT = 0x0001;

// WC_DIALOG == MAKEINTATOM(0x8002); the C compares (ULONG_PTR)WC_DIALOG.
const WC_DIALOG_ATOM: ULONG_PTR = 0x8002;

const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF;

// ── The DEBUG_SPY channel is off in this build: TRACE is a no-op. The dump helpers
//   are thus dead at runtime but structurally faithful. (TRACE_ON(message) == FALSE.)

// ───────────────────────────────────────────────────────────────────────────────
// Name tables. The C tables are `const char * const Xxx[]` — arrays of C string
// pointers (ANSI, NUL-terminated). Translated as `[*const c_char; N]`, with NULL
// entries as `core::ptr::null()`. A small helper macro keeps them readable.
// ───────────────────────────────────────────────────────────────────────────────

const NULLP: *const c_char = core::ptr::null();

macro_rules! cstr {
    ($s:literal) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}

// Tables hold raw pointers; mark Sync so they can be `static`.
struct StrTable<const N: usize>([*const c_char; N]);
unsafe impl<const N: usize> Sync for StrTable<N> {}

static ClassLongOffsetNames: StrTable<14> = StrTable([
    cstr!("GCLP_MENUNAME"),      /*  -8 */
    cstr!("GCLP_HBRBACKGROUND"), /* -10 */
    cstr!("GCLP_HCURSOR"),       /* -12 */
    cstr!("GCLP_HICON"),         /* -14 */
    cstr!("GCLP_HMODULE"),       /* -16 */
    cstr!("GCL_CBWNDEXTRA"),     /* -18 */
    cstr!("GCL_CBCLSEXTRA"),     /* -20 */
    cstr!("?"),
    cstr!("GCLP_WNDPROC"), /* -24 */
    cstr!("GCL_STYLE"),    /* -26 */
    cstr!("?"),
    cstr!("?"),
    cstr!("GCW_ATOM"),     /* -32 */
    cstr!("GCLP_HICONSM"), /* -34 */
]);

static MessageTypeNames: StrTable<{ SPY_MAX_MSGNUM as usize + 1 }> = StrTable([
    cstr!("WM_NULL"), /* 0x00 */
    cstr!("WM_CREATE"),
    cstr!("WM_DESTROY"),
    cstr!("WM_MOVE"),
    cstr!("wm_sizewait"),
    cstr!("WM_SIZE"),
    cstr!("WM_ACTIVATE"),
    cstr!("WM_SETFOCUS"),
    cstr!("WM_KILLFOCUS"),
    cstr!("WM_SETVISIBLE"),
    cstr!("WM_ENABLE"),
    cstr!("WM_SETREDRAW"),
    cstr!("WM_SETTEXT"),
    cstr!("WM_GETTEXT"),
    cstr!("WM_GETTEXTLENGTH"),
    cstr!("WM_PAINT"),
    cstr!("WM_CLOSE"), /* 0x10 */
    cstr!("WM_QUERYENDSESSION"),
    cstr!("WM_QUIT"),
    cstr!("WM_QUERYOPEN"),
    cstr!("WM_ERASEBKGND"),
    cstr!("WM_SYSCOLORCHANGE"),
    cstr!("WM_ENDSESSION"),
    cstr!("wm_systemerror"),
    cstr!("WM_SHOWWINDOW"),
    cstr!("WM_CTLCOLOR"),
    cstr!("WM_WININICHANGE"),
    cstr!("WM_DEVMODECHANGE"),
    cstr!("WM_ACTIVATEAPP"),
    cstr!("WM_FONTCHANGE"),
    cstr!("WM_TIMECHANGE"),
    cstr!("WM_CANCELMODE"),
    cstr!("WM_SETCURSOR"), /* 0x20 */
    cstr!("WM_MOUSEACTIVATE"),
    cstr!("WM_CHILDACTIVATE"),
    cstr!("WM_QUEUESYNC"),
    cstr!("WM_GETMINMAXINFO"),
    cstr!("wm_unused3"),
    cstr!("wm_painticon"),
    cstr!("WM_ICONERASEBKGND"),
    cstr!("WM_NEXTDLGCTL"),
    cstr!("wm_alttabactive"),
    cstr!("WM_SPOOLERSTATUS"),
    cstr!("WM_DRAWITEM"),
    cstr!("WM_MEASUREITEM"),
    cstr!("WM_DELETEITEM"),
    cstr!("WM_VKEYTOITEM"),
    cstr!("WM_CHARTOITEM"),
    cstr!("WM_SETFONT"), /* 0x30 */
    cstr!("WM_GETFONT"),
    cstr!("WM_SETHOTKEY"),
    cstr!("WM_GETHOTKEY"),
    cstr!("wm_filesyschange"),
    cstr!("wm_isactiveicon"),
    cstr!("wm_queryparkicon"),
    cstr!("WM_QUERYDRAGICON"),
    cstr!("wm_querysavestate"),
    cstr!("WM_COMPAREITEM"),
    cstr!("wm_testing"),
    NULLP,
    NULLP,
    cstr!("WM_GETOBJECT"), /* 0x3d */
    cstr!("wm_activateshellwindow"),
    NULLP,
    NULLP, /* 0x40 */
    cstr!("wm_compacting"),
    NULLP,
    NULLP,
    cstr!("WM_COMMNOTIFY"),
    NULLP,
    cstr!("WM_WINDOWPOSCHANGING"), /* 0x0046 */
    cstr!("WM_WINDOWPOSCHANGED"),  /* 0x0047 */
    cstr!("WM_POWER"),
    NULLP,
    cstr!("WM_COPYDATA"),
    cstr!("WM_CANCELJOURNAL"),
    NULLP,
    NULLP,
    cstr!("WM_NOTIFY"),
    NULLP,
    /* 0x0050 */
    cstr!("WM_INPUTLANGCHANGEREQUEST"),
    cstr!("WM_INPUTLANGCHANGE"),
    cstr!("WM_TCARD"),
    cstr!("WM_HELP"),
    cstr!("WM_USERCHANGED"),
    cstr!("WM_NOTIFYFORMAT"),
    NULLP,
    NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0060 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0070 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP,
    cstr!("WM_CONTEXTMENU"),
    cstr!("WM_STYLECHANGING"),
    cstr!("WM_STYLECHANGED"),
    cstr!("WM_DISPLAYCHANGE"),
    cstr!("WM_GETICON"),
    cstr!("WM_SETICON"),     /* 0x0080 */
    cstr!("WM_NCCREATE"),    /* 0x0081 */
    cstr!("WM_NCDESTROY"),   /* 0x0082 */
    cstr!("WM_NCCALCSIZE"),  /* 0x0083 */
    cstr!("WM_NCHITTEST"),   /* 0x0084 */
    cstr!("WM_NCPAINT"),     /* 0x0085 */
    cstr!("WM_NCACTIVATE"),  /* 0x0086 */
    cstr!("WM_GETDLGCODE"),  /* 0x0087 */
    cstr!("WM_SYNCPAINT"),
    cstr!("WM_SYNCTASK"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0090 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x00A0 */
    cstr!("WM_NCMOUSEMOVE"),     /* 0x00a0 */
    cstr!("WM_NCLBUTTONDOWN"),   /* 0x00a1 */
    cstr!("WM_NCLBUTTONUP"),     /* 0x00a2 */
    cstr!("WM_NCLBUTTONDBLCLK"), /* 0x00a3 */
    cstr!("WM_NCRBUTTONDOWN"),   /* 0x00a4 */
    cstr!("WM_NCRBUTTONUP"),     /* 0x00a5 */
    cstr!("WM_NCRBUTTONDBLCLK"), /* 0x00a6 */
    cstr!("WM_NCMBUTTONDOWN"),   /* 0x00a7 */
    cstr!("WM_NCMBUTTONUP"),     /* 0x00a8 */
    cstr!("WM_NCMBUTTONDBLCLK"), /* 0x00a9 */
    NULLP,                       /* 0x00aa */
    cstr!("WM_NCXBUTTONDOWN"),   /* 0x00ab */
    cstr!("WM_NCXBUTTONUP"),     /* 0x00ac */
    cstr!("WM_NCXBUTTONDBLCLK"), /* 0x00ad */
    NULLP,                       /* 0x00ae */
    NULLP,                       /* 0x00af */
    /* 0x00B0 - Win32 Edit controls */
    cstr!("EM_GETSEL"),       /* 0x00b0 */
    cstr!("EM_SETSEL"),       /* 0x00b1 */
    cstr!("EM_GETRECT"),      /* 0x00b2 */
    cstr!("EM_SETRECT"),      /* 0x00b3 */
    cstr!("EM_SETRECTNP"),    /* 0x00b4 */
    cstr!("EM_SCROLL"),       /* 0x00b5 */
    cstr!("EM_LINESCROLL"),   /* 0x00b6 */
    cstr!("EM_SCROLLCARET"),  /* 0x00b7 */
    cstr!("EM_GETMODIFY"),    /* 0x00b8 */
    cstr!("EM_SETMODIFY"),    /* 0x00b9 */
    cstr!("EM_GETLINECOUNT"), /* 0x00ba */
    cstr!("EM_LINEINDEX"),    /* 0x00bb */
    cstr!("EM_SETHANDLE"),    /* 0x00bc */
    cstr!("EM_GETHANDLE"),    /* 0x00bd */
    cstr!("EM_GETTHUMB"),     /* 0x00be */
    NULLP,                    /* 0x00bf */
    NULLP,                    /* 0x00c0 */
    cstr!("EM_LINELENGTH"),   /* 0x00c1 */
    cstr!("EM_REPLACESEL"),   /* 0x00c2 */
    NULLP,                    /* 0x00c3 */
    cstr!("EM_GETLINE"),      /* 0x00c4 */
    cstr!("EM_LIMITTEXT"),    /* 0x00c5 */
    cstr!("EM_CANUNDO"),      /* 0x00c6 */
    cstr!("EM_UNDO"),         /* 0x00c7 */
    cstr!("EM_FMTLINES"),     /* 0x00c8 */
    cstr!("EM_LINEFROMCHAR"), /* 0x00c9 */
    NULLP,                    /* 0x00ca */
    cstr!("EM_SETTABSTOPS"),  /* 0x00cb */
    cstr!("EM_SETPASSWORDCHAR"),     /* 0x00cc */
    cstr!("EM_EMPTYUNDOBUFFER"),     /* 0x00cd */
    cstr!("EM_GETFIRSTVISIBLELINE"), /* 0x00ce */
    cstr!("EM_SETREADONLY"),         /* 0x00cf */
    cstr!("EM_SETWORDBREAKPROC"),    /* 0x00d0 */
    cstr!("EM_GETWORDBREAKPROC"),    /* 0x00d1 */
    cstr!("EM_GETPASSWORDCHAR"),     /* 0x00d2 */
    cstr!("EM_SETMARGINS"),   /* 0x00d3 */
    cstr!("EM_GETMARGINS"),   /* 0x00d4 */
    cstr!("EM_GETLIMITTEXT"), /* 0x00d5 */
    cstr!("EM_POSFROMCHAR"),  /* 0x00d6 */
    cstr!("EM_CHARFROMPOS"),  /* 0x00d7 */
    cstr!("EM_SETIMESTATUS"), /* 0x00d8 */
    cstr!("EM_GETIMESTATUS"), /* 0x00d9 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x00E0 - Win32 Scrollbars */
    cstr!("SBM_SETPOS"),         /* 0x00e0 */
    cstr!("SBM_GETPOS"),         /* 0x00e1 */
    cstr!("SBM_SETRANGE"),       /* 0x00e2 */
    cstr!("SBM_GETRANGE"),       /* 0x00e3 */
    cstr!("SBM_ENABLE_ARROWS"),  /* 0x00e4 */
    NULLP,
    cstr!("SBM_SETRANGEREDRAW"), /* 0x00e6 */
    NULLP, NULLP,
    cstr!("SBM_SETSCROLLINFO"),  /* 0x00e9 */
    cstr!("SBM_GETSCROLLINFO"),  /* 0x00ea */
    NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x00F0 - Win32 Buttons */
    cstr!("BM_GETCHECK"), /* 0x00f0 */
    cstr!("BM_SETCHECK"), /* 0x00f1 */
    cstr!("BM_GETSTATE"), /* 0x00f2 */
    cstr!("BM_SETSTATE"), /* 0x00f3 */
    cstr!("BM_SETSTYLE"), /* 0x00f4 */
    cstr!("BM_CLICK"),    /* 0x00f5 */
    cstr!("BM_GETIMAGE"), /* 0x00f6 */
    cstr!("BM_SETIMAGE"), /* 0x00f7 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_INPUT_DEVICE_CHANGE"), /* 0x00fe */
    cstr!("WM_INPUT"),               /* 0x00ff */
    cstr!("WM_KEYDOWN"),    /* 0x0100 */
    cstr!("WM_KEYUP"),      /* 0x0101 */
    cstr!("WM_CHAR"),       /* 0x0102 */
    cstr!("WM_DEADCHAR"),   /* 0x0103 */
    cstr!("WM_SYSKEYDOWN"), /* 0x0104 */
    cstr!("WM_SYSKEYUP"),   /* 0x0105 */
    cstr!("WM_SYSCHAR"),    /* 0x0106 */
    cstr!("WM_SYSDEADCHAR"),/* 0x0107 */
    NULLP,
    cstr!("WM_UNICHAR"),            /* 0x0109 */
    cstr!("WM_CONVERTREQUEST"),     /* 0x010a */
    cstr!("WM_CONVERTRESULT"),      /* 0x010b */
    cstr!("WM_INTERIM"),            /* 0x010c */
    cstr!("WM_IME_STARTCOMPOSITION"),/* 0x010d */
    cstr!("WM_IME_ENDCOMPOSITION"), /* 0x010e */
    cstr!("WM_IME_COMPOSITION"),    /* 0x010f */
    cstr!("WM_INITDIALOG"),    /* 0x0110 */
    cstr!("WM_COMMAND"),       /* 0x0111 */
    cstr!("WM_SYSCOMMAND"),    /* 0x0112 */
    cstr!("WM_TIMER"),         /* 0x0113 */
    cstr!("WM_HSCROLL"),       /* 0x0114 */
    cstr!("WM_VSCROLL"),       /* 0x0115 */
    cstr!("WM_INITMENU"),      /* 0x0116 */
    cstr!("WM_INITMENUPOPUP"), /* 0x0117 */
    cstr!("WM_SYSTIMER"),      /* 0x0118 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_MENUSELECT"),     /* 0x011f */
    cstr!("WM_MENUCHAR"),       /* 0x0120 */
    cstr!("WM_ENTERIDLE"),      /* 0x0121 */
    cstr!("WM_MENURBUTTONUP"),  /* 0x0122 */
    cstr!("WM_MENUDRAG"),       /* 0x0123 */
    cstr!("WM_MENUGETOBJECT"),  /* 0x0124 */
    cstr!("WM_UNINITMENUPOPUP"),/* 0x0125 */
    cstr!("WM_MENUCOMMAND"),    /* 0x0126 */
    cstr!("WM_CHANGEUISTATE"),  /* 0x0127 */
    cstr!("WM_UPDATEUISTATE"),  /* 0x0128 */
    cstr!("WM_QUERYUISTATE"),   /* 0x0129 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0130 */
    NULLP,
    cstr!("WM_LBTRACKPOINT"),      /* 0x0131 */
    cstr!("WM_CTLCOLORMSGBOX"),    /* 0x0132 */
    cstr!("WM_CTLCOLOREDIT"),      /* 0x0133 */
    cstr!("WM_CTLCOLORLISTBOX"),   /* 0x0134 */
    cstr!("WM_CTLCOLORBTN"),       /* 0x0135 */
    cstr!("WM_CTLCOLORDLG"),       /* 0x0136 */
    cstr!("WM_CTLCOLORSCROLLBAR"), /* 0x0137 */
    cstr!("WM_CTLCOLORSTATIC"),    /* 0x0138 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0140 - Win32 Comboboxes */
    cstr!("CB_GETEDITSEL"),   /* 0x0140 */
    cstr!("CB_LIMITTEXT"),    /* 0x0141 */
    cstr!("CB_SETEDITSEL"),   /* 0x0142 */
    cstr!("CB_ADDSTRING"),    /* 0x0143 */
    cstr!("CB_DELETESTRING"), /* 0x0144 */
    cstr!("CB_DIR"),          /* 0x0145 */
    cstr!("CB_GETCOUNT"),     /* 0x0146 */
    cstr!("CB_GETCURSEL"),    /* 0x0147 */
    cstr!("CB_GETLBTEXT"),    /* 0x0148 */
    cstr!("CB_GETLBTEXTLEN"), /* 0x0149 */
    cstr!("CB_INSERTSTRING"), /* 0x014a */
    cstr!("CB_RESETCONTENT"), /* 0x014b */
    cstr!("CB_FINDSTRING"),   /* 0x014c */
    cstr!("CB_SELECTSTRING"), /* 0x014d */
    cstr!("CB_SETCURSEL"),    /* 0x014e */
    cstr!("CB_SHOWDROPDOWN"), /* 0x014f */
    cstr!("CB_GETITEMDATA"),  /* 0x0150 */
    cstr!("CB_SETITEMDATA"),  /* 0x0151 */
    cstr!("CB_GETDROPPEDCONTROLRECT"), /* 0x0152 */
    cstr!("CB_SETITEMHEIGHT"),    /* 0x0153 */
    cstr!("CB_GETITEMHEIGHT"),    /* 0x0154 */
    cstr!("CB_SETEXTENDEDUI"),    /* 0x0155 */
    cstr!("CB_GETEXTENDEDUI"),    /* 0x0156 */
    cstr!("CB_GETDROPPEDSTATE"),  /* 0x0157 */
    cstr!("CB_FINDSTRINGEXACT"),  /* 0x0158 */
    cstr!("CB_SETLOCALE"),        /* 0x0159 */
    cstr!("CB_GETLOCALE"),        /* 0x015a */
    cstr!("CB_GETTOPINDEX"),      /* 0x015b */
    cstr!("CB_SETTOPINDEX"),      /* 0x015c */
    cstr!("CB_GETHORIZONTALEXTENT"), /* 0x015d */
    cstr!("CB_SETHORIZONTALEXTENT"), /* 0x015e */
    cstr!("CB_GETDROPPEDWIDTH"),  /* 0x015f */
    cstr!("CB_SETDROPPEDWIDTH"),  /* 0x0160 */
    cstr!("CB_INITSTORAGE"),      /* 0x0161 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0170 - Win32 Static controls */
    cstr!("STM_SETICON"),  /* 0x0170 */
    cstr!("STM_GETICON"),  /* 0x0171 */
    cstr!("STM_SETIMAGE"), /* 0x0172 */
    cstr!("STM_GETIMAGE"), /* 0x0173 */
    NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0180 - Win32 Listboxes */
    cstr!("LB_ADDSTRING"),      /* 0x0180 */
    cstr!("LB_INSERTSTRING"),   /* 0x0181 */
    cstr!("LB_DELETESTRING"),   /* 0x0182 */
    cstr!("LB_SELITEMRANGEEX"), /* 0x0183 */
    cstr!("LB_RESETCONTENT"),   /* 0x0184 */
    cstr!("LB_SETSEL"),         /* 0x0185 */
    cstr!("LB_SETCURSEL"),      /* 0x0186 */
    cstr!("LB_GETSEL"),         /* 0x0187 */
    cstr!("LB_GETCURSEL"),      /* 0x0188 */
    cstr!("LB_GETTEXT"),        /* 0x0189 */
    cstr!("LB_GETTEXTLEN"),     /* 0x018a */
    cstr!("LB_GETCOUNT"),       /* 0x018b */
    cstr!("LB_SELECTSTRING"),   /* 0x018c */
    cstr!("LB_DIR"),            /* 0x018d */
    cstr!("LB_GETTOPINDEX"),    /* 0x018e */
    cstr!("LB_FINDSTRING"),     /* 0x018f */
    cstr!("LB_GETSELCOUNT"),    /* 0x0190 */
    cstr!("LB_GETSELITEMS"),    /* 0x0191 */
    cstr!("LB_SETTABSTOPS"),    /* 0x0192 */
    cstr!("LB_GETHORIZONTALEXTENT"), /* 0x0193 */
    cstr!("LB_SETHORIZONTALEXTENT"), /* 0x0194 */
    cstr!("LB_SETCOLUMNWIDTH"), /* 0x0195 */
    cstr!("LB_ADDFILE"),        /* 0x0196 */
    cstr!("LB_SETTOPINDEX"),    /* 0x0197 */
    cstr!("LB_GETITEMRECT"),    /* 0x0198 */
    cstr!("LB_GETITEMDATA"),    /* 0x0199 */
    cstr!("LB_SETITEMDATA"),    /* 0x019a */
    cstr!("LB_SELITEMRANGE"),   /* 0x019b */
    cstr!("LB_SETANCHORINDEX"), /* 0x019c */
    cstr!("LB_GETANCHORINDEX"), /* 0x019d */
    cstr!("LB_SETCARETINDEX"),  /* 0x019e */
    cstr!("LB_GETCARETINDEX"),  /* 0x019f */
    cstr!("LB_SETITEMHEIGHT"),  /* 0x01a0 */
    cstr!("LB_GETITEMHEIGHT"),  /* 0x01a1 */
    cstr!("LB_FINDSTRINGEXACT"),/* 0x01a2 */
    cstr!("LB_CARETON"),        /* 0x01a3 */
    cstr!("LB_CARETOFF"),       /* 0x01a4 */
    cstr!("LB_SETLOCALE"),      /* 0x01a5 */
    cstr!("LB_GETLOCALE"),      /* 0x01a6 */
    cstr!("LB_SETCOUNT"),       /* 0x01a7 */
    cstr!("LB_INITSTORAGE"),    /* 0x01a8 */
    cstr!("LB_ITEMFROMPOINT"),  /* 0x01a9 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x01B0 */
    NULLP, NULLP,
    cstr!("LB_GETLISTBOXINFO"), /* 0x01b2 */
    NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x01C0 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x01D0 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x01E0 */
    NULLP,
    cstr!("MN_GETHMENU"), /* 0x01E1 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x01F0 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_MOUSEMOVE"),     /* 0x0200 */
    cstr!("WM_LBUTTONDOWN"),   /* 0x0201 */
    cstr!("WM_LBUTTONUP"),     /* 0x0202 */
    cstr!("WM_LBUTTONDBLCLK"), /* 0x0203 */
    cstr!("WM_RBUTTONDOWN"),   /* 0x0204 */
    cstr!("WM_RBUTTONUP"),     /* 0x0205 */
    cstr!("WM_RBUTTONDBLCLK"), /* 0x0206 */
    cstr!("WM_MBUTTONDOWN"),   /* 0x0207 */
    cstr!("WM_MBUTTONUP"),     /* 0x0208 */
    cstr!("WM_MBUTTONDBLCLK"), /* 0x0209 */
    cstr!("WM_MOUSEWHEEL"),    /* 0x020A */
    cstr!("WM_XBUTTONDOWN"),   /* 0x020B */
    cstr!("WM_XBUTTONUP"),     /* 0x020C */
    cstr!("WM_XBUTTONDBLCLK"), /* 0x020D */
    cstr!("WM_MOUSEHWHEEL"),   /* 0x020E */
    NULLP,
    cstr!("WM_PARENTNOTIFY"),  /* 0x0210 */
    cstr!("WM_ENTERMENULOOP"), /* 0x0211 */
    cstr!("WM_EXITMENULOOP"),  /* 0x0212 */
    cstr!("WM_NEXTMENU"),      /* 0x0213 */
    cstr!("WM_SIZING"),
    cstr!("WM_CAPTURECHANGED"),
    cstr!("WM_MOVING"),
    NULLP,
    cstr!("WM_POWERBROADCAST"),
    cstr!("WM_DEVICECHANGE"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_MDICREATE"),      /* 0x0220 */
    cstr!("WM_MDIDESTROY"),     /* 0x0221 */
    cstr!("WM_MDIACTIVATE"),    /* 0x0222 */
    cstr!("WM_MDIRESTORE"),     /* 0x0223 */
    cstr!("WM_MDINEXT"),        /* 0x0224 */
    cstr!("WM_MDIMAXIMIZE"),    /* 0x0225 */
    cstr!("WM_MDITILE"),        /* 0x0226 */
    cstr!("WM_MDICASCADE"),     /* 0x0227 */
    cstr!("WM_MDIICONARRANGE"), /* 0x0228 */
    cstr!("WM_MDIGETACTIVE"),   /* 0x0229 */
    cstr!("WM_DROPOBJECT"),
    cstr!("WM_QUERYDROPOBJECT"),
    cstr!("WM_BEGINDRAG"),
    cstr!("WM_DRAGLOOP"),
    cstr!("WM_DRAGSELECT"),
    cstr!("WM_DRAGMOVE"),
    /* 0x0230*/
    cstr!("WM_MDISETMENU"),     /* 0x0230 */
    cstr!("WM_ENTERSIZEMOVE"),  /* 0x0231 */
    cstr!("WM_EXITSIZEMOVE"),   /* 0x0232 */
    cstr!("WM_DROPFILES"),      /* 0x0233 */
    cstr!("WM_MDIREFRESHMENU"),
    NULLP, NULLP, NULLP,
    /* 0x0238*/
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0240 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0250 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0260 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0280 */
    NULLP,
    cstr!("WM_IME_SETCONTEXT"),      /* 0x0281 */
    cstr!("WM_IME_NOTIFY"),          /* 0x0282 */
    cstr!("WM_IME_CONTROL"),         /* 0x0283 */
    cstr!("WM_IME_COMPOSITIONFULL"), /* 0x0284 */
    cstr!("WM_IME_SELECT"),          /* 0x0285 */
    cstr!("WM_IME_CHAR"),            /* 0x0286 */
    NULLP,
    cstr!("WM_IME_REQUEST"),         /* 0x0288 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_IME_KEYDOWN"),         /* 0x0290 */
    cstr!("WM_IME_KEYUP"),           /* 0x0291 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x02a0 */
    cstr!("WM_NCMOUSEHOVER"), /* 0x02A0 */
    cstr!("WM_MOUSEHOVER"),   /* 0x02A1 */
    cstr!("WM_NCMOUSELEAVE"), /* 0x02A2 */
    cstr!("WM_MOUSELEAVE"),   /* 0x02A3 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_WTSSESSION_CHANGE"), /* 0x02B1 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x02c0 */
    cstr!("WM_TABLET_FIRST"),    /* 0x02c0 */
    cstr!("WM_TABLET_FIRST+1"),  /* 0x02c1 */
    cstr!("WM_TABLET_FIRST+2"),  /* 0x02c2 */
    cstr!("WM_TABLET_FIRST+3"),  /* 0x02c3 */
    cstr!("WM_TABLET_FIRST+4"),  /* 0x02c4 */
    cstr!("WM_TABLET_FIRST+5"),  /* 0x02c5 */
    cstr!("WM_TABLET_FIRST+7"),  /* 0x02c6 */
    cstr!("WM_TABLET_FIRST+8"),  /* 0x02c7 */
    cstr!("WM_TABLET_FIRST+9"),  /* 0x02c8 */
    cstr!("WM_TABLET_FIRST+10"), /* 0x02c9 */
    cstr!("WM_TABLET_FIRST+11"), /* 0x02ca */
    cstr!("WM_TABLET_FIRST+12"), /* 0x02cb */
    cstr!("WM_TABLET_FIRST+13"), /* 0x02cc */
    cstr!("WM_TABLET_FIRST+14"), /* 0x02cd */
    cstr!("WM_TABLET_FIRST+15"), /* 0x02ce */
    cstr!("WM_TABLET_FIRST+16"), /* 0x02cf */
    cstr!("WM_TABLET_FIRST+17"), /* 0x02d0 */
    cstr!("WM_TABLET_FIRST+18"), /* 0x02d1 */
    cstr!("WM_TABLET_FIRST+19"), /* 0x02d2 */
    cstr!("WM_TABLET_FIRST+20"), /* 0x02d3 */
    cstr!("WM_TABLET_FIRST+21"), /* 0x02d4 */
    cstr!("WM_TABLET_FIRST+22"), /* 0x02d5 */
    cstr!("WM_TABLET_FIRST+23"), /* 0x02d6 */
    cstr!("WM_TABLET_FIRST+24"), /* 0x02d7 */
    cstr!("WM_TABLET_FIRST+25"), /* 0x02d8 */
    cstr!("WM_TABLET_FIRST+26"), /* 0x02d9 */
    cstr!("WM_TABLET_FIRST+27"), /* 0x02da */
    cstr!("WM_TABLET_FIRST+28"), /* 0x02db */
    cstr!("WM_TABLET_FIRST+29"), /* 0x02dc */
    cstr!("WM_TABLET_FIRST+30"), /* 0x02dd */
    cstr!("WM_TABLET_FIRST+31"), /* 0x02de */
    cstr!("WM_TABLET_LAST"),     /* 0x02df */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_CUT"), /* 0x0300 */
    cstr!("WM_COPY"),
    cstr!("WM_PASTE"),
    cstr!("WM_CLEAR"),
    cstr!("WM_UNDO"),
    cstr!("WM_RENDERFORMAT"),
    cstr!("WM_RENDERALLFORMATS"),
    cstr!("WM_DESTROYCLIPBOARD"),
    cstr!("WM_DRAWCLIPBOARD"),
    cstr!("WM_PAINTCLIPBOARD"),
    cstr!("WM_VSCROLLCLIPBOARD"),
    cstr!("WM_SIZECLIPBOARD"),
    cstr!("WM_ASKCBFORMATNAME"),
    cstr!("WM_CHANGECBCHAIN"),
    cstr!("WM_HSCROLLCLIPBOARD"),
    cstr!("WM_QUERYNEWPALETTE"), /* 0x030f*/
    cstr!("WM_PALETTEISCHANGING"),
    cstr!("WM_PALETTECHANGED"),
    cstr!("WM_HOTKEY"),          /* 0x0312 */
    cstr!("WM_POPUPSYSTEMMENU"), /* 0x0313 */
    NULLP, NULLP, NULLP,
    cstr!("WM_PRINT"),        /* 0x0317 */
    cstr!("WM_PRINTCLIENT"),  /* 0x0318 */
    cstr!("WM_APPCOMMAND"),   /* 0x0319 */
    cstr!("WM_THEMECHANGED"), /* 0x031A */
    NULLP, NULLP,
    cstr!("WM_CLIPBOARDUPDATE"),       /* 0x031D */
    cstr!("WM_DWMCOMPOSITIONCHANGED"), /* 0x031E */
    cstr!("WM_DWMNCRENDERINGCHANGED"), /* 0x031F */
    cstr!("WM_DWMCOLORIZATIONCOLORCHANGED"), /* 0x0320 */
    cstr!("WM_DWMWINDOWMAXIMIZEDCHANGE"),    /* 0x0321 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_GETTITLEBARINFOEX"), /* 0x033F */
    /* 0x0340 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x0350 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_HANDHELDFIRST"),   /* 0x0358 */
    cstr!("WM_HANDHELDFIRST+1"), /* 0x0359 */
    cstr!("WM_HANDHELDFIRST+2"), /* 0x035A */
    cstr!("WM_HANDHELDFIRST+3"), /* 0x035B */
    cstr!("WM_HANDHELDFIRST+4"), /* 0x035C */
    cstr!("WM_HANDHELDFIRST+5"), /* 0x035D */
    cstr!("WM_HANDHELDFIRST+6"), /* 0x035E */
    cstr!("WM_HANDHELDLAST"),    /* 0x035F */
    cstr!("WM_QUERYAFXWNDPROC"), /*  0x0360 WM_AFXFIRST */
    cstr!("WM_SIZEPARENT"),      /*  0x0361 */
    cstr!("WM_SETMESSAGESTRING"),/*  0x0362 */
    cstr!("WM_IDLEUPDATECMDUI"), /*  0x0363 */
    cstr!("WM_INITIALUPDATE"),   /*  0x0364 */
    cstr!("WM_COMMANDHELP"),     /*  0x0365 */
    cstr!("WM_HELPHITTEST"),     /*  0x0366 */
    cstr!("WM_EXITHELPMODE"),    /*  0x0367 */
    cstr!("WM_RECALCPARENT"),    /*  0x0368 */
    cstr!("WM_SIZECHILD"),       /*  0x0369 */
    cstr!("WM_KICKIDLE"),        /*  0x036A */
    cstr!("WM_QUERYCENTERWND"),  /*  0x036B */
    cstr!("WM_DISABLEMODAL"),    /*  0x036C */
    cstr!("WM_FLOATSTATUS"),     /*  0x036D */
    cstr!("WM_ACTIVATETOPLEVEL"),/*  0x036E */
    cstr!("WM_QUERY3DCONTROLS"), /*  0x036F */
    NULLP, NULLP, NULLP,
    cstr!("WM_SOCKET_NOTIFY"),    /*  0x0373 */
    cstr!("WM_SOCKET_DEAD"),      /*  0x0374 */
    cstr!("WM_POPMESSAGESTRING"), /*  0x0375 */
    cstr!("WM_OCC_LOADFROMSTREAM"),     /* 0x0376 */
    cstr!("WM_OCC_LOADFROMSTORAGE"),    /* 0x0377 */
    cstr!("WM_OCC_INITNEW"),            /* 0x0378 */
    cstr!("WM_QUEUE_SENTINEL"),         /* 0x0379 */
    cstr!("WM_OCC_LOADFROMSTREAM_EX"),  /* 0x037A */
    cstr!("WM_OCC_LOADFROMSTORAGE_EX"), /* 0x037B */
    NULLP, NULLP, NULLP,
    cstr!("WM_AFXLAST"), /* 0x037F */
    cstr!("WM_PENWINFIRST"),   /* 0x0380 */
    cstr!("WM_RCRESULT"),      /* 0x0381 */
    cstr!("WM_HOOKRCRESULT"),  /* 0x0382 */
    cstr!("WM_GLOBALRCCHANGE"),/* 0x0383 */
    cstr!("WM_SKB"),           /* 0x0384 */
    cstr!("WM_HEDITCTL"),      /* 0x0385 */
    NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_PENWINLAST"),    /* 0x038F */
    cstr!("WM_COALESCE_FIRST"),/* 0x0390 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_COALESCE_LAST"), /* 0x039F */
    /* 0x03a0 */
    cstr!("MM_JOY1MOVE"),
    cstr!("MM_JOY2MOVE"),
    cstr!("MM_JOY1ZMOVE"),
    cstr!("MM_JOY2ZMOVE"),
    NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x03b0 */
    NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("MM_JOY1BUTTONDOWN"),
    cstr!("MM_JOY2BUTTONDOWN"),
    cstr!("MM_JOY1BUTTONUP"),
    cstr!("MM_JOY2BUTTONUP"),
    cstr!("MM_MCINOTIFY"), /* 0x03B9 */
    NULLP,
    cstr!("MM_WOM_OPEN"),  /* 0x03BB */
    cstr!("MM_WOM_CLOSE"), /* 0x03BC */
    cstr!("MM_WOM_DONE"),  /* 0x03BD */
    cstr!("MM_WIM_OPEN"),  /* 0x03BE */
    cstr!("MM_WIM_CLOSE"), /* 0x03BF */
    /* 0x03c0 */
    cstr!("MM_WIM_DATA"),       /* 0x03C0 */
    cstr!("MM_MIM_OPEN"),       /* 0x03C1 */
    cstr!("MM_MIM_CLOSE"),      /* 0x03C2 */
    cstr!("MM_MIM_DATA"),       /* 0x03C3 */
    cstr!("MM_MIM_LONGDATA"),   /* 0x03C4 */
    cstr!("MM_MIM_ERROR"),      /* 0x03C5 */
    cstr!("MM_MIM_LONGERROR"),  /* 0x03C6 */
    cstr!("MM_MOM_OPEN"),       /* 0x03C7 */
    cstr!("MM_MOM_CLOSE"),      /* 0x03C8 */
    cstr!("MM_MOM_DONE"),       /* 0x03C9 */
    cstr!("MM_MOM_POSITIONCB"), /* 0x03CA */
    cstr!("MM_MCISIGNAL"),      /* 0x03CB */
    cstr!("MM_MIM_MOREDATA"),   /* 0x03CC */
    NULLP, NULLP, NULLP,
    /* 0x03d0 */
    cstr!("MM_MIXM_LINE_CHANGE"),
    cstr!("MM_MIXM_CONTROL_CHANGE"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x03e0 */
    cstr!("WM_DDE_INITIATE"),  /* 0x3E0 */
    cstr!("WM_DDE_TERMINATE"), /* 0x3E1 */
    cstr!("WM_DDE_ADVISE"),    /* 0x3E2 */
    cstr!("WM_DDE_UNADVISE"),  /* 0x3E3 */
    cstr!("WM_DDE_ACK"),       /* 0x3E4 */
    cstr!("WM_DDE_DATA"),      /* 0x3E5 */
    cstr!("WM_DDE_REQUEST"),   /* 0x3E6 */
    cstr!("WM_DDE_POKE"),      /* 0x3E7 */
    cstr!("WM_DDE_EXECUTE"),   /* 0x3E8 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    /* 0x03f0 */
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("WM_USER"), /* 0x0400 */
]);

const SPY_MAX_LVMMSGNUM: usize = 182;
static LVMMessageTypeNames: StrTable<{ SPY_MAX_LVMMSGNUM + 1 }> = StrTable([
    cstr!("LVM_GETBKCOLOR"), /* 1000 */
    cstr!("LVM_SETBKCOLOR"),
    cstr!("LVM_GETIMAGELIST"),
    cstr!("LVM_SETIMAGELIST"),
    cstr!("LVM_GETITEMCOUNT"),
    cstr!("LVM_GETITEMA"),
    cstr!("LVM_SETITEMA"),
    cstr!("LVM_INSERTITEMA"),
    cstr!("LVM_DELETEITEM"),
    cstr!("LVM_DELETEALLITEMS"),
    cstr!("LVM_GETCALLBACKMASK"),
    cstr!("LVM_SETCALLBACKMASK"),
    cstr!("LVM_GETNEXTITEM"),
    cstr!("LVM_FINDITEMA"),
    cstr!("LVM_GETITEMRECT"),
    cstr!("LVM_SETITEMPOSITION"),
    cstr!("LVM_GETITEMPOSITION"),
    cstr!("LVM_GETSTRINGWIDTHA"),
    cstr!("LVM_HITTEST"),
    cstr!("LVM_ENSUREVISIBLE"),
    cstr!("LVM_SCROLL"),
    cstr!("LVM_REDRAWITEMS"),
    cstr!("LVM_ARRANGE"),
    cstr!("LVM_EDITLABELA"),
    cstr!("LVM_GETEDITCONTROL"),
    cstr!("LVM_GETCOLUMNA"),
    cstr!("LVM_SETCOLUMNA"),
    cstr!("LVM_INSERTCOLUMNA"),
    cstr!("LVM_DELETECOLUMN"),
    cstr!("LVM_GETCOLUMNWIDTH"),
    cstr!("LVM_SETCOLUMNWIDTH"),
    cstr!("LVM_GETHEADER"),
    NULLP,
    cstr!("LVM_CREATEDRAGIMAGE"),
    cstr!("LVM_GETVIEWRECT"),
    cstr!("LVM_GETTEXTCOLOR"),
    cstr!("LVM_SETTEXTCOLOR"),
    cstr!("LVM_GETTEXTBKCOLOR"),
    cstr!("LVM_SETTEXTBKCOLOR"),
    cstr!("LVM_GETTOPINDEX"),
    cstr!("LVM_GETCOUNTPERPAGE"),
    cstr!("LVM_GETORIGIN"),
    cstr!("LVM_UPDATE"),
    cstr!("LVM_SETITEMSTATE"),
    cstr!("LVM_GETITEMSTATE"),
    cstr!("LVM_GETITEMTEXTA"),
    cstr!("LVM_SETITEMTEXTA"),
    cstr!("LVM_SETITEMCOUNT"),
    cstr!("LVM_SORTITEMS"),
    cstr!("LVM_SETITEMPOSITION32"),
    cstr!("LVM_GETSELECTEDCOUNT"),
    cstr!("LVM_GETITEMSPACING"),
    cstr!("LVM_GETISEARCHSTRINGA"),
    cstr!("LVM_SETICONSPACING"),
    cstr!("LVM_SETEXTENDEDLISTVIEWSTYLE"),
    cstr!("LVM_GETEXTENDEDLISTVIEWSTYLE"),
    cstr!("LVM_GETSUBITEMRECT"),
    cstr!("LVM_SUBITEMHITTEST"),
    cstr!("LVM_SETCOLUMNORDERARRAY"),
    cstr!("LVM_GETCOLUMNORDERARRAY"),
    cstr!("LVM_SETHOTITEM"),
    cstr!("LVM_GETHOTITEM"),
    cstr!("LVM_SETHOTCURSOR"),
    cstr!("LVM_GETHOTCURSOR"),
    cstr!("LVM_APPROXIMATEVIEWRECT"),
    cstr!("LVM_SETWORKAREAS"),
    cstr!("LVM_GETSELECTIONMARK"),
    cstr!("LVM_SETSELECTIONMARK"),
    cstr!("LVM_SETBKIMAGEA"),
    cstr!("LVM_GETBKIMAGEA"),
    cstr!("LVM_GETWORKAREAS"),
    cstr!("LVM_SETHOVERTIME"),
    cstr!("LVM_GETHOVERTIME"),
    cstr!("LVM_GETNUMBEROFWORKAREAS"),
    cstr!("LVM_SETTOOLTIPS"),
    cstr!("LVM_GETITEMW"),
    cstr!("LVM_SETITEMW"),
    cstr!("LVM_INSERTITEMW"),
    cstr!("LVM_GETTOOLTIPS"),
    NULLP, NULLP, NULLP, NULLP,
    cstr!("LVM_FINDITEMW"),
    NULLP, NULLP, NULLP,
    cstr!("LVM_GETSTRINGWIDTHW"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("LVM_GETCOLUMNW"),
    cstr!("LVM_SETCOLUMNW"),
    cstr!("LVM_INSERTCOLUMNW"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("LVM_GETITEMTEXTW"),
    cstr!("LVM_SETITEMTEXTW"),
    cstr!("LVM_GETISEARCHSTRINGW"),
    cstr!("LVM_EDITLABELW"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP,
    cstr!("LVM_SETBKIMAGEW"),
    cstr!("LVM_GETBKIMAGEW"), /* 0x108B */
    cstr!("LVM_SETSELECTEDCOLUMN"),
    cstr!("LVM_SETTILEWIDTH"),
    cstr!("LVM_SETVIEW"),
    cstr!("LVM_GETVIEW"),
    NULLP,
    cstr!("LVM_INSERTGROUP"),
    NULLP,
    cstr!("LVM_SETGROUPINFO"),
    NULLP,
    cstr!("LVM_GETGROUPINFO"),
    cstr!("LVM_REMOVEGROUP"),
    cstr!("LVM_MOVEGROUP"),
    NULLP, NULLP,
    cstr!("LVM_MOVEITEMTOGROUP"),
    cstr!("LVM_SETGROUPMETRICS"),
    cstr!("LVM_GETGROUPMETRICS"),
    cstr!("LVM_ENABLEGROUPVIEW"),
    cstr!("LVM_SORTGROUPS"),
    cstr!("LVM_INSERTGROUPSORTED"),
    cstr!("LVM_REMOVEALLGROUPS"),
    cstr!("LVM_HASGROUP"),
    cstr!("LVM_SETTILEVIEWINFO"),
    cstr!("LVM_GETTILEVIEWINFO"),
    cstr!("LVM_SETTILEINFO"),
    cstr!("LVM_GETTILEINFO"),
    cstr!("LVM_SETINSERTMARK"),
    cstr!("LVM_GETINSERTMARK"),
    cstr!("LVM_INSERTMARKHITTEST"),
    cstr!("LVM_GETINSERTMARKRECT"),
    cstr!("LVM_SETINSERTMARKCOLOR"),
    cstr!("LVM_GETINSERTMARKCOLOR"),
    NULLP,
    cstr!("LVM_SETINFOTIP"),
    cstr!("LVM_GETSELECTEDCOLUMN"),
    cstr!("LVM_ISGROUPVIEWENABLED"),
    cstr!("LVM_GETOUTLINECOLOR"),
    cstr!("LVM_SETOUTLINECOLOR"),
    NULLP,
    cstr!("LVM_CANCELEDITLABEL"),
    cstr!("LVM_MAPINDEXTOID"),
    cstr!("LVM_MAPIDTOINDEX"),
    cstr!("LVM_ISITEMVISIBLE"),
]);

const SPY_MAX_TVMSGNUM: usize = 65;
static TVMessageTypeNames: StrTable<{ SPY_MAX_TVMSGNUM + 1 }> = StrTable([
    cstr!("TVM_INSERTITEMA"), /* 1100 */
    cstr!("TVM_DELETEITEM"),
    cstr!("TVM_EXPAND"),
    NULLP,
    cstr!("TVM_GETITEMRECT"),
    cstr!("TVM_GETCOUNT"),
    cstr!("TVM_GETINDENT"),
    cstr!("TVM_SETINDENT"),
    cstr!("TVM_GETIMAGELIST"),
    cstr!("TVM_SETIMAGELIST"),
    cstr!("TVM_GETNEXTITEM"),
    cstr!("TVM_SELECTITEM"),
    cstr!("TVM_GETITEMA"),
    cstr!("TVM_SETITEMA"),
    cstr!("TVM_EDITLABELA"),
    cstr!("TVM_GETEDITCONTROL"),
    cstr!("TVM_GETVISIBLECOUNT"),
    cstr!("TVM_HITTEST"),
    cstr!("TVM_CREATEDRAGIMAGE"),
    cstr!("TVM_SORTCHILDREN"),
    cstr!("TVM_ENSUREVISIBLE"),
    cstr!("TVM_SORTCHILDRENCB"),
    cstr!("TVM_ENDEDITLABELNOW"),
    cstr!("TVM_GETISEARCHSTRINGA"),
    cstr!("TVM_SETTOOLTIPS"),
    cstr!("TVM_GETTOOLTIPS"),
    cstr!("TVM_SETINSERTMARK"),
    cstr!("TVM_SETITEMHEIGHT"),
    cstr!("TVM_GETITEMHEIGHT"),
    cstr!("TVM_SETBKCOLOR"),
    cstr!("TVM_SETTEXTCOLOR"),
    cstr!("TVM_GETBKCOLOR"),
    cstr!("TVM_GETTEXTCOLOR"),
    cstr!("TVM_SETSCROLLTIME"),
    cstr!("TVM_GETSCROLLTIME"),
    cstr!("TVM_UNKNOWN35"),
    cstr!("TVM_UNKNOWN36"),
    cstr!("TVM_SETINSERTMARKCOLOR"),
    cstr!("TVM_GETINSERTMARKCOLOR"),
    cstr!("TVM_GETITEMSTATE"),
    cstr!("TVM_SETLINECOLOR"),
    cstr!("TVM_GETLINECOLOR"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("TVM_INSERTITEMW"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("TVM_GETITEMW"),
    cstr!("TVM_SETITEMW"),
    cstr!("TVM_GETISEARCHSTRINGW"),
    cstr!("TVM_EDITLABELW"),
]);

const SPY_MAX_HDMMSGNUM: usize = 19;
static HDMMessageTypeNames: StrTable<{ SPY_MAX_HDMMSGNUM + 1 }> = StrTable([
    cstr!("HDM_GETITEMCOUNT"), /* 1200 */
    cstr!("HDM_INSERTITEMA"),
    cstr!("HDM_DELETEITEM"),
    cstr!("HDM_GETITEMA"),
    cstr!("HDM_SETITEMA"),
    cstr!("HDM_LAYOUT"),
    cstr!("HDM_HITTEST"),
    cstr!("HDM_GETITEMRECT"),
    cstr!("HDM_SETIMAGELIST"),
    cstr!("HDM_GETIMAGELIST"),
    cstr!("HDM_INSERTITEMW"),
    cstr!("HDM_GETITEMW"),
    cstr!("HDM_SETITEMW"),
    NULLP, NULLP,
    cstr!("HDM_ORDERTOINDEX"),
    cstr!("HDM_CREATEDRAGIMAGE"),
    cstr!("GETORDERARRAYINDEX"),
    cstr!("SETORDERARRAYINDEX"),
    cstr!("SETHOTDIVIDER"),
]);

const SPY_MAX_TCMMSGNUM: usize = 62;
static TCMMessageTypeNames: StrTable<{ SPY_MAX_TCMMSGNUM + 1 }> = StrTable([
    NULLP, /* 1300 */
    NULLP,
    cstr!("TCM_SETIMAGELIST"),
    cstr!("TCM_GETIMAGELIST"),
    cstr!("TCM_GETITEMCOUNT"),
    cstr!("TCM_GETITEMA"),
    cstr!("TCM_SETITEMA"),
    cstr!("TCM_INSERTITEMA"),
    cstr!("TCM_DELETEITEM"),
    cstr!("TCM_DELETEALLITEMS"),
    cstr!("TCM_GETITEMRECT"),
    cstr!("TCM_GETCURSEL"),
    cstr!("TCM_SETCURSEL"),
    cstr!("TCM_HITTEST"),
    cstr!("TCM_SETITEMEXTRA"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("TCM_ADJUSTRECT"),
    cstr!("TCM_SETITEMSIZE"),
    cstr!("TCM_REMOVEIMAGE"),
    cstr!("TCM_SETPADDING"),
    cstr!("TCM_GETROWCOUNT"),
    cstr!("TCM_GETTOOLTIPS"),
    cstr!("TCM_SETTOOLTIPS"),
    cstr!("TCM_GETCURFOCUS"),
    cstr!("TCM_SETCURFOCUS"),
    cstr!("TCM_SETMINTABWIDTH"),
    cstr!("TCM_DESELECTALL"),
    cstr!("TCM_HIGHLIGHTITEM"),
    cstr!("TCM_SETEXTENDEDSTYLE"),
    cstr!("TCM_GETEXTENDEDSTYLE"),
    NULLP, NULLP, NULLP, NULLP, NULLP, NULLP,
    cstr!("TCM_GETITEMW"),
    cstr!("TCM_SETITEMW"),
    cstr!("TCM_INSERTITEMW"),
]);

const SPY_MAX_PGMMSGNUM: usize = 13;
static PGMMessageTypeNames: StrTable<{ SPY_MAX_PGMMSGNUM + 1 }> = StrTable([
    NULLP, /* 1400 */
    cstr!("PGM_SETCHILD"),
    cstr!("PGM_RECALCSIZE"),
    cstr!("PGM_FORWARDMOUSE"),
    cstr!("PGM_SETBKCOLOR"),
    cstr!("PGM_GETBKCOLOR"),
    cstr!("PGM_SETBORDER"),
    cstr!("PGM_GETBORDER"),
    cstr!("PGM_SETPOS"),
    cstr!("PGM_GETPOS"),
    cstr!("PGM_SETBUTTONSIZE"),
    cstr!("PGM_GETBUTTONSIZE"),
    cstr!("PGM_GETBUTTONSTATE"),
    cstr!("PGM_GETDROPTARGET"),
]);

const SPY_MAX_CCMMSGNUM: usize = 9;
static CCMMessageTypeNames: StrTable<{ SPY_MAX_CCMMSGNUM + 1 }> = StrTable([
    NULLP, /* 0x2000 */
    cstr!("CCM_SETBKCOLOR"),
    cstr!("CCM_SETCOLORSCHEME"),
    cstr!("CCM_GETCOLORSCHEME"),
    cstr!("CCM_GETDROPTARGET"),
    cstr!("CCM_SETUNICODEFORMAT"),
    cstr!("CCM_GETUNICODEFORMAT"),
    cstr!("CCM_SETVERSION"),
    cstr!("CCM_GETVERSION"),
    cstr!("CCM_SETNOTIFYWINDOW"),
]);

const SPY_MAX_WINEMSGNUM: usize = 9;
// WINEMessageTypeNames + the WM_WINE_* lookup live under #ifndef __REACTOS__ in the
// C source. __REACTOS__ is defined in this build, so they are excluded entirely.

/* Virtual key names */
const SPY_MAX_VKKEYSNUM: usize = 255;
static VK_KeyNames: StrTable<{ SPY_MAX_VKKEYSNUM + 1 }> = StrTable([
    NULLP,             /* 0x00 */
    cstr!("VK_LBUTTON"),  /* 0x01 */
    cstr!("VK_RBUTTON"),  /* 0x02 */
    cstr!("VK_CANCEL"),   /* 0x03 */
    cstr!("VK_MBUTTON"),  /* 0x04 */
    cstr!("VK_XBUTTON1"), /* 0x05 */
    cstr!("VK_XBUTTON2"), /* 0x06 */
    NULLP,             /* 0x07 */
    cstr!("VK_BACK"),  /* 0x08 */
    cstr!("VK_TAB"),   /* 0x09 */
    NULLP,             /* 0x0A */
    NULLP,             /* 0x0B */
    cstr!("VK_CLEAR"), /* 0x0C */
    cstr!("VK_RETURN"),/* 0x0D */
    NULLP,             /* 0x0E */
    NULLP,             /* 0x0F */
    cstr!("VK_SHIFT"),   /* 0x10 */
    cstr!("VK_CONTROL"), /* 0x11 */
    cstr!("VK_MENU"),    /* 0x12 */
    cstr!("VK_PAUSE"),   /* 0x13 */
    cstr!("VK_CAPITAL"), /* 0x14 */
    NULLP,             /* 0x15 */
    NULLP,             /* 0x16 */
    NULLP,             /* 0x17 */
    NULLP,             /* 0x18 */
    NULLP,             /* 0x19 */
    NULLP,             /* 0x1A */
    cstr!("VK_ESCAPE"),     /* 0x1B */
    cstr!("VK_CONVERT"),    /* 0x1C */
    cstr!("VK_NONCONVERT"), /* 0x1D */
    cstr!("VK_ACCEPT"),     /* 0x1E */
    cstr!("VK_MODECHANGE"), /* 0x1F */
    cstr!("VK_SPACE"),  /* 0x20 */
    cstr!("VK_PRIOR"),  /* 0x21 */
    cstr!("VK_NEXT"),   /* 0x22 */
    cstr!("VK_END"),    /* 0x23 */
    cstr!("VK_HOME"),   /* 0x24 */
    cstr!("VK_LEFT"),   /* 0x25 */
    cstr!("VK_UP"),     /* 0x26 */
    cstr!("VK_RIGHT"),  /* 0x27 */
    cstr!("VK_DOWN"),   /* 0x28 */
    cstr!("VK_SELECT"), /* 0x29 */
    cstr!("VK_PRINT"),  /* 0x2A */
    cstr!("VK_EXECUTE"),/* 0x2B */
    cstr!("VK_SNAPSHOT"),/* 0x2C */
    cstr!("VK_INSERT"), /* 0x2D */
    cstr!("VK_DELETE"), /* 0x2E */
    cstr!("VK_HELP"),   /* 0x2F */
    cstr!("VK_0"),      /* 0x30 */
    cstr!("VK_1"),      /* 0x31 */
    cstr!("VK_2"),      /* 0x32 */
    cstr!("VK_3"),      /* 0x33 */
    cstr!("VK_4"),      /* 0x34 */
    cstr!("VK_5"),      /* 0x35 */
    cstr!("VK_6"),      /* 0x36 */
    cstr!("VK_7"),      /* 0x37 */
    cstr!("VK_8"),      /* 0x38 */
    cstr!("VK_9"),      /* 0x39 */
    NULLP,             /* 0x3A */
    NULLP,             /* 0x3B */
    NULLP,             /* 0x3C */
    NULLP,             /* 0x3D */
    NULLP,             /* 0x3E */
    NULLP,             /* 0x3F */
    NULLP,             /* 0x40 */
    cstr!("VK_A"),      /* 0x41 */
    cstr!("VK_B"),      /* 0x42 */
    cstr!("VK_C"),      /* 0x43 */
    cstr!("VK_D"),      /* 0x44 */
    cstr!("VK_E"),      /* 0x45 */
    cstr!("VK_F"),      /* 0x46 */
    cstr!("VK_G"),      /* 0x47 */
    cstr!("VK_H"),      /* 0x48 */
    cstr!("VK_I"),      /* 0x49 */
    cstr!("VK_J"),      /* 0x4A */
    cstr!("VK_K"),      /* 0x4B */
    cstr!("VK_L"),      /* 0x4C */
    cstr!("VK_M"),      /* 0x4D */
    cstr!("VK_N"),      /* 0x4E */
    cstr!("VK_O"),      /* 0x4F */
    cstr!("VK_P"),      /* 0x50 */
    cstr!("VK_Q"),      /* 0x51 */
    cstr!("VK_R"),      /* 0x52 */
    cstr!("VK_S"),      /* 0x53 */
    cstr!("VK_T"),      /* 0x54 */
    cstr!("VK_U"),      /* 0x55 */
    cstr!("VK_V"),      /* 0x56 */
    cstr!("VK_W"),      /* 0x57 */
    cstr!("VK_X"),      /* 0x58 */
    cstr!("VK_Y"),      /* 0x59 */
    cstr!("VK_Z"),      /* 0x5A */
    cstr!("VK_LWIN"),   /* 0x5B */
    cstr!("VK_RWIN"),   /* 0x5C */
    cstr!("VK_APPS"),   /* 0x5D */
    NULLP,             /* 0x5E */
    cstr!("VK_SLEEP"),  /* 0x5F */
    cstr!("VK_NUMPAD0"),/* 0x60 */
    cstr!("VK_NUMPAD1"),/* 0x61 */
    cstr!("VK_NUMPAD2"),/* 0x62 */
    cstr!("VK_NUMPAD3"),/* 0x63 */
    cstr!("VK_NUMPAD4"),/* 0x64 */
    cstr!("VK_NUMPAD5"),/* 0x65 */
    cstr!("VK_NUMPAD6"),/* 0x66 */
    cstr!("VK_NUMPAD7"),/* 0x67 */
    cstr!("VK_NUMPAD8"),/* 0x68 */
    cstr!("VK_NUMPAD9"),/* 0x69 */
    cstr!("VK_MULTIPLY"),/* 0x6A */
    cstr!("VK_ADD"),    /* 0x6B */
    cstr!("VK_SEPARATOR"),/* 0x6C */
    cstr!("VK_SUBTRACT"),/* 0x6D */
    cstr!("VK_DECIMAL"),/* 0x6E */
    cstr!("VK_DIVIDE"), /* 0x6F */
    cstr!("VK_F1"),     /* 0x70 */
    cstr!("VK_F2"),     /* 0x71 */
    cstr!("VK_F3"),     /* 0x72 */
    cstr!("VK_F4"),     /* 0x73 */
    cstr!("VK_F5"),     /* 0x74 */
    cstr!("VK_F6"),     /* 0x75 */
    cstr!("VK_F7"),     /* 0x76 */
    cstr!("VK_F8"),     /* 0x77 */
    cstr!("VK_F9"),     /* 0x78 */
    cstr!("VK_F10"),    /* 0x79 */
    cstr!("VK_F11"),    /* 0x7A */
    cstr!("VK_F12"),    /* 0x7B */
    cstr!("VK_F13"),    /* 0x7C */
    cstr!("VK_F14"),    /* 0x7D */
    cstr!("VK_F15"),    /* 0x7E */
    cstr!("VK_F16"),    /* 0x7F */
    cstr!("VK_F17"),    /* 0x80 */
    cstr!("VK_F18"),    /* 0x81 */
    cstr!("VK_F19"),    /* 0x82 */
    cstr!("VK_F20"),    /* 0x83 */
    cstr!("VK_F21"),    /* 0x84 */
    cstr!("VK_F22"),    /* 0x85 */
    cstr!("VK_F23"),    /* 0x86 */
    cstr!("VK_F24"),    /* 0x87 */
    NULLP,             /* 0x88 */
    NULLP,             /* 0x89 */
    NULLP,             /* 0x8A */
    NULLP,             /* 0x8B */
    NULLP,             /* 0x8C */
    NULLP,             /* 0x8D */
    NULLP,             /* 0x8E */
    NULLP,             /* 0x8F */
    cstr!("VK_NUMLOCK"),/* 0x90 */
    cstr!("VK_SCROLL"), /* 0x91 */
    cstr!("VK_OEM_NEC_EQUAL"),  /* 0x92 aka. VK_OEM_FJ_JISHO */
    cstr!("VK_OEM_FJ_MASSHOU"), /* 0x93 */
    cstr!("VK_OEM_FJ_TOUROKU"), /* 0x94 */
    cstr!("VK_OEM_FJ_LOYA"),    /* 0x95 */
    cstr!("VK_OEM_FJ_ROYA"),    /* 0x96 */
    NULLP,             /* 0x97 */
    NULLP,             /* 0x98 */
    NULLP,             /* 0x99 */
    NULLP,             /* 0x9A */
    NULLP,             /* 0x9B */
    NULLP,             /* 0x9C */
    NULLP,             /* 0x9D */
    NULLP,             /* 0x9E */
    NULLP,             /* 0x9F */
    cstr!("VK_LSHIFT"),  /* 0xA0 */
    cstr!("VK_RSHIFT"),  /* 0xA1 */
    cstr!("VK_LCONTROL"),/* 0xA2 */
    cstr!("VK_RCONTROL"),/* 0xA3 */
    cstr!("VK_LMENU"),   /* 0xA4 */
    cstr!("VK_RMENU"),   /* 0xA5 */
    cstr!("VK_BROWSER_BACK"),     /* 0xA6 */
    cstr!("VK_BROWSER_FORWARD"),  /* 0xA7 */
    cstr!("VK_BROWSER_REFRESH"),  /* 0xA8 */
    cstr!("VK_BROWSER_STOP"),     /* 0xA9 */
    cstr!("VK_BROWSER_SEARCH"),   /* 0xAA */
    cstr!("VK_BROWSER_FAVORITES"),/* 0xAB */
    cstr!("VK_BROWSER_HOME"),     /* 0xAC */
    cstr!("VK_VOLUME_MUTE"),      /* 0xAD */
    cstr!("VK_VOLUME_DOWN"),      /* 0xAE */
    cstr!("VK_VOLUME_UP"),        /* 0xAF */
    cstr!("VK_MEDIA_NEXT_TRACK"), /* 0xB0 */
    cstr!("VK_MEDIA_PREV_TRACK"), /* 0xB1 */
    cstr!("VK_MEDIA_STOP"),       /* 0xB2 */
    cstr!("VK_MEDIA_PLAY_PAUSE"), /* 0xB3 */
    cstr!("VK_LAUNCH_MAIL"),      /* 0xB4 */
    cstr!("VK_LAUNCH_MEDIA_SELECT"),/* 0xB5 */
    cstr!("VK_LAUNCH_APP1"),      /* 0xB6 */
    cstr!("VK_LAUNCH_APP2"),      /* 0xB7 */
    NULLP,             /* 0xB8 */
    NULLP,             /* 0xB9 */
    cstr!("VK_OEM_1"),    /* 0xBA */
    cstr!("VK_OEM_PLUS"), /* 0xBB */
    cstr!("VK_OEM_COMMA"),/* 0xBC */
    cstr!("VK_OEM_MINUS"),/* 0xBD */
    cstr!("VK_OEM_PERIOD"),/* 0xBE */
    cstr!("VK_OEM_2"),    /* 0xBF */
    cstr!("VK_OEM_3"),    /* 0xC0 */
    NULLP,             /* 0xC1 */
    NULLP,             /* 0xC2 */
    NULLP,             /* 0xC3 */
    NULLP,             /* 0xC4 */
    NULLP,             /* 0xC5 */
    NULLP,             /* 0xC6 */
    NULLP,             /* 0xC7 */
    NULLP,             /* 0xC8 */
    NULLP,             /* 0xC9 */
    NULLP,             /* 0xCA */
    NULLP,             /* 0xCB */
    NULLP,             /* 0xCC */
    NULLP,             /* 0xCD */
    NULLP,             /* 0xCE */
    NULLP,             /* 0xCF */
    NULLP,             /* 0xD0 */
    NULLP,             /* 0xD1 */
    NULLP,             /* 0xD2 */
    NULLP,             /* 0xD3 */
    NULLP,             /* 0xD4 */
    NULLP,             /* 0xD5 */
    NULLP,             /* 0xD6 */
    NULLP,             /* 0xD7 */
    NULLP,             /* 0xD8 */
    NULLP,             /* 0xD9 */
    NULLP,             /* 0xDA */
    cstr!("VK_OEM_4"),  /* 0xDB */
    cstr!("VK_OEM_5"),  /* 0xDC */
    cstr!("VK_OEM_6"),  /* 0xDD */
    cstr!("VK_OEM_7"),  /* 0xDE */
    cstr!("VK_OEM_8"),  /* 0xDF */
    NULLP,             /* 0xE0 */
    cstr!("VK_OEM_AX"), /* 0xE1 */
    cstr!("VK_OEM_102"),/* 0xE2 */
    cstr!("VK_ICO_HELP"),/* 0xE3 */
    cstr!("VK_ICO_00"), /* 0xE4 */
    cstr!("VK_PROCESSKEY"),/* 0xE5 */
    NULLP,             /* 0xE6 */
    cstr!("VK_PACKET"), /* 0xE7 */
    NULLP,             /* 0xE8 */
    NULLP,             /* 0xE9 */
    NULLP,             /* 0xEA */
    NULLP,             /* 0xEB */
    NULLP,             /* 0xEC */
    NULLP,             /* 0xED */
    NULLP,             /* 0xEE */
    NULLP,             /* 0xEF */
    NULLP,             /* 0xF0 */
    NULLP,             /* 0xF1 */
    NULLP,             /* 0xF2 */
    NULLP,             /* 0xF3 */
    NULLP,             /* 0xF4 */
    NULLP,             /* 0xF5 */
    cstr!("VK_ATTN"),   /* 0xF6 */
    cstr!("VK_CRSEL"),  /* 0xF7 */
    cstr!("VK_EXSEL"),  /* 0xF8 */
    cstr!("VK_EREOF"),  /* 0xF9 */
    cstr!("VK_PLAY"),   /* 0xFA */
    cstr!("VK_ZOOM"),   /* 0xFB */
    cstr!("VK_NONAME"), /* 0xFC */
    cstr!("VK_PA1"),    /* 0xFD */
    cstr!("VK_OEM_CLEAR"),/* 0xFE */
    NULLP,             /* 0xFF */
]);

/* ************************************************************************ */
/* WM_USER+n message values for "common controls" */

#[repr(C)]
struct USER_MSG {
    name: *const c_char, /* name of control message           */
    value: UINT,         /* message number (0x0401-0x0fff)    */
    len: UINT,           /* length of space at lParam to dump */
}
unsafe impl Sync for USER_MSG {}

#[repr(C)]
struct CONTROL_CLASS {
    classname: *const WCHAR,    /* class name to match */
    classmsg: *const USER_MSG,  /* pointer to first USER_MSG for class */
    count: UINT,                /* number of entries */
}
unsafe impl Sync for CONTROL_CLASS {}

// `USM(a,b)` => { name: stringify(a), value: a, len: b }. SZOF(a) => sizeof(a) (the
// arrays use 0 or a literal byte count). The control-message-number constants below
// come from psdk/commctrl.h + richedit.h + prsht.h + wine/commctrl.h (RB_GETBANDINFO_OLD
// and TB_UNKWN464 are local #defines in spy.c / wine headers). WM_USER == 1024.
const CBEM_GETCOMBOCONTROL: UINT = WM_USER+6;
const CBEM_GETEDITCONTROL: UINT = WM_USER+7;
const CBEM_GETEXTENDEDSTYLE: UINT = WM_USER+9;
const CBEM_GETIMAGELIST: UINT = WM_USER+3;
const CBEM_GETITEMA: UINT = WM_USER+4;
const CBEM_GETITEMW: UINT = WM_USER+13;
const CBEM_HASEDITCHANGED: UINT = WM_USER+10;
const CBEM_INSERTITEMA: UINT = WM_USER+1;
const CBEM_INSERTITEMW: UINT = WM_USER+11;
const CBEM_SETEXSTYLE: UINT = WM_USER+8;
const CBEM_SETEXTENDEDSTYLE: UINT = WM_USER+14;
const CBEM_SETIMAGELIST: UINT = WM_USER+2;
const CBEM_SETITEMA: UINT = WM_USER+5;
const CBEM_SETITEMW: UINT = WM_USER+12;
const EM_AUTOURLDETECT: UINT = WM_USER+91;
const EM_CANPASTE: UINT = WM_USER+50;
const EM_CANREDO: UINT = WM_USER+85;
const EM_CONVPOSITION: UINT = WM_USER+108;
const EM_DISPLAYBAND: UINT = WM_USER+51;
const EM_EXGETSEL: UINT = WM_USER+52;
const EM_EXLIMITTEXT: UINT = WM_USER+53;
const EM_EXLINEFROMCHAR: UINT = WM_USER+54;
const EM_EXSETSEL: UINT = WM_USER+55;
const EM_FINDTEXT: UINT = WM_USER+56;
const EM_FINDTEXTEX: UINT = WM_USER+79;
const EM_FINDTEXTEXW: UINT = WM_USER+124;
const EM_FINDTEXTW: UINT = WM_USER+123;
const EM_FINDWORDBREAK: UINT = WM_USER+76;
const EM_FORMATRANGE: UINT = WM_USER+57;
const EM_GETAUTOURLDETECT: UINT = WM_USER+92;
const EM_GETBIDIOPTIONS: UINT = WM_USER+201;
const EM_GETCHARFORMAT: UINT = WM_USER+58;
const EM_GETEDITSTYLE: UINT = WM_USER+205;
const EM_GETEVENTMASK: UINT = WM_USER+59;
const EM_GETIMECOLOR: UINT = WM_USER+105;
const EM_GETIMECOMPMODE: UINT = WM_USER+122;
const EM_GETIMEMODEBIAS: UINT = WM_USER+127;
const EM_GETIMEOPTIONS: UINT = WM_USER+107;
const EM_GETLANGOPTIONS: UINT = WM_USER+121;
const EM_GETOLEINTERFACE: UINT = WM_USER+60;
const EM_GETOPTIONS: UINT = WM_USER+78;
const EM_GETPARAFORMAT: UINT = WM_USER+61;
const EM_GETPUNCTUATION: UINT = WM_USER+101;
const EM_GETREDONAME: UINT = WM_USER+87;
const EM_GETSCROLLPOS: UINT = WM_USER+221;
const EM_GETSELTEXT: UINT = WM_USER+62;
const EM_GETTEXTEX: UINT = WM_USER+94;
const EM_GETTEXTLENGTHEX: UINT = WM_USER+95;
const EM_GETTEXTMODE: UINT = WM_USER+90;
const EM_GETTEXTRANGE: UINT = WM_USER+75;
const EM_GETTYPOGRAPHYOPTIONS: UINT = WM_USER+203;
const EM_GETUNDONAME: UINT = WM_USER+86;
const EM_GETWORDBREAKPROCEX: UINT = WM_USER+80;
const EM_GETWORDWRAPMODE: UINT = WM_USER+103;
const EM_GETZOOM: UINT = WM_USER+224;
const EM_HIDESELECTION: UINT = WM_USER+63;
const EM_OUTLINE: UINT = WM_USER+220;
const EM_PASTESPECIAL: UINT = WM_USER+64;
const EM_RECONVERSION: UINT = WM_USER+125;
const EM_REDO: UINT = WM_USER+84;
const EM_REQUESTRESIZE: UINT = WM_USER+65;
const EM_SELECTIONTYPE: UINT = WM_USER+66;
const EM_SETBIDIOPTIONS: UINT = WM_USER+200;
const EM_SETBKGNDCOLOR: UINT = WM_USER+67;
const EM_SETCHARFORMAT: UINT = WM_USER+68;
const EM_SETEDITSTYLE: UINT = WM_USER+204;
const EM_SETEVENTMASK: UINT = WM_USER+69;
const EM_SETFONTSIZE: UINT = WM_USER+223;
const EM_SETIMECOLOR: UINT = WM_USER+104;
const EM_SETIMEMODEBIAS: UINT = WM_USER+126;
const EM_SETIMEOPTIONS: UINT = WM_USER+106;
const EM_SETLANGOPTIONS: UINT = WM_USER+120;
const EM_SETOLECALLBACK: UINT = WM_USER+70;
const EM_SETOPTIONS: UINT = WM_USER+77;
const EM_SETPALETTE: UINT = WM_USER+93;
const EM_SETPARAFORMAT: UINT = WM_USER+71;
const EM_SETPUNCTUATION: UINT = WM_USER+100;
const EM_SETSCROLLPOS: UINT = WM_USER+222;
const EM_SETTARGETDEVICE: UINT = WM_USER+72;
const EM_SETTEXTEX: UINT = WM_USER+97;
const EM_SETTEXTMODE: UINT = WM_USER+89;
const EM_SETTYPOGRAPHYOPTIONS: UINT = WM_USER+202;
const EM_SETUNDOLIMIT: UINT = WM_USER+82;
const EM_SETWORDBREAKPROCEX: UINT = WM_USER+81;
const EM_SETWORDWRAPMODE: UINT = WM_USER+102;
const EM_SETZOOM: UINT = WM_USER+225;
const EM_SHOWSCROLLBAR: UINT = WM_USER+96;
const EM_STOPGROUPTYPING: UINT = WM_USER+88;
const EM_STREAMIN: UINT = WM_USER+73;
const EM_STREAMOUT: UINT = WM_USER+74;
const PSM_ADDPAGE: UINT = WM_USER+103;
const PSM_APPLY: UINT = WM_USER+110;
const PSM_CANCELTOCLOSE: UINT = WM_USER+107;
const PSM_CHANGED: UINT = WM_USER+104;
const PSM_GETCURRENTPAGEHWND: UINT = 1142;
const PSM_GETTABCONTROL: UINT = 1140;
const PSM_ISDIALOGMESSAGE: UINT = 1141;
const PSM_PRESSBUTTON: UINT = 1137;
const PSM_QUERYSIBLINGS: UINT = WM_USER+108;
const PSM_REBOOTSYSTEM: UINT = WM_USER+106;
const PSM_REMOVEPAGE: UINT = WM_USER+102;
const PSM_RESTARTWINDOWS: UINT = WM_USER+105;
const PSM_SETCURSEL: UINT = WM_USER+101;
const PSM_SETCURSELID: UINT = 1138;
const PSM_SETFINISHTEXTA: UINT = 1139;
const PSM_SETFINISHTEXTW: UINT = 1145;
const PSM_SETTITLEA: UINT = WM_USER+111;
const PSM_SETTITLEW: UINT = WM_USER+120;
const PSM_SETWIZBUTTONS: UINT = WM_USER+112;
const PSM_UNCHANGED: UINT = WM_USER+109;
const RB_BEGINDRAG: UINT = WM_USER+24;
const RB_DELETEBAND: UINT = WM_USER+2;
const RB_DRAGMOVE: UINT = WM_USER+26;
const RB_ENDDRAG: UINT = WM_USER+25;
const RB_GETBANDBORDERS: UINT = WM_USER+34;
const RB_GETBANDCOUNT: UINT = WM_USER+12;
const RB_GETBANDINFOA: UINT = WM_USER+29;
const RB_GETBANDINFOW: UINT = WM_USER+28;
const RB_GETBANDINFO_OLD: UINT = WM_USER + 5;
const RB_GETBARHEIGHT: UINT = WM_USER+27;
const RB_GETBARINFO: UINT = WM_USER+3;
const RB_GETBKCOLOR: UINT = WM_USER+20;
const RB_GETPALETTE: UINT = WM_USER+38;
const RB_GETRECT: UINT = WM_USER+9;
const RB_GETROWCOUNT: UINT = WM_USER+13;
const RB_GETROWHEIGHT: UINT = WM_USER+14;
const RB_GETTEXTCOLOR: UINT = WM_USER+22;
const RB_GETTOOLTIPS: UINT = WM_USER+17;
const RB_HITTEST: UINT = WM_USER+8;
const RB_IDTOINDEX: UINT = WM_USER+16;
const RB_INSERTBANDA: UINT = WM_USER+1;
const RB_INSERTBANDW: UINT = WM_USER+10;
const RB_MAXIMIZEBAND: UINT = WM_USER+31;
const RB_MINIMIZEBAND: UINT = WM_USER+30;
const RB_MOVEBAND: UINT = WM_USER+39;
const RB_SETBANDINFOA: UINT = WM_USER+6;
const RB_SETBANDINFOW: UINT = WM_USER+11;
const RB_SETBARINFO: UINT = WM_USER+4;
const RB_SETBKCOLOR: UINT = WM_USER+19;
const RB_SETPALETTE: UINT = WM_USER+37;
const RB_SETPARENT: UINT = WM_USER+7;
const RB_SETTEXTCOLOR: UINT = WM_USER+21;
const RB_SETTOOLTIPS: UINT = WM_USER+18;
const RB_SHOWBAND: UINT = WM_USER+35;
const RB_SIZETORECT: UINT = WM_USER+23;
const TB_ADDBITMAP: UINT = WM_USER+19;
const TB_ADDBUTTONSA: UINT = WM_USER+20;
const TB_ADDBUTTONSW: UINT = WM_USER+68;
const TB_ADDSTRINGA: UINT = WM_USER+28;
const TB_ADDSTRINGW: UINT = WM_USER+77;
const TB_AUTOSIZE: UINT = WM_USER+33;
const TB_BUTTONCOUNT: UINT = WM_USER+24;
const TB_BUTTONSTRUCTSIZE: UINT = WM_USER+30;
const TB_CHANGEBITMAP: UINT = WM_USER+43;
const TB_CHECKBUTTON: UINT = WM_USER+2;
const TB_COMMANDTOINDEX: UINT = WM_USER+25;
const TB_CUSTOMIZE: UINT = WM_USER+27;
const TB_DELETEBUTTON: UINT = WM_USER+22;
const TB_ENABLEBUTTON: UINT = WM_USER+1;
const TB_GETANCHORHIGHLIGHT: UINT = WM_USER+74;
const TB_GETBITMAP: UINT = WM_USER+44;
const TB_GETBITMAPFLAGS: UINT = WM_USER+41;
const TB_GETBUTTON: UINT = WM_USER+23;
const TB_GETBUTTONINFOA: UINT = WM_USER+65;
const TB_GETBUTTONINFOW: UINT = WM_USER+63;
const TB_GETBUTTONSIZE: UINT = WM_USER+58;
const TB_GETBUTTONTEXTA: UINT = WM_USER+45;
const TB_GETBUTTONTEXTW: UINT = WM_USER+75;
const TB_GETDISABLEDIMAGELIST: UINT = WM_USER+55;
const TB_GETEXTENDEDSTYLE: UINT = WM_USER+85;
const TB_GETHOTIMAGELIST: UINT = WM_USER+53;
const TB_GETHOTITEM: UINT = WM_USER+71;
const TB_GETIDEALSIZE: UINT = WM_USER+99;
const TB_GETIMAGELIST: UINT = WM_USER+49;
const TB_GETIMAGELISTCOUNT: UINT = WM_USER+98;
const TB_GETINSERTMARK: UINT = WM_USER+79;
const TB_GETINSERTMARKCOLOR: UINT = WM_USER+89;
const TB_GETITEMRECT: UINT = WM_USER+29;
const TB_GETMAXSIZE: UINT = WM_USER+83;
const TB_GETOBJECT: UINT = WM_USER+62;
const TB_GETPADDING: UINT = WM_USER+86;
const TB_GETRECT: UINT = WM_USER+51;
const TB_GETROWS: UINT = WM_USER+40;
const TB_GETSTATE: UINT = WM_USER+18;
const TB_GETSTRINGA: UINT = WM_USER+92;
const TB_GETSTRINGW: UINT = WM_USER+91;
const TB_GETSTYLE: UINT = WM_USER+57;
const TB_GETTEXTROWS: UINT = WM_USER+61;
const TB_GETTOOLTIPS: UINT = WM_USER+35;
const TB_HASACCELERATOR: UINT = WM_USER+95;
const TB_HIDEBUTTON: UINT = WM_USER+4;
const TB_HITTEST: UINT = WM_USER+69;
const TB_INDETERMINATE: UINT = WM_USER+5;
const TB_INSERTBUTTONA: UINT = WM_USER+21;
const TB_INSERTBUTTONW: UINT = WM_USER+67;
const TB_INSERTMARKHITTEST: UINT = WM_USER+81;
const TB_ISBUTTONCHECKED: UINT = WM_USER+10;
const TB_ISBUTTONENABLED: UINT = WM_USER+9;
const TB_ISBUTTONHIDDEN: UINT = WM_USER+12;
const TB_ISBUTTONHIGHLIGHTED: UINT = WM_USER+14;
const TB_ISBUTTONINDETERMINATE: UINT = WM_USER+13;
const TB_ISBUTTONPRESSED: UINT = WM_USER+11;
const TB_LOADIMAGES: UINT = WM_USER+50;
const TB_MAPACCELERATORA: UINT = WM_USER+78;
const TB_MAPACCELERATORW: UINT = WM_USER+90;
const TB_MARKBUTTON: UINT = WM_USER+6;
const TB_MOVEBUTTON: UINT = WM_USER+82;
const TB_PRESSBUTTON: UINT = WM_USER+3;
const TB_REPLACEBITMAP: UINT = WM_USER+46;
const TB_SAVERESTOREA: UINT = WM_USER+26;
const TB_SAVERESTOREW: UINT = WM_USER+76;
const TB_SETANCHORHIGHLIGHT: UINT = WM_USER+73;
const TB_SETBITMAPSIZE: UINT = WM_USER+32;
const TB_SETBOUNDINGSIZE: UINT = WM_USER+93;
const TB_SETBUTTONINFOA: UINT = WM_USER+66;
const TB_SETBUTTONINFOW: UINT = WM_USER+64;
const TB_SETBUTTONSIZE: UINT = WM_USER+31;
const TB_SETBUTTONWIDTH: UINT = WM_USER+59;
const TB_SETCMDID: UINT = WM_USER+42;
const TB_SETDISABLEDIMAGELIST: UINT = WM_USER+54;
const TB_SETDRAWTEXTFLAGS: UINT = WM_USER+70;
const TB_SETEXTENDEDSTYLE: UINT = WM_USER+84;
const TB_SETHOTIMAGELIST: UINT = WM_USER+52;
const TB_SETHOTITEM: UINT = WM_USER+72;
const TB_SETHOTITEM2: UINT = WM_USER+94;
const TB_SETIMAGELIST: UINT = WM_USER+48;
const TB_SETINDENT: UINT = WM_USER+47;
const TB_SETINSERTMARK: UINT = WM_USER+80;
const TB_SETINSERTMARKCOLOR: UINT = WM_USER+88;
const TB_SETLISTGAP: UINT = WM_USER+96;
const TB_SETMAXTEXTROWS: UINT = WM_USER+60;
const TB_SETPADDING: UINT = WM_USER+87;
const TB_SETPARENT: UINT = WM_USER+37;
const TB_SETROWS: UINT = WM_USER+39;
const TB_SETSTATE: UINT = WM_USER+17;
const TB_SETSTYLE: UINT = WM_USER+56;
const TB_SETTOOLTIPS: UINT = WM_USER+36;
const TB_UNKWN464: UINT = WM_USER + 100;
const TTM_ACTIVATE: UINT = WM_USER+1;
const TTM_ADDTOOLA: UINT = WM_USER+4;
const TTM_ADDTOOLW: UINT = WM_USER+50;
const TTM_DELTOOLA: UINT = WM_USER+5;
const TTM_DELTOOLW: UINT = WM_USER+51;
const TTM_ENUMTOOLSA: UINT = WM_USER+14;
const TTM_ENUMTOOLSW: UINT = WM_USER+58;
const TTM_GETBUBBLESIZE: UINT = WM_USER+30;
const TTM_GETCURRENTTOOLA: UINT = WM_USER+15;
const TTM_GETCURRENTTOOLW: UINT = WM_USER+59;
const TTM_GETDELAYTIME: UINT = WM_USER+21;
const TTM_GETMARGIN: UINT = WM_USER+27;
const TTM_GETMAXTIPWIDTH: UINT = WM_USER+25;
const TTM_GETTEXTA: UINT = WM_USER+11;
const TTM_GETTEXTW: UINT = WM_USER+56;
const TTM_GETTIPBKCOLOR: UINT = WM_USER+22;
const TTM_GETTIPTEXTCOLOR: UINT = WM_USER+23;
const TTM_GETTOOLCOUNT: UINT = WM_USER+13;
const TTM_GETTOOLINFOA: UINT = WM_USER+8;
const TTM_GETTOOLINFOW: UINT = WM_USER+53;
const TTM_HITTESTA: UINT = WM_USER+10;
const TTM_HITTESTW: UINT = WM_USER+55;
const TTM_NEWTOOLRECTA: UINT = WM_USER+6;
const TTM_NEWTOOLRECTW: UINT = WM_USER+52;
const TTM_POP: UINT = WM_USER+28;
const TTM_RELAYEVENT: UINT = WM_USER+7;
const TTM_SETDELAYTIME: UINT = WM_USER+3;
const TTM_SETMARGIN: UINT = WM_USER+26;
const TTM_SETMAXTIPWIDTH: UINT = WM_USER+24;
const TTM_SETTIPBKCOLOR: UINT = WM_USER+19;
const TTM_SETTIPTEXTCOLOR: UINT = WM_USER+20;
const TTM_SETTOOLINFOW: UINT = WM_USER+54;
const TTM_TRACKACTIVATE: UINT = WM_USER+17;
const TTM_TRACKPOSITION: UINT = WM_USER+18;
const TTM_UPDATE: UINT = WM_USER+29;
const TTM_UPDATETIPTEXTA: UINT = WM_USER+12;
const TTM_UPDATETIPTEXTW: UINT = WM_USER+57;
const TTM_WINDOWFROMPOINT: UINT = WM_USER+16;
const UDM_GETACCEL: UINT = WM_USER+108;
const UDM_GETBASE: UINT = WM_USER+110;
const UDM_GETBUDDY: UINT = WM_USER+106;
const UDM_GETPOS: UINT = WM_USER+104;
const UDM_GETPOS32: UINT = WM_USER+114;
const UDM_GETRANGE: UINT = WM_USER+102;
const UDM_GETRANGE32: UINT = WM_USER+112;
const UDM_SETACCEL: UINT = WM_USER+107;
const UDM_SETBASE: UINT = WM_USER+109;
const UDM_SETBUDDY: UINT = WM_USER+105;
const UDM_SETPOS: UINT = WM_USER+103;
const UDM_SETPOS32: UINT = WM_USER+113;
const UDM_SETRANGE: UINT = WM_USER+101;
const UDM_SETRANGE32: UINT = WM_USER+111;

// const fn helper for the USM(...) macro.
const fn usm(name: *const c_char, value: UINT, len: UINT) -> USER_MSG {
    USER_MSG { name, value, len }
}

// RB_GETBANDINFO_OLD already defined above. The arrays below mirror the C `static const
// USER_MSG xxx_array[]` definitions verbatim (faithful order — required for bsearch).
static rebar_array: &[USER_MSG] = &[
    usm(cstr!("RB_INSERTBANDA"), RB_INSERTBANDA, 0),
    usm(cstr!("RB_DELETEBAND"), RB_DELETEBAND, 0),
    usm(cstr!("RB_GETBARINFO"), RB_GETBARINFO, 0),
    usm(cstr!("RB_SETBARINFO"), RB_SETBARINFO, 0),
    usm(cstr!("RB_GETBANDINFO_OLD"), RB_GETBANDINFO_OLD, 0),
    usm(cstr!("RB_SETBANDINFOA"), RB_SETBANDINFOA, 0),
    usm(cstr!("RB_SETPARENT"), RB_SETPARENT, 0),
    usm(cstr!("RB_HITTEST"), RB_HITTEST, 0),
    usm(cstr!("RB_GETRECT"), RB_GETRECT, 0),
    usm(cstr!("RB_INSERTBANDW"), RB_INSERTBANDW, 0),
    usm(cstr!("RB_SETBANDINFOW"), RB_SETBANDINFOW, 0),
    usm(cstr!("RB_GETBANDCOUNT"), RB_GETBANDCOUNT, 0),
    usm(cstr!("RB_GETROWCOUNT"), RB_GETROWCOUNT, 0),
    usm(cstr!("RB_GETROWHEIGHT"), RB_GETROWHEIGHT, 0),
    usm(cstr!("RB_IDTOINDEX"), RB_IDTOINDEX, 0),
    usm(cstr!("RB_GETTOOLTIPS"), RB_GETTOOLTIPS, 0),
    usm(cstr!("RB_SETTOOLTIPS"), RB_SETTOOLTIPS, 0),
    usm(cstr!("RB_SETBKCOLOR"), RB_SETBKCOLOR, 0),
    usm(cstr!("RB_GETBKCOLOR"), RB_GETBKCOLOR, 0),
    usm(cstr!("RB_SETTEXTCOLOR"), RB_SETTEXTCOLOR, 0),
    usm(cstr!("RB_GETTEXTCOLOR"), RB_GETTEXTCOLOR, 0),
    usm(cstr!("RB_SIZETORECT"), RB_SIZETORECT, 0),
    usm(cstr!("RB_BEGINDRAG"), RB_BEGINDRAG, 0),
    usm(cstr!("RB_ENDDRAG"), RB_ENDDRAG, 0),
    usm(cstr!("RB_DRAGMOVE"), RB_DRAGMOVE, 0),
    usm(cstr!("RB_GETBARHEIGHT"), RB_GETBARHEIGHT, 0),
    usm(cstr!("RB_GETBANDINFOW"), RB_GETBANDINFOW, 0),
    usm(cstr!("RB_GETBANDINFOA"), RB_GETBANDINFOA, 0),
    usm(cstr!("RB_MINIMIZEBAND"), RB_MINIMIZEBAND, 0),
    usm(cstr!("RB_MAXIMIZEBAND"), RB_MAXIMIZEBAND, 0),
    usm(cstr!("RB_GETBANDBORDERS"), RB_GETBANDBORDERS, 0),
    usm(cstr!("RB_SHOWBAND"), RB_SHOWBAND, 0),
    usm(cstr!("RB_SETPALETTE"), RB_SETPALETTE, 0),
    usm(cstr!("RB_GETPALETTE"), RB_GETPALETTE, 0),
    usm(cstr!("RB_MOVEBAND"), RB_MOVEBAND, 0),
];

static toolbar_array: &[USER_MSG] = &[
    usm(cstr!("TB_ENABLEBUTTON"), TB_ENABLEBUTTON, 0),
    usm(cstr!("TB_CHECKBUTTON"), TB_CHECKBUTTON, 0),
    usm(cstr!("TB_PRESSBUTTON"), TB_PRESSBUTTON, 0),
    usm(cstr!("TB_HIDEBUTTON"), TB_HIDEBUTTON, 0),
    usm(cstr!("TB_INDETERMINATE"), TB_INDETERMINATE, 0),
    usm(cstr!("TB_MARKBUTTON"), TB_MARKBUTTON, 0),
    usm(cstr!("TB_ISBUTTONENABLED"), TB_ISBUTTONENABLED, 0),
    usm(cstr!("TB_ISBUTTONCHECKED"), TB_ISBUTTONCHECKED, 0),
    usm(cstr!("TB_ISBUTTONPRESSED"), TB_ISBUTTONPRESSED, 0),
    usm(cstr!("TB_ISBUTTONHIDDEN"), TB_ISBUTTONHIDDEN, 0),
    usm(cstr!("TB_ISBUTTONINDETERMINATE"), TB_ISBUTTONINDETERMINATE, 0),
    usm(cstr!("TB_ISBUTTONHIGHLIGHTED"), TB_ISBUTTONHIGHLIGHTED, 0),
    usm(cstr!("TB_SETSTATE"), TB_SETSTATE, 0),
    usm(cstr!("TB_GETSTATE"), TB_GETSTATE, 0),
    usm(cstr!("TB_ADDBITMAP"), TB_ADDBITMAP, 0),
    usm(cstr!("TB_ADDBUTTONSA"), TB_ADDBUTTONSA, 0),
    usm(cstr!("TB_INSERTBUTTONA"), TB_INSERTBUTTONA, 0),
    usm(cstr!("TB_DELETEBUTTON"), TB_DELETEBUTTON, 0),
    usm(cstr!("TB_GETBUTTON"), TB_GETBUTTON, 0),
    usm(cstr!("TB_BUTTONCOUNT"), TB_BUTTONCOUNT, 0),
    usm(cstr!("TB_COMMANDTOINDEX"), TB_COMMANDTOINDEX, 0),
    usm(cstr!("TB_SAVERESTOREA"), TB_SAVERESTOREA, 0),
    usm(cstr!("TB_CUSTOMIZE"), TB_CUSTOMIZE, 0),
    usm(cstr!("TB_ADDSTRINGA"), TB_ADDSTRINGA, 0),
    usm(cstr!("TB_GETITEMRECT"), TB_GETITEMRECT, 0),
    usm(cstr!("TB_BUTTONSTRUCTSIZE"), TB_BUTTONSTRUCTSIZE, 0),
    usm(cstr!("TB_SETBUTTONSIZE"), TB_SETBUTTONSIZE, 0),
    usm(cstr!("TB_SETBITMAPSIZE"), TB_SETBITMAPSIZE, 0),
    usm(cstr!("TB_AUTOSIZE"), TB_AUTOSIZE, 0),
    usm(cstr!("TB_GETTOOLTIPS"), TB_GETTOOLTIPS, 0),
    usm(cstr!("TB_SETTOOLTIPS"), TB_SETTOOLTIPS, 0),
    usm(cstr!("TB_SETPARENT"), TB_SETPARENT, 0),
    usm(cstr!("TB_SETROWS"), TB_SETROWS, 0),
    usm(cstr!("TB_GETROWS"), TB_GETROWS, 0),
    usm(cstr!("TB_GETBITMAPFLAGS"), TB_GETBITMAPFLAGS, 0),
    usm(cstr!("TB_SETCMDID"), TB_SETCMDID, 0),
    usm(cstr!("TB_CHANGEBITMAP"), TB_CHANGEBITMAP, 0),
    usm(cstr!("TB_GETBITMAP"), TB_GETBITMAP, 0),
    usm(cstr!("TB_GETBUTTONTEXTA"), TB_GETBUTTONTEXTA, 0),
    usm(cstr!("TB_REPLACEBITMAP"), TB_REPLACEBITMAP, 0),
    usm(cstr!("TB_SETINDENT"), TB_SETINDENT, 0),
    usm(cstr!("TB_SETIMAGELIST"), TB_SETIMAGELIST, 0),
    usm(cstr!("TB_GETIMAGELIST"), TB_GETIMAGELIST, 0),
    usm(cstr!("TB_LOADIMAGES"), TB_LOADIMAGES, 0),
    usm(cstr!("TB_GETRECT"), TB_GETRECT, 0),
    usm(cstr!("TB_SETHOTIMAGELIST"), TB_SETHOTIMAGELIST, 0),
    usm(cstr!("TB_GETHOTIMAGELIST"), TB_GETHOTIMAGELIST, 0),
    usm(cstr!("TB_SETDISABLEDIMAGELIST"), TB_SETDISABLEDIMAGELIST, 0),
    usm(cstr!("TB_GETDISABLEDIMAGELIST"), TB_GETDISABLEDIMAGELIST, 0),
    usm(cstr!("TB_SETSTYLE"), TB_SETSTYLE, 0),
    usm(cstr!("TB_GETSTYLE"), TB_GETSTYLE, 0),
    usm(cstr!("TB_GETBUTTONSIZE"), TB_GETBUTTONSIZE, 0),
    usm(cstr!("TB_SETBUTTONWIDTH"), TB_SETBUTTONWIDTH, 0),
    usm(cstr!("TB_SETMAXTEXTROWS"), TB_SETMAXTEXTROWS, 0),
    usm(cstr!("TB_GETTEXTROWS"), TB_GETTEXTROWS, 0),
    usm(cstr!("TB_GETOBJECT"), TB_GETOBJECT, 0),
    usm(cstr!("TB_GETBUTTONINFOW"), TB_GETBUTTONINFOW, 0),
    usm(cstr!("TB_SETBUTTONINFOW"), TB_SETBUTTONINFOW, 0),
    usm(cstr!("TB_GETBUTTONINFOA"), TB_GETBUTTONINFOA, 0),
    usm(cstr!("TB_SETBUTTONINFOA"), TB_SETBUTTONINFOA, 0),
    usm(cstr!("TB_INSERTBUTTONW"), TB_INSERTBUTTONW, 0),
    usm(cstr!("TB_ADDBUTTONSW"), TB_ADDBUTTONSW, 0),
    usm(cstr!("TB_HITTEST"), TB_HITTEST, 0),
    usm(cstr!("TB_SETDRAWTEXTFLAGS"), TB_SETDRAWTEXTFLAGS, 0),
    usm(cstr!("TB_GETHOTITEM"), TB_GETHOTITEM, 0),
    usm(cstr!("TB_SETHOTITEM"), TB_SETHOTITEM, 0),
    usm(cstr!("TB_SETANCHORHIGHLIGHT"), TB_SETANCHORHIGHLIGHT, 0),
    usm(cstr!("TB_GETANCHORHIGHLIGHT"), TB_GETANCHORHIGHLIGHT, 0),
    usm(cstr!("TB_GETBUTTONTEXTW"), TB_GETBUTTONTEXTW, 0),
    usm(cstr!("TB_SAVERESTOREW"), TB_SAVERESTOREW, 0),
    usm(cstr!("TB_ADDSTRINGW"), TB_ADDSTRINGW, 0),
    usm(cstr!("TB_MAPACCELERATORA"), TB_MAPACCELERATORA, 0),
    usm(cstr!("TB_GETINSERTMARK"), TB_GETINSERTMARK, 0),
    usm(cstr!("TB_SETINSERTMARK"), TB_SETINSERTMARK, 0),
    usm(cstr!("TB_INSERTMARKHITTEST"), TB_INSERTMARKHITTEST, 0),
    usm(cstr!("TB_MOVEBUTTON"), TB_MOVEBUTTON, 0),
    usm(cstr!("TB_GETMAXSIZE"), TB_GETMAXSIZE, 0),
    usm(cstr!("TB_SETEXTENDEDSTYLE"), TB_SETEXTENDEDSTYLE, 0),
    usm(cstr!("TB_GETEXTENDEDSTYLE"), TB_GETEXTENDEDSTYLE, 0),
    usm(cstr!("TB_GETPADDING"), TB_GETPADDING, 0),
    usm(cstr!("TB_SETPADDING"), TB_SETPADDING, 0),
    usm(cstr!("TB_SETINSERTMARKCOLOR"), TB_SETINSERTMARKCOLOR, 0),
    usm(cstr!("TB_GETINSERTMARKCOLOR"), TB_GETINSERTMARKCOLOR, 0),
    usm(cstr!("TB_MAPACCELERATORW"), TB_MAPACCELERATORW, 0),
    usm(cstr!("TB_GETSTRINGW"), TB_GETSTRINGW, 0),
    usm(cstr!("TB_GETSTRINGA"), TB_GETSTRINGA, 0),
    usm(cstr!("TB_SETBOUNDINGSIZE"), TB_SETBOUNDINGSIZE, 8),
    usm(cstr!("TB_SETHOTITEM2"), TB_SETHOTITEM2, 0),
    usm(cstr!("TB_HASACCELERATOR"), TB_HASACCELERATOR, 0),
    usm(cstr!("TB_SETLISTGAP"), TB_SETLISTGAP, 0),
    usm(cstr!("TB_GETIMAGELISTCOUNT"), TB_GETIMAGELISTCOUNT, 0),
    usm(cstr!("TB_GETIDEALSIZE"), TB_GETIDEALSIZE, 8),
    usm(cstr!("TB_UNKWN464"), TB_UNKWN464, 0),
];

static tooltips_array: &[USER_MSG] = &[
    usm(cstr!("TTM_ACTIVATE"), TTM_ACTIVATE, 0),
    usm(cstr!("TTM_SETDELAYTIME"), TTM_SETDELAYTIME, 0),
    usm(cstr!("TTM_ADDTOOLA"), TTM_ADDTOOLA, 0),
    usm(cstr!("TTM_DELTOOLA"), TTM_DELTOOLA, 0),
    usm(cstr!("TTM_NEWTOOLRECTA"), TTM_NEWTOOLRECTA, 0),
    usm(cstr!("TTM_RELAYEVENT"), TTM_RELAYEVENT, 0),
    usm(cstr!("TTM_GETTOOLINFOA"), TTM_GETTOOLINFOA, 0),
    usm(cstr!("TTM_HITTESTA"), TTM_HITTESTA, 0),
    usm(cstr!("TTM_GETTEXTA"), TTM_GETTEXTA, 0),
    usm(cstr!("TTM_UPDATETIPTEXTA"), TTM_UPDATETIPTEXTA, 0),
    usm(cstr!("TTM_GETTOOLCOUNT"), TTM_GETTOOLCOUNT, 0),
    usm(cstr!("TTM_ENUMTOOLSA"), TTM_ENUMTOOLSA, 0),
    usm(cstr!("TTM_GETCURRENTTOOLA"), TTM_GETCURRENTTOOLA, 0),
    usm(cstr!("TTM_WINDOWFROMPOINT"), TTM_WINDOWFROMPOINT, 0),
    usm(cstr!("TTM_TRACKACTIVATE"), TTM_TRACKACTIVATE, 0),
    usm(cstr!("TTM_TRACKPOSITION"), TTM_TRACKPOSITION, 0),
    usm(cstr!("TTM_SETTIPBKCOLOR"), TTM_SETTIPBKCOLOR, 0),
    usm(cstr!("TTM_SETTIPTEXTCOLOR"), TTM_SETTIPTEXTCOLOR, 0),
    usm(cstr!("TTM_GETDELAYTIME"), TTM_GETDELAYTIME, 0),
    usm(cstr!("TTM_GETTIPBKCOLOR"), TTM_GETTIPBKCOLOR, 0),
    usm(cstr!("TTM_GETTIPTEXTCOLOR"), TTM_GETTIPTEXTCOLOR, 0),
    usm(cstr!("TTM_SETMAXTIPWIDTH"), TTM_SETMAXTIPWIDTH, 0),
    usm(cstr!("TTM_GETMAXTIPWIDTH"), TTM_GETMAXTIPWIDTH, 0),
    usm(cstr!("TTM_SETMARGIN"), TTM_SETMARGIN, 0),
    usm(cstr!("TTM_GETMARGIN"), TTM_GETMARGIN, 0),
    usm(cstr!("TTM_POP"), TTM_POP, 0),
    usm(cstr!("TTM_UPDATE"), TTM_UPDATE, 0),
    usm(cstr!("TTM_GETBUBBLESIZE"), TTM_GETBUBBLESIZE, 0),
    usm(cstr!("TTM_ADDTOOLW"), TTM_ADDTOOLW, 0),
    usm(cstr!("TTM_DELTOOLW"), TTM_DELTOOLW, 0),
    usm(cstr!("TTM_NEWTOOLRECTW"), TTM_NEWTOOLRECTW, 0),
    usm(cstr!("TTM_GETTOOLINFOW"), TTM_GETTOOLINFOW, 0),
    usm(cstr!("TTM_SETTOOLINFOW"), TTM_SETTOOLINFOW, 0),
    usm(cstr!("TTM_HITTESTW"), TTM_HITTESTW, 0),
    usm(cstr!("TTM_GETTEXTW"), TTM_GETTEXTW, 0),
    usm(cstr!("TTM_UPDATETIPTEXTW"), TTM_UPDATETIPTEXTW, 0),
    usm(cstr!("TTM_ENUMTOOLSW"), TTM_ENUMTOOLSW, 0),
    usm(cstr!("TTM_GETCURRENTTOOLW"), TTM_GETCURRENTTOOLW, 0),
];

static comboex_array: &[USER_MSG] = &[
    usm(cstr!("CBEM_INSERTITEMA"), CBEM_INSERTITEMA, 0),
    usm(cstr!("CBEM_SETIMAGELIST"), CBEM_SETIMAGELIST, 0),
    usm(cstr!("CBEM_GETIMAGELIST"), CBEM_GETIMAGELIST, 0),
    usm(cstr!("CBEM_GETITEMA"), CBEM_GETITEMA, 0),
    usm(cstr!("CBEM_SETITEMA"), CBEM_SETITEMA, 0),
    usm(cstr!("CBEM_GETCOMBOCONTROL"), CBEM_GETCOMBOCONTROL, 0),
    usm(cstr!("CBEM_GETEDITCONTROL"), CBEM_GETEDITCONTROL, 0),
    usm(cstr!("CBEM_SETEXSTYLE"), CBEM_SETEXSTYLE, 0),
    usm(cstr!("CBEM_GETEXTENDEDSTYLE"), CBEM_GETEXTENDEDSTYLE, 0),
    usm(cstr!("CBEM_HASEDITCHANGED"), CBEM_HASEDITCHANGED, 0),
    usm(cstr!("CBEM_INSERTITEMW"), CBEM_INSERTITEMW, 0),
    usm(cstr!("CBEM_SETITEMW"), CBEM_SETITEMW, 0),
    usm(cstr!("CBEM_GETITEMW"), CBEM_GETITEMW, 0),
    usm(cstr!("CBEM_SETEXTENDEDSTYLE"), CBEM_SETEXTENDEDSTYLE, 0),
];

static propsht_array: &[USER_MSG] = &[
    usm(cstr!("PSM_SETCURSEL"), PSM_SETCURSEL, 0),
    usm(cstr!("PSM_REMOVEPAGE"), PSM_REMOVEPAGE, 0),
    usm(cstr!("PSM_ADDPAGE"), PSM_ADDPAGE, 0),
    usm(cstr!("PSM_CHANGED"), PSM_CHANGED, 0),
    usm(cstr!("PSM_RESTARTWINDOWS"), PSM_RESTARTWINDOWS, 0),
    usm(cstr!("PSM_REBOOTSYSTEM"), PSM_REBOOTSYSTEM, 0),
    usm(cstr!("PSM_CANCELTOCLOSE"), PSM_CANCELTOCLOSE, 0),
    usm(cstr!("PSM_QUERYSIBLINGS"), PSM_QUERYSIBLINGS, 0),
    usm(cstr!("PSM_UNCHANGED"), PSM_UNCHANGED, 0),
    usm(cstr!("PSM_APPLY"), PSM_APPLY, 0),
    usm(cstr!("PSM_SETTITLEA"), PSM_SETTITLEA, 0),
    usm(cstr!("PSM_SETWIZBUTTONS"), PSM_SETWIZBUTTONS, 0),
    usm(cstr!("PSM_PRESSBUTTON"), PSM_PRESSBUTTON, 0),
    usm(cstr!("PSM_SETCURSELID"), PSM_SETCURSELID, 0),
    usm(cstr!("PSM_SETFINISHTEXTA"), PSM_SETFINISHTEXTA, 0),
    usm(cstr!("PSM_GETTABCONTROL"), PSM_GETTABCONTROL, 0),
    usm(cstr!("PSM_ISDIALOGMESSAGE"), PSM_ISDIALOGMESSAGE, 0),
    usm(cstr!("PSM_GETCURRENTPAGEHWND"), PSM_GETCURRENTPAGEHWND, 0),
    usm(cstr!("PSM_SETTITLEW"), PSM_SETTITLEW, 0),
    usm(cstr!("PSM_SETFINISHTEXTW"), PSM_SETFINISHTEXTW, 0),
];

static updown_array: &[USER_MSG] = &[
    usm(cstr!("UDM_SETRANGE"), UDM_SETRANGE, 0),
    usm(cstr!("UDM_GETRANGE"), UDM_GETRANGE, 0),
    usm(cstr!("UDM_SETPOS"), UDM_SETPOS, 0),
    usm(cstr!("UDM_GETPOS"), UDM_GETPOS, 0),
    usm(cstr!("UDM_SETBUDDY"), UDM_SETBUDDY, 0),
    usm(cstr!("UDM_GETBUDDY"), UDM_GETBUDDY, 0),
    usm(cstr!("UDM_SETACCEL"), UDM_SETACCEL, 0),
    usm(cstr!("UDM_GETACCEL"), UDM_GETACCEL, 0),
    usm(cstr!("UDM_SETBASE"), UDM_SETBASE, 0),
    usm(cstr!("UDM_GETBASE"), UDM_GETBASE, 0),
    usm(cstr!("UDM_SETRANGE32"), UDM_SETRANGE32, 0),
    usm(cstr!("UDM_GETRANGE32"), UDM_GETRANGE32, 0),
    usm(cstr!("UDM_SETPOS32"), UDM_SETPOS32, 0),
    usm(cstr!("UDM_GETPOS32"), UDM_GETPOS32, 0),
];

static richedit_array: &[USER_MSG] = &[
    USER_MSG { name: cstr!("EM_SCROLLCARET"), value: WM_USER + 49, len: 0 },
    usm(cstr!("EM_CANPASTE"), EM_CANPASTE, 0),
    usm(cstr!("EM_DISPLAYBAND"), EM_DISPLAYBAND, 0),
    usm(cstr!("EM_EXGETSEL"), EM_EXGETSEL, 0),
    usm(cstr!("EM_EXLIMITTEXT"), EM_EXLIMITTEXT, 0),
    usm(cstr!("EM_EXLINEFROMCHAR"), EM_EXLINEFROMCHAR, 0),
    usm(cstr!("EM_EXSETSEL"), EM_EXSETSEL, 0),
    usm(cstr!("EM_FINDTEXT"), EM_FINDTEXT, 0),
    usm(cstr!("EM_FORMATRANGE"), EM_FORMATRANGE, 0),
    usm(cstr!("EM_GETCHARFORMAT"), EM_GETCHARFORMAT, 0),
    usm(cstr!("EM_GETEVENTMASK"), EM_GETEVENTMASK, 0),
    usm(cstr!("EM_GETOLEINTERFACE"), EM_GETOLEINTERFACE, 0),
    usm(cstr!("EM_GETPARAFORMAT"), EM_GETPARAFORMAT, 0),
    usm(cstr!("EM_GETSELTEXT"), EM_GETSELTEXT, 0),
    usm(cstr!("EM_HIDESELECTION"), EM_HIDESELECTION, 0),
    usm(cstr!("EM_PASTESPECIAL"), EM_PASTESPECIAL, 0),
    usm(cstr!("EM_REQUESTRESIZE"), EM_REQUESTRESIZE, 0),
    usm(cstr!("EM_SELECTIONTYPE"), EM_SELECTIONTYPE, 0),
    usm(cstr!("EM_SETBKGNDCOLOR"), EM_SETBKGNDCOLOR, 0),
    usm(cstr!("EM_SETCHARFORMAT"), EM_SETCHARFORMAT, 0),
    usm(cstr!("EM_SETEVENTMASK"), EM_SETEVENTMASK, 0),
    usm(cstr!("EM_SETOLECALLBACK"), EM_SETOLECALLBACK, 0),
    usm(cstr!("EM_SETPARAFORMAT"), EM_SETPARAFORMAT, 0),
    usm(cstr!("EM_SETTARGETDEVICE"), EM_SETTARGETDEVICE, 0),
    usm(cstr!("EM_STREAMIN"), EM_STREAMIN, 0),
    usm(cstr!("EM_STREAMOUT"), EM_STREAMOUT, 0),
    usm(cstr!("EM_GETTEXTRANGE"), EM_GETTEXTRANGE, 0),
    usm(cstr!("EM_FINDWORDBREAK"), EM_FINDWORDBREAK, 0),
    usm(cstr!("EM_SETOPTIONS"), EM_SETOPTIONS, 0),
    usm(cstr!("EM_GETOPTIONS"), EM_GETOPTIONS, 0),
    usm(cstr!("EM_FINDTEXTEX"), EM_FINDTEXTEX, 0),
    usm(cstr!("EM_GETWORDBREAKPROCEX"), EM_GETWORDBREAKPROCEX, 0),
    usm(cstr!("EM_SETWORDBREAKPROCEX"), EM_SETWORDBREAKPROCEX, 0),
    usm(cstr!("EM_SETUNDOLIMIT"), EM_SETUNDOLIMIT, 0),
    usm(cstr!("EM_REDO"), EM_REDO, 0),
    usm(cstr!("EM_CANREDO"), EM_CANREDO, 0),
    usm(cstr!("EM_GETUNDONAME"), EM_GETUNDONAME, 0),
    usm(cstr!("EM_GETREDONAME"), EM_GETREDONAME, 0),
    usm(cstr!("EM_STOPGROUPTYPING"), EM_STOPGROUPTYPING, 0),
    usm(cstr!("EM_SETTEXTMODE"), EM_SETTEXTMODE, 0),
    usm(cstr!("EM_GETTEXTMODE"), EM_GETTEXTMODE, 0),
    usm(cstr!("EM_AUTOURLDETECT"), EM_AUTOURLDETECT, 0),
    usm(cstr!("EM_GETAUTOURLDETECT"), EM_GETAUTOURLDETECT, 0),
    usm(cstr!("EM_SETPALETTE"), EM_SETPALETTE, 0),
    usm(cstr!("EM_GETTEXTEX"), EM_GETTEXTEX, 0),
    usm(cstr!("EM_GETTEXTLENGTHEX"), EM_GETTEXTLENGTHEX, 0),
    usm(cstr!("EM_SHOWSCROLLBAR"), EM_SHOWSCROLLBAR, 0),
    usm(cstr!("EM_SETTEXTEX"), EM_SETTEXTEX, 0),
    usm(cstr!("EM_SETPUNCTUATION"), EM_SETPUNCTUATION, 0),
    usm(cstr!("EM_GETPUNCTUATION"), EM_GETPUNCTUATION, 0),
    usm(cstr!("EM_SETWORDWRAPMODE"), EM_SETWORDWRAPMODE, 0),
    usm(cstr!("EM_GETWORDWRAPMODE"), EM_GETWORDWRAPMODE, 0),
    usm(cstr!("EM_SETIMECOLOR"), EM_SETIMECOLOR, 0),
    usm(cstr!("EM_GETIMECOLOR"), EM_GETIMECOLOR, 0),
    usm(cstr!("EM_SETIMEOPTIONS"), EM_SETIMEOPTIONS, 0),
    usm(cstr!("EM_GETIMEOPTIONS"), EM_GETIMEOPTIONS, 0),
    usm(cstr!("EM_CONVPOSITION"), EM_CONVPOSITION, 0),
    usm(cstr!("EM_SETLANGOPTIONS"), EM_SETLANGOPTIONS, 0),
    usm(cstr!("EM_GETLANGOPTIONS"), EM_GETLANGOPTIONS, 0),
    usm(cstr!("EM_GETIMECOMPMODE"), EM_GETIMECOMPMODE, 0),
    usm(cstr!("EM_FINDTEXTW"), EM_FINDTEXTW, 0),
    usm(cstr!("EM_FINDTEXTEXW"), EM_FINDTEXTEXW, 0),
    usm(cstr!("EM_RECONVERSION"), EM_RECONVERSION, 0),
    usm(cstr!("EM_SETIMEMODEBIAS"), EM_SETIMEMODEBIAS, 0),
    usm(cstr!("EM_GETIMEMODEBIAS"), EM_GETIMEMODEBIAS, 0),
    usm(cstr!("EM_SETBIDIOPTIONS"), EM_SETBIDIOPTIONS, 0),
    usm(cstr!("EM_GETBIDIOPTIONS"), EM_GETBIDIOPTIONS, 0),
    usm(cstr!("EM_SETTYPOGRAPHYOPTIONS"), EM_SETTYPOGRAPHYOPTIONS, 0),
    usm(cstr!("EM_GETTYPOGRAPHYOPTIONS"), EM_GETTYPOGRAPHYOPTIONS, 0),
    usm(cstr!("EM_SETEDITSTYLE"), EM_SETEDITSTYLE, 0),
    usm(cstr!("EM_GETEDITSTYLE"), EM_GETEDITSTYLE, 0),
    usm(cstr!("EM_OUTLINE"), EM_OUTLINE, 0),
    usm(cstr!("EM_GETSCROLLPOS"), EM_GETSCROLLPOS, 0),
    usm(cstr!("EM_SETSCROLLPOS"), EM_SETSCROLLPOS, 0),
    usm(cstr!("EM_SETFONTSIZE"), EM_SETFONTSIZE, 0),
    usm(cstr!("EM_GETZOOM"), EM_GETZOOM, 0),
    usm(cstr!("EM_SETZOOM"), EM_SETZOOM, 0),
];


/* ************************************************************************ */
/* Class-name match strings (psdk/commctrl.h, richedit.h, prsht.h) as wide L"" */
//   WC_COMBOBOXEXW  = L"ComboBoxEx32"
//   WC_PROPSHEETW   = L"SysPropertySheet"
//   REBARCLASSNAMEW = L"ReBarWindow32"
//   TOOLBARCLASSNAMEW = L"ToolbarWindow32"
//   TOOLTIPS_CLASSW = L"tooltips_class32"
//   UPDOWN_CLASSW   = L"msctls_updown32"
//   RICHEDIT_CLASS20W = L"RichEdit20W"
// Built as &[u16] arrays (NUL-terminated) per the wide-string-encoding guidance.
static WSTR_COMBOBOXEX: &[u16] = &[
    0x43, 0x6F, 0x6D, 0x62, 0x6F, 0x42, 0x6F, 0x78, 0x45, 0x78, 0x33, 0x32, 0,
]; // "ComboBoxEx32"
static WSTR_PROPSHEET: &[u16] = &[
    0x53, 0x79, 0x73, 0x50, 0x72, 0x6F, 0x70, 0x65, 0x72, 0x74, 0x79, 0x53, 0x68, 0x65, 0x65, 0x74,
    0,
]; // "SysPropertySheet"
static WSTR_REBAR: &[u16] = &[
    0x52, 0x65, 0x42, 0x61, 0x72, 0x57, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x33, 0x32, 0,
]; // "ReBarWindow32"
static WSTR_TOOLBAR: &[u16] = &[
    0x54, 0x6F, 0x6F, 0x6C, 0x62, 0x61, 0x72, 0x57, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x33, 0x32, 0,
]; // "ToolbarWindow32"
static WSTR_TOOLTIPS: &[u16] = &[
    0x74, 0x6F, 0x6F, 0x6C, 0x74, 0x69, 0x70, 0x73, 0x5F, 0x63, 0x6C, 0x61, 0x73, 0x73, 0x33, 0x32,
    0,
]; // "tooltips_class32"
static WSTR_UPDOWN: &[u16] = &[
    0x6D, 0x73, 0x63, 0x74, 0x6C, 0x73, 0x5F, 0x75, 0x70, 0x64, 0x6F, 0x77, 0x6E, 0x33, 0x32, 0,
]; // "msctls_updown32"
static WSTR_RICHEDIT20: &[u16] = &[
    0x52, 0x69, 0x63, 0x68, 0x45, 0x64, 0x69, 0x74, 0x32, 0x30, 0x57, 0,
]; // "RichEdit20W"

// `PropSheetInfoStr[] = L"PropertySheetInfo"` (static, file-scope).
static PropSheetInfoStr: [u16; 18] = [
    0x50, 0x72, 0x6F, 0x70, 0x65, 0x72, 0x74, 0x79, 0x53, 0x68, 0x65, 0x65, 0x74, 0x49, 0x6E, 0x66,
    0x6F, 0,
]; // "PropertySheetInfo"

static cc_array: &[CONTROL_CLASS] = &[
    CONTROL_CLASS { classname: WSTR_COMBOBOXEX.as_ptr(), classmsg: comboex_array.as_ptr(), count: comboex_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_PROPSHEET.as_ptr(), classmsg: propsht_array.as_ptr(), count: propsht_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_REBAR.as_ptr(), classmsg: rebar_array.as_ptr(), count: rebar_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_TOOLBAR.as_ptr(), classmsg: toolbar_array.as_ptr(), count: toolbar_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_TOOLTIPS.as_ptr(), classmsg: tooltips_array.as_ptr(), count: tooltips_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_UPDOWN.as_ptr(), classmsg: updown_array.as_ptr(), count: updown_array.len() as UINT },
    CONTROL_CLASS { classname: WSTR_RICHEDIT20.as_ptr(), classmsg: richedit_array.as_ptr(), count: richedit_array.len() as UINT },
    // C terminator {0,0,0}: a null classname stops the SPY_GetMsgStuff scan.
    CONTROL_CLASS { classname: core::ptr::null(), classmsg: core::ptr::null(), count: 0 },
];

/* ************************************************************************ */
/* WM_NOTIFY function codes display */

#[repr(C)]
struct SPY_NOTIFY {
    name: *const c_char, /* name of notify message        */
    value: UINT,         /* notify code value             */
    len: UINT,           /* length of extra space to dump */
}
unsafe impl Sync for SPY_NOTIFY {}

// `SPNFY(a,b)` => { #a, a, sizeof(b)-sizeof(NMHDR) }. The `len` (extra dump bytes) is
// consumed only by the dead SPY_DumpStructure/WM_NOTIFY path (DEBUG_SPY off, TRACE off),
// so the per-struct sizeof deltas are not material at runtime and are recorded as 0.
// `value` (the notify code) IS faithful: each XXX_FIRST base is `0u32.wrapping_sub(k)`
// and the code is `base - n` (also wrapping). Array MUST stay in descending order by
// `value` (the C bsearch requires it) — order preserved verbatim from the source.
const fn spnfy(name: *const c_char, value: UINT) -> SPY_NOTIFY {
    SPY_NOTIFY { name, value, len: 0 }
}

static spnfy_array: &[SPY_NOTIFY] = &[
    spnfy(cstr!("NM_OUTOFMEMORY"), 0u32.wrapping_sub(1)),
    spnfy(cstr!("NM_CLICK"), 0u32.wrapping_sub(2)),
    spnfy(cstr!("NM_DBLCLK"), 0u32.wrapping_sub(3)),
    spnfy(cstr!("NM_RETURN"), 0u32.wrapping_sub(4)),
    spnfy(cstr!("NM_RCLICK"), 0u32.wrapping_sub(5)),
    spnfy(cstr!("NM_RDBLCLK"), 0u32.wrapping_sub(6)),
    spnfy(cstr!("NM_SETFOCUS"), 0u32.wrapping_sub(7)),
    spnfy(cstr!("NM_KILLFOCUS"), 0u32.wrapping_sub(8)),
    spnfy(cstr!("NM_CUSTOMDRAW"), 0u32.wrapping_sub(12)),
    spnfy(cstr!("NM_HOVER"), 0u32.wrapping_sub(13)),
    spnfy(cstr!("NM_NCHITTEST"), 0u32.wrapping_sub(14)),
    spnfy(cstr!("NM_KEYDOWN"), 0u32.wrapping_sub(15)),
    spnfy(cstr!("NM_RELEASEDCAPTURE"), 0u32.wrapping_sub(16)),
    spnfy(cstr!("NM_SETCURSOR"), 0u32.wrapping_sub(17)),
    spnfy(cstr!("NM_CHAR"), 0u32.wrapping_sub(18)),
    spnfy(cstr!("NM_TOOLTIPSCREATED"), 0u32.wrapping_sub(19)),
    spnfy(cstr!("LVN_ITEMCHANGING"), 0u32.wrapping_sub(100)),
    spnfy(cstr!("LVN_ITEMCHANGED"), 0u32.wrapping_sub(101)),
    spnfy(cstr!("LVN_INSERTITEM"), 0u32.wrapping_sub(102)),
    spnfy(cstr!("LVN_DELETEITEM"), 0u32.wrapping_sub(103)),
    spnfy(cstr!("LVN_DELETEALLITEMS"), 0u32.wrapping_sub(104)),
    spnfy(cstr!("LVN_BEGINLABELEDITA"), 0u32.wrapping_sub(105)),
    spnfy(cstr!("LVN_ENDLABELEDITA"), 0u32.wrapping_sub(106)),
    spnfy(cstr!("LVN_COLUMNCLICK"), 0u32.wrapping_sub(108)),
    spnfy(cstr!("LVN_BEGINDRAG"), 0u32.wrapping_sub(109)),
    spnfy(cstr!("LVN_BEGINRDRAG"), 0u32.wrapping_sub(111)),
    spnfy(cstr!("LVN_ODCACHEHINT"), 0u32.wrapping_sub(113)),
    spnfy(cstr!("LVN_ITEMACTIVATE"), 0u32.wrapping_sub(114)),
    spnfy(cstr!("LVN_ODSTATECHANGED"), 0u32.wrapping_sub(115)),
    spnfy(cstr!("LVN_HOTTRACK"), 0u32.wrapping_sub(121)),
    spnfy(cstr!("LVN_GETDISPINFOA"), 0u32.wrapping_sub(150)),
    spnfy(cstr!("LVN_SETDISPINFOA"), 0u32.wrapping_sub(151)),
    spnfy(cstr!("LVN_ODFINDITEMA"), 0u32.wrapping_sub(152)),
    spnfy(cstr!("LVN_KEYDOWN"), 0u32.wrapping_sub(155)),
    spnfy(cstr!("LVN_MARQUEEBEGIN"), 0u32.wrapping_sub(156)),
    spnfy(cstr!("LVN_GETINFOTIPA"), 0u32.wrapping_sub(157)),
    spnfy(cstr!("LVN_GETINFOTIPW"), 0u32.wrapping_sub(158)),
    spnfy(cstr!("LVN_BEGINLABELEDITW"), 0u32.wrapping_sub(175)),
    spnfy(cstr!("LVN_ENDLABELEDITW"), 0u32.wrapping_sub(176)),
    spnfy(cstr!("LVN_GETDISPINFOW"), 0u32.wrapping_sub(177)),
    spnfy(cstr!("LVN_SETDISPINFOW"), 0u32.wrapping_sub(178)),
    spnfy(cstr!("LVN_ODFINDITEMW"), 0u32.wrapping_sub(179)),
    spnfy(cstr!("PSN_SETACTIVE"), 0u32.wrapping_sub(200)),
    spnfy(cstr!("PSN_KILLACTIVE"), 0u32.wrapping_sub(201)),
    spnfy(cstr!("PSN_APPLY"), 0u32.wrapping_sub(202)),
    spnfy(cstr!("PSN_RESET"), 0u32.wrapping_sub(203)),
    spnfy(cstr!("PSN_HELP"), 0u32.wrapping_sub(205)),
    spnfy(cstr!("PSN_WIZBACK"), 0u32.wrapping_sub(206)),
    spnfy(cstr!("PSN_WIZNEXT"), 0u32.wrapping_sub(207)),
    spnfy(cstr!("PSN_WIZFINISH"), 0u32.wrapping_sub(208)),
    spnfy(cstr!("PSN_QUERYCANCEL"), 0u32.wrapping_sub(209)),
    spnfy(cstr!("PSN_GETOBJECT"), 0u32.wrapping_sub(210)),
    spnfy(cstr!("PSN_TRANSLATEACCELERATOR"), 0u32.wrapping_sub(212)),
    spnfy(cstr!("PSN_QUERYINITIALFOCUS"), 0u32.wrapping_sub(213)),
    spnfy(cstr!("HDN_ITEMCHANGINGA"), 0u32.wrapping_sub(300)),
    spnfy(cstr!("HDN_ITEMCHANGEDA"), 0u32.wrapping_sub(301)),
    spnfy(cstr!("HDN_ITEMCLICKA"), 0u32.wrapping_sub(302)),
    spnfy(cstr!("HDN_ITEMDBLCLICKA"), 0u32.wrapping_sub(303)),
    spnfy(cstr!("HDN_DIVIDERDBLCLICKA"), 0u32.wrapping_sub(305)),
    spnfy(cstr!("HDN_BEGINTRACKA"), 0u32.wrapping_sub(306)),
    spnfy(cstr!("HDN_ENDTRACKA"), 0u32.wrapping_sub(307)),
    spnfy(cstr!("HDN_TRACKA"), 0u32.wrapping_sub(308)),
    spnfy(cstr!("HDN_GETDISPINFOA"), 0u32.wrapping_sub(309)),
    spnfy(cstr!("HDN_BEGINDRAG"), 0u32.wrapping_sub(310)),
    spnfy(cstr!("HDN_ENDDRAG"), 0u32.wrapping_sub(311)),
    spnfy(cstr!("HDN_ITEMCHANGINGW"), 0u32.wrapping_sub(320)),
    spnfy(cstr!("HDN_ITEMCHANGEDW"), 0u32.wrapping_sub(321)),
    spnfy(cstr!("HDN_ITEMCLICKW"), 0u32.wrapping_sub(322)),
    spnfy(cstr!("HDN_ITEMDBLCLICKW"), 0u32.wrapping_sub(323)),
    spnfy(cstr!("HDN_DIVIDERDBLCLICKW"), 0u32.wrapping_sub(325)),
    spnfy(cstr!("HDN_BEGINTRACKW"), 0u32.wrapping_sub(326)),
    spnfy(cstr!("HDN_ENDTRACKW"), 0u32.wrapping_sub(327)),
    spnfy(cstr!("HDN_TRACKW"), 0u32.wrapping_sub(328)),
    spnfy(cstr!("HDN_GETDISPINFOW"), 0u32.wrapping_sub(329)),
    spnfy(cstr!("TVN_SELCHANGINGA"), 0u32.wrapping_sub(401)),
    spnfy(cstr!("TVN_SELCHANGEDA"), 0u32.wrapping_sub(402)),
    spnfy(cstr!("TVN_GETDISPINFOA"), 0u32.wrapping_sub(403)),
    spnfy(cstr!("TVN_SETDISPINFOA"), 0u32.wrapping_sub(404)),
    spnfy(cstr!("TVN_ITEMEXPANDINGA"), 0u32.wrapping_sub(405)),
    spnfy(cstr!("TVN_ITEMEXPANDEDA"), 0u32.wrapping_sub(406)),
    spnfy(cstr!("TVN_BEGINDRAGA"), 0u32.wrapping_sub(407)),
    spnfy(cstr!("TVN_BEGINRDRAGA"), 0u32.wrapping_sub(408)),
    spnfy(cstr!("TVN_DELETEITEMA"), 0u32.wrapping_sub(409)),
    spnfy(cstr!("TVN_BEGINLABELEDITA"), 0u32.wrapping_sub(410)),
    spnfy(cstr!("TVN_ENDLABELEDITA"), 0u32.wrapping_sub(411)),
    spnfy(cstr!("TVN_KEYDOWN"), 0u32.wrapping_sub(412)),
    spnfy(cstr!("TVN_SELCHANGINGW"), 0u32.wrapping_sub(450)),
    spnfy(cstr!("TVN_SELCHANGEDW"), 0u32.wrapping_sub(451)),
    spnfy(cstr!("TVN_GETDISPINFOW"), 0u32.wrapping_sub(452)),
    spnfy(cstr!("TVN_SETDISPINFOW"), 0u32.wrapping_sub(453)),
    spnfy(cstr!("TVN_ITEMEXPANDINGW"), 0u32.wrapping_sub(454)),
    spnfy(cstr!("TVN_ITEMEXPANDEDW"), 0u32.wrapping_sub(455)),
    spnfy(cstr!("TVN_BEGINDRAGW"), 0u32.wrapping_sub(456)),
    spnfy(cstr!("TVN_BEGINRDRAGW"), 0u32.wrapping_sub(457)),
    spnfy(cstr!("TVN_DELETEITEMW"), 0u32.wrapping_sub(458)),
    spnfy(cstr!("TVN_BEGINLABELEDITW"), 0u32.wrapping_sub(459)),
    spnfy(cstr!("TVN_ENDLABELEDITW"), 0u32.wrapping_sub(460)),
    spnfy(cstr!("TTN_GETDISPINFOA"), 0u32.wrapping_sub(520)),
    spnfy(cstr!("TTN_SHOW"), 0u32.wrapping_sub(521)),
    spnfy(cstr!("TTN_POP"), 0u32.wrapping_sub(522)),
    spnfy(cstr!("TTN_GETDISPINFOW"), 0u32.wrapping_sub(530)),
    spnfy(cstr!("TCN_KEYDOWN"), 0u32.wrapping_sub(550)),
    spnfy(cstr!("TCN_SELCHANGE"), 0u32.wrapping_sub(551)),
    spnfy(cstr!("TCN_SELCHANGING"), 0u32.wrapping_sub(552)),
    spnfy(cstr!("TCN_GETOBJECT"), 0u32.wrapping_sub(553)),
    spnfy(cstr!("CDN_INITDONE"), 0u32.wrapping_sub(601)),
    spnfy(cstr!("CDN_SELCHANGE"), 0u32.wrapping_sub(602)),
    spnfy(cstr!("CDN_FOLDERCHANGE"), 0u32.wrapping_sub(603)),
    spnfy(cstr!("CDN_SHAREVIOLATION"), 0u32.wrapping_sub(604)),
    spnfy(cstr!("CDN_HELP"), 0u32.wrapping_sub(605)),
    spnfy(cstr!("CDN_FILEOK"), 0u32.wrapping_sub(606)),
    spnfy(cstr!("CDN_TYPECHANGE"), 0u32.wrapping_sub(607)),
    spnfy(cstr!("TBN_GETBUTTONINFOA"), 0u32.wrapping_sub(700)),
    spnfy(cstr!("TBN_BEGINDRAG"), 0u32.wrapping_sub(701)),
    spnfy(cstr!("TBN_ENDDRAG"), 0u32.wrapping_sub(702)),
    spnfy(cstr!("TBN_BEGINADJUST"), 0u32.wrapping_sub(703)),
    spnfy(cstr!("TBN_ENDADJUST"), 0u32.wrapping_sub(704)),
    spnfy(cstr!("TBN_RESET"), 0u32.wrapping_sub(705)),
    spnfy(cstr!("TBN_QUERYINSERT"), 0u32.wrapping_sub(706)),
    spnfy(cstr!("TBN_QUERYDELETE"), 0u32.wrapping_sub(707)),
    spnfy(cstr!("TBN_TOOLBARCHANGE"), 0u32.wrapping_sub(708)),
    spnfy(cstr!("TBN_CUSTHELP"), 0u32.wrapping_sub(709)),
    spnfy(cstr!("TBN_DROPDOWN"), 0u32.wrapping_sub(710)),
    spnfy(cstr!("TBN_GETOBJECT"), 0u32.wrapping_sub(712)),
    spnfy(cstr!("TBN_HOTITEMCHANGE"), 0u32.wrapping_sub(713)),
    spnfy(cstr!("TBN_DRAGOUT"), 0u32.wrapping_sub(714)),
    spnfy(cstr!("TBN_DELETINGBUTTON"), 0u32.wrapping_sub(715)),
    spnfy(cstr!("TBN_GETDISPINFOA"), 0u32.wrapping_sub(716)),
    spnfy(cstr!("TBN_GETDISPINFOW"), 0u32.wrapping_sub(717)),
    spnfy(cstr!("TBN_GETINFOTIPA"), 0u32.wrapping_sub(718)),
    spnfy(cstr!("TBN_GETINFOTIPW"), 0u32.wrapping_sub(719)),
    spnfy(cstr!("TBN_GETBUTTONINFOW"), 0u32.wrapping_sub(720)),
    spnfy(cstr!("UDN_DELTAPOS"), 0u32.wrapping_sub(722)),
    spnfy(cstr!("DTN_FORMATQUERYW"), 0u32.wrapping_sub(778)),
    spnfy(cstr!("DTN_FORMATW"), 0u32.wrapping_sub(777)),
    spnfy(cstr!("DTN_WMKEYDOWNW"), 0u32.wrapping_sub(776)),
    spnfy(cstr!("DTN_USERSTRINGW"), 0u32.wrapping_sub(775)),
    spnfy(cstr!("MCN_SELECT"), 0u32.wrapping_sub(754)),
    spnfy(cstr!("MCN_GETDAYSTATE"), 0u32.wrapping_sub(753)),
    spnfy(cstr!("MCN_SELCHANGE"), 0u32.wrapping_sub(751)),
    spnfy(cstr!("DTN_CLOSEUP"), 0u32.wrapping_sub(767)),
    spnfy(cstr!("DTN_DROPDOWN"), 0u32.wrapping_sub(766)),
    spnfy(cstr!("DTN_FORMATQUERYA"), 0u32.wrapping_sub(765)),
    spnfy(cstr!("DTN_FORMATA"), 0u32.wrapping_sub(764)),
    spnfy(cstr!("DTN_WMKEYDOWNA"), 0u32.wrapping_sub(763)),
    spnfy(cstr!("DTN_USERSTRINGA"), 0u32.wrapping_sub(762)),
    spnfy(cstr!("DTN_DATETIMECHANGE"), 0u32.wrapping_sub(761)),
    spnfy(cstr!("CBEN_GETDISPINFOA"), 0u32.wrapping_sub(800)),
    spnfy(cstr!("CBEN_INSERTITEM"), 0u32.wrapping_sub(801)),
    spnfy(cstr!("CBEN_DELETEITEM"), 0u32.wrapping_sub(802)),
    spnfy(cstr!("CBEN_BEGINEDIT"), 0u32.wrapping_sub(804)),
    spnfy(cstr!("CBEN_ENDEDITA"), 0u32.wrapping_sub(805)),
    spnfy(cstr!("CBEN_ENDEDITW"), 0u32.wrapping_sub(806)),
    spnfy(cstr!("CBEN_GETDISPINFOW"), 0u32.wrapping_sub(807)),
    spnfy(cstr!("CBEN_DRAGBEGINA"), 0u32.wrapping_sub(808)),
    spnfy(cstr!("CBEN_DRAGBEGINW"), 0u32.wrapping_sub(809)),
    spnfy(cstr!("RBN_HEIGHTCHANGE"), 0u32.wrapping_sub(831)),
    spnfy(cstr!("RBN_GETOBJECT"), 0u32.wrapping_sub(832)),
    spnfy(cstr!("RBN_LAYOUTCHANGED"), 0u32.wrapping_sub(833)),
    spnfy(cstr!("RBN_AUTOSIZE"), 0u32.wrapping_sub(834)),
    spnfy(cstr!("RBN_BEGINDRAG"), 0u32.wrapping_sub(835)),
    spnfy(cstr!("RBN_ENDDRAG"), 0u32.wrapping_sub(836)),
    spnfy(cstr!("RBN_DELETINGBAND"), 0u32.wrapping_sub(837)),
    spnfy(cstr!("RBN_DELETEDBAND"), 0u32.wrapping_sub(838)),
    spnfy(cstr!("RBN_CHILDSIZE"), 0u32.wrapping_sub(839)),
    spnfy(cstr!("IPN_FIELDCHANGED"), 0u32.wrapping_sub(860)),
    spnfy(cstr!("SBN_SIMPLEMODECHANGE"), 0u32.wrapping_sub(880)),
    spnfy(cstr!("PGN_SCROLL"), 0u32.wrapping_sub(901)),
    spnfy(cstr!("PGN_CALCSIZE"), 0u32.wrapping_sub(902)),
];

// ───────────────────────────────────────────────────────────────────────────────
// Substrate / cross-DLL imports.
// ───────────────────────────────────────────────────────────────────────────────

// user32-rust client APIs (other TUs, WINAPI = stdcall on i386 → extern "system").
extern "system" {
    fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR;
    fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn InternalGetWindowText(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT;
    fn IsWindowUnicode(hWnd: HWND) -> BOOL;
}

// kernel32 (WINAPI = stdcall on i386 → extern "system").
extern "system" {
    fn GetLastError() -> DWORD;
    fn SetLastError(dwErrCode: DWORD);
    fn GetCurrentThreadId() -> DWORD;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: LPDWORD) -> DWORD;
    fn TlsAlloc() -> DWORD;
    fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    fn TlsGetValue(dwTlsIndex: DWORD) -> LPVOID;
    fn TlsSetValue(dwTlsIndex: DWORD, lpTlsValue: LPVOID) -> BOOL;
    fn GlobalGetAtomNameA(nAtom: ATOM, lpBuffer: LPSTR, nSize: INT) -> UINT;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn RegOpenKeyA(hKey: HKEY, lpSubKey: LPCSTR, phkResult: PHKEY) -> LONG;
    fn RegQueryValueExA(
        hKey: HKEY,
        lpValueName: LPCSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LONG;
    fn RegCloseKey(hKey: HKEY) -> LONG;
    fn InterlockedCompareExchange(
        Destination: *mut LONG,
        Exchange: LONG,
        Comparand: LONG,
    ) -> LONG;
    fn InterlockedCompareExchangePointer(
        Destination: *mut PVOID,
        Exchange: PVOID,
        Comparand: PVOID,
    ) -> PVOID;
}

// CRT (msvcrt-style, cdecl).
extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> INT;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn atoi(s: *const c_char) -> INT;
    fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> INT;
    fn wcslen(s: *const WCHAR) -> usize;
}

const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;
const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY;
// WM_APP / WM_USER come from win32k_sys::win32k.

// ───────────────────────────────────────────────────────────────────────────────
// Wide-string helpers (ndk inline macros strcpyW / strcmpiW / strcmpW).
// ───────────────────────────────────────────────────────────────────────────────
#[inline]
unsafe fn strcpyW(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR {
    let mut d = dst;
    let mut s = src;
    loop {
        *d = *s;
        if *s == 0 {
            break;
        }
        d = d.add(1);
        s = s.add(1);
    }
    dst
}
#[inline]
unsafe fn strcmpW(s1: *const WCHAR, s2: *const WCHAR) -> INT {
    let mut a = s1;
    let mut b = s2;
    while *a != 0 && *a == *b {
        a = a.add(1);
        b = b.add(1);
    }
    (*a as INT) - (*b as INT)
}
#[inline]
unsafe fn towlower(c: WCHAR) -> WCHAR {
    if (0x41..=0x5A).contains(&c) {
        c + 0x20
    } else {
        c
    }
}
#[inline]
unsafe fn strcmpiW(s1: *const WCHAR, s2: *const WCHAR) -> INT {
    let mut a = s1;
    let mut b = s2;
    loop {
        let ca = towlower(*a);
        let cb = towlower(*b);
        if ca != cb {
            return (ca as INT) - (cb as INT);
        }
        if ca == 0 {
            return 0;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

// `min` (windef.h macro).
#[inline]
fn min_uint(a: UINT, b: UINT) -> UINT {
    if a < b {
        a
    } else {
        b
    }
}

// ───────────────────────────────────────────────────────────────────────────────
// SPY_INSTANCE + module statics
// ───────────────────────────────────────────────────────────────────────────────
#[repr(C)]
struct SPY_INSTANCE {
    msgnum: UINT,            /* message number */
    msg_hwnd: HWND,          /* window handle for message */
    wParam: WPARAM,          /* message parameter */
    lParam: LPARAM,          /* message parameter */
    data_len: INT,           /* length of data to dump */
    msg_name: [c_char; 60],  /* message name (see SPY_GetMsgName) */
    wnd_class: [WCHAR; 60],  /* window class name (full) */
    wnd_name: [WCHAR; 16],   /* window name for message */
}

// `static unsigned char *spy_exclude;` — set once, then read-only.
static mut spy_exclude: *mut u8 = core::ptr::null_mut();
// `static int indent_tls_index = TLS_OUT_OF_INDEXES;`
static mut indent_tls_index: INT = TLS_OUT_OF_INDEXES as INT;

// ───────────────────────────────────────────────────────────────────────────────
// TRACE_ON(message): the WINE message debug channel. It is OFF in this build, so the
// spy is effectively compiled out — every TRACE is a no-op and spy_init() returns FALSE.
// ───────────────────────────────────────────────────────────────────────────────
#[inline]
fn TRACE_ON_message() -> BOOL {
    0 // message channel disabled
}

#[inline]
unsafe fn exclude_msg(msg: UINT) -> BOOL {
    *spy_exclude.add(min_uint(msg, SPY_MAX_MSGNUM) as usize) as BOOL
}

#[inline]
unsafe fn exclude_dwp() -> BOOL {
    *spy_exclude.add((SPY_MAX_MSGNUM + 1) as usize) as BOOL
}

/***********************************************************************
 *           get_indent_level
 */
#[inline]
unsafe fn get_indent_level() -> INT_PTR {
    TlsGetValue(indent_tls_index as DWORD) as INT_PTR
}

/***********************************************************************
 *           set_indent_level
 */
#[inline]
unsafe fn set_indent_level(level: INT_PTR) {
    TlsSetValue(indent_tls_index as DWORD, level as LPVOID);
}

/***********************************************************************
 *           SPY_GetMsgInternal
 */
unsafe fn SPY_GetMsgInternal(msg: UINT) -> *const c_char {
    if msg <= SPY_MAX_MSGNUM {
        return MessageTypeNames.0[msg as usize];
    }
    if msg >= LVM_FIRST && msg <= LVM_FIRST + SPY_MAX_LVMMSGNUM as UINT {
        return LVMMessageTypeNames.0[(msg - LVM_FIRST) as usize];
    }
    if msg >= TV_FIRST && msg <= TV_FIRST + SPY_MAX_TVMSGNUM as UINT {
        return TVMessageTypeNames.0[(msg - TV_FIRST) as usize];
    }
    if msg >= HDM_FIRST && msg <= HDM_FIRST + SPY_MAX_HDMMSGNUM as UINT {
        return HDMMessageTypeNames.0[(msg - HDM_FIRST) as usize];
    }
    if msg >= TCM_FIRST && msg <= TCM_FIRST + SPY_MAX_TCMMSGNUM as UINT {
        return TCMMessageTypeNames.0[(msg - TCM_FIRST) as usize];
    }
    if msg >= PGM_FIRST && msg <= PGM_FIRST + SPY_MAX_PGMMSGNUM as UINT {
        return PGMMessageTypeNames.0[(msg - PGM_FIRST) as usize];
    }
    if msg >= CCM_FIRST && msg <= CCM_FIRST + SPY_MAX_CCMMSGNUM as UINT {
        return CCMMessageTypeNames.0[(msg - CCM_FIRST) as usize];
    }
    // The WM_WINE_* range is #ifndef __REACTOS__ — excluded in this build.
    core::ptr::null()
}

/***********************************************************************
 *           SPY_Bsearch_Msg
 */
unsafe fn SPY_Bsearch_Msg(msgs: *const USER_MSG, count: UINT, code: UINT) -> *const USER_MSG {
    let mut low: INT = 0;
    let mut high: INT = count as INT - 1;
    while low <= high {
        let idx = (low + high) / 2;
        let m = msgs.offset(idx as isize);
        if (*m).value == code {
            return m;
        }
        if (*m).value > code {
            high = idx - 1;
        } else {
            low = idx + 1;
        }
    }
    core::ptr::null()
}

/***********************************************************************
 *           SPY_GetClassLongOffsetName
 *
 * Gets the name of a class long offset.   ****  External function  ****
 */
#[no_mangle]
pub unsafe extern "C" fn SPY_GetClassLongOffsetName(offset: INT) -> *const c_char {
    let index: INT;
    if offset < 0
        && offset % 2 == 0
        && {
            index = -(offset + 8) / 2;
            (index as usize) < ClassLongOffsetNames.0.len()
        }
    {
        return ClassLongOffsetNames.0[index as usize];
    }
    cstr!("?")
}

/***********************************************************************
 *           SPY_GetClassName
 *
 *  Sets the value of "wnd_class" member of the instance structure.
 */
unsafe fn SPY_GetClassName(sp_e: *mut SPY_INSTANCE) {
    /* special code to detect a property sheet dialog */
    if GetClassLongPtrW((*sp_e).msg_hwnd, GCW_ATOM) == WC_DIALOG_ATOM
        && !GetPropW((*sp_e).msg_hwnd, PropSheetInfoStr.as_ptr()).is_null()
    {
        // strcpyW(sp_e->wnd_class, WC_PROPSHEETW)
        strcpyW(
            (*sp_e).wnd_class.as_mut_ptr(),
            WSTR_PROPSHEET.as_ptr(),
        );
    } else {
        GetClassNameW(
            (*sp_e).msg_hwnd,
            (*sp_e).wnd_class.as_mut_ptr(),
            (core::mem::size_of_val(&(*sp_e).wnd_class) / core::mem::size_of::<WCHAR>()) as INT,
        );
    }
}

/***********************************************************************
 *           SPY_GetMsgStuff
 *
 *  Get message name and other information for dumping
 */
unsafe fn SPY_GetMsgStuff(sp_e: *mut SPY_INSTANCE) {
    let mut p: *const USER_MSG;
    let msg_name = SPY_GetMsgInternal((*sp_e).msgnum);

    (*sp_e).data_len = 0;
    if msg_name.is_null() {
        let mut i: INT = 0;

        if (*sp_e).msgnum >= 0xc000 {
            if GlobalGetAtomNameA(
                (*sp_e).msgnum as ATOM,
                (*sp_e).msg_name.as_mut_ptr().add(1),
                (core::mem::size_of_val(&(*sp_e).msg_name) - 2) as INT,
            ) != 0
            {
                (*sp_e).msg_name[0] = b'"' as c_char;
                strcat((*sp_e).msg_name.as_mut_ptr(), cstr!("\""));
                return;
            }
        }
        if (*sp_e).wnd_class[0] == 0 {
            SPY_GetClassName(sp_e);
        }

        // (DEBUG_SPY trace omitted — channel off.)
        while !cc_array[i as usize].classname.is_null()
            && strcmpiW(cc_array[i as usize].classname, (*sp_e).wnd_class.as_ptr()) != 0
        {
            i += 1;
        }

        if !cc_array[i as usize].classname.is_null() {
            p = SPY_Bsearch_Msg(
                cc_array[i as usize].classmsg,
                cc_array[i as usize].count,
                (*sp_e).msgnum,
            );
            if !p.is_null() {
                lstrcpynA(
                    (*sp_e).msg_name.as_mut_ptr(),
                    (*p).name,
                    core::mem::size_of_val(&(*sp_e).msg_name) as INT,
                );
                (*sp_e).data_len = (*p).len as INT;
                return;
            }
        }
        if (*sp_e).msgnum >= WM_USER && (*sp_e).msgnum <= WM_APP {
            sprintf(
                (*sp_e).msg_name.as_mut_ptr(),
                cstr!("WM_USER+%d"),
                ((*sp_e).msgnum - WM_USER) as INT,
            );
        } else {
            sprintf((*sp_e).msg_name.as_mut_ptr(), cstr!("%04x"), (*sp_e).msgnum);
        }
    } else {
        lstrcpynA(
            (*sp_e).msg_name.as_mut_ptr(),
            msg_name,
            core::mem::size_of_val(&(*sp_e).msg_name) as INT,
        );
    }
}

// lstrcpynA: copy at most n-1 chars + NUL (kernel32 semantics, reproduced locally).
unsafe fn lstrcpynA(dst: *mut c_char, src: *const c_char, n: INT) -> *mut c_char {
    if n <= 0 || dst.is_null() || src.is_null() {
        return dst;
    }
    let mut i: INT = 0;
    while i < n - 1 && *src.offset(i as isize) != 0 {
        *dst.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }
    *dst.offset(i as isize) = 0;
    dst
}

/***********************************************************************
 *           SPY_GetWndName
 *
 *  Sets the value of "wnd_name" and "wnd_class" members of the instance structure.
 */
unsafe fn SPY_GetWndName(sp_e: *mut SPY_INSTANCE) {
    let len: INT;

    SPY_GetClassName(sp_e);

    len = InternalGetWindowText(
        (*sp_e).msg_hwnd,
        (*sp_e).wnd_name.as_mut_ptr(),
        (core::mem::size_of_val(&(*sp_e).wnd_name) / core::mem::size_of::<WCHAR>()) as INT,
    );
    if len == 0 {
        /* get class name */
        let mut dst: *mut WCHAR = (*sp_e).wnd_name.as_mut_ptr();
        let mut src: *mut WCHAR = (*sp_e).wnd_class.as_mut_ptr();
        let mut n: INT =
            (core::mem::size_of_val(&(*sp_e).wnd_name) / core::mem::size_of::<WCHAR>()) as INT - 3;
        *dst = b'{' as WCHAR;
        dst = dst.add(1);
        while {
            let cont = n > 0 && *src != 0;
            n -= 1;
            cont
        } {
            *dst = *src;
            dst = dst.add(1);
            src = src.add(1);
        }
        *dst = b'}' as WCHAR;
        dst = dst.add(1);
        *dst = 0;
    }
}

/***********************************************************************
 *           SPY_GetMsgName    ****  External function  ****
 *
 *  Get message name
 */
#[no_mangle]
pub unsafe extern "C" fn SPY_GetMsgName(msg: UINT, hWnd: HWND) -> *const c_char {
    let mut ext_sp_e: SPY_INSTANCE = core::mem::zeroed();
    let save_error = GetLastError();

    ext_sp_e.msgnum = msg;
    ext_sp_e.msg_hwnd = hWnd;
    ext_sp_e.lParam = 0;
    ext_sp_e.wParam = 0;
    ext_sp_e.wnd_class[0] = 0;
    SPY_GetMsgStuff(&mut ext_sp_e);
    SetLastError(save_error);
    // wine_dbg_sprintf("%s", ext_sp_e.msg_name): return the name in a rotating buffer.
    wine_dbg_sprintf_s(ext_sp_e.msg_name.as_ptr())
}

// Faithful stand-in for wine_dbg_sprintf("%s", s): copies the formatted string into one
// of a small ring of static buffers and returns a stable pointer to it.
const WINE_DBG_NBUF: usize = 16;
const WINE_DBG_BUFSZ: usize = 256;
static mut WINE_DBG_BUFS: [[c_char; WINE_DBG_BUFSZ]; WINE_DBG_NBUF] =
    [[0; WINE_DBG_BUFSZ]; WINE_DBG_NBUF];
static mut WINE_DBG_IDX: usize = 0;
unsafe fn wine_dbg_sprintf_s(s: *const c_char) -> *const c_char {
    let idx = WINE_DBG_IDX;
    WINE_DBG_IDX = (WINE_DBG_IDX + 1) % WINE_DBG_NBUF;
    let buf = WINE_DBG_BUFS[idx].as_mut_ptr();
    let mut i = 0usize;
    while i < WINE_DBG_BUFSZ - 1 && !s.is_null() && *s.add(i) != 0 {
        *buf.add(i) = *s.add(i);
        i += 1;
    }
    *buf.add(i) = 0;
    buf as *const c_char
}

/***********************************************************************
 *           SPY_GetVKeyName     ****  External function  ****
 */
#[no_mangle]
pub unsafe extern "C" fn SPY_GetVKeyName(wParam: WPARAM) -> *const c_char {
    let vk_key_name: *const c_char;
    if wParam <= SPY_MAX_VKKEYSNUM as WPARAM && !VK_KeyNames.0[wParam as usize].is_null() {
        vk_key_name = VK_KeyNames.0[wParam as usize];
    } else {
        vk_key_name = cstr!("VK_???");
    }
    vk_key_name
}

/***********************************************************************
 *           SPY_Bsearch_Notify
 */
unsafe fn SPY_Bsearch_Notify(code: UINT) -> *const SPY_NOTIFY {
    let mut low: INT = 0;
    let mut high: INT = spnfy_array.len() as INT - 1;
    while low <= high {
        let idx = (low + high) / 2;
        let n = &spnfy_array[idx as usize];
        if n.value == code {
            return n as *const SPY_NOTIFY;
        }
        if n.value < code {
            high = idx - 1;
        } else {
            low = idx + 1;
        }
    }
    core::ptr::null()
}

/***********************************************************************
 *           SPY_DumpMem
 *
 * Dead at runtime (TRACE channel off); structure preserved faithfully.
 */
unsafe fn SPY_DumpMem(header: *const c_char, q: *const UINT, len: INT) {
    let _ = header;
    let mut i: INT = 0;
    let mut q = q;
    while i < len - 12 {
        TRACE!();
        q = q.add(4);
        i += 16;
    }
    match (len - i + 3) & !3 {
        16 => {
            TRACE!();
        }
        12 => {
            TRACE!();
        }
        8 => {
            TRACE!();
        }
        4 => {
            TRACE!();
        }
        _ => {}
    }
}

/***********************************************************************
 *           SPY_DumpStructure
 *
 * Dead at runtime (only reached from the SPY_SENDMESSAGE / SPY_RESULT_OK paths, which
 * require spy_init()==TRUE, i.e. TRACE_ON(message); the channel is off in this build).
 * The message dispatch and the `enter`/pointer guards are preserved faithfully; the
 * per-case TRACE payloads (which would format struct fields) are no-ops since the
 * debug channel is disabled. lParam pointers are NOT dereferenced (the dumps that
 * would read them are inert), matching "validate before deref in dead-SEH paths".
 */
unsafe fn SPY_DumpStructure(sp_e: *const SPY_INSTANCE, enter: BOOL) {
    let msgnum = (*sp_e).msgnum;
    // Message-number constants used only for case labels (psdk values).
    const LVM_INSERTITEMW: UINT = LVM_FIRST + 77;
    const LVM_SETITEMW: UINT = LVM_FIRST + 76;
    const LVM_GETITEMW: UINT = LVM_FIRST + 75;
    const LVM_GETITEMRECT: UINT = LVM_FIRST + 14;
    const LVM_GETSUBITEMRECT: UINT = LVM_FIRST + 56;
    const LVM_SETITEMPOSITION32: UINT = LVM_FIRST + 49;
    const LVM_GETITEMPOSITION: UINT = LVM_FIRST + 16;
    const LVM_GETORIGIN: UINT = LVM_FIRST + 41;
    const TCM_INSERTITEMW: UINT = TCM_FIRST + 62;
    const TCM_INSERTITEMA: UINT = TCM_FIRST + 7;
    const TCM_SETITEMW: UINT = TCM_FIRST + 61;
    const TCM_SETITEMA: UINT = TCM_FIRST + 6;
    const TCM_GETITEMW: UINT = TCM_FIRST + 60;
    const TCM_GETITEMA: UINT = TCM_FIRST + 5;
    const TCM_ADJUSTRECT: UINT = TCM_FIRST + 40;
    const SBM_SETRANGE: UINT = 0x00e2;
    const SBM_GETRANGE: UINT = 0x00e3;
    const EM_EXSETSEL: UINT = WM_USER + 55;
    const EM_SETCHARFORMAT: UINT = WM_USER + 68;
    const WM_DRAWITEM: UINT = 0x002B;
    const WM_MEASUREITEM: UINT = 0x002C;
    const WM_NCCREATE: UINT = 0x0081;
    const WM_CREATE: UINT = 0x0001;
    const WM_SIZE: UINT = 0x0005;
    const WM_WINDOWPOSCHANGED: UINT = 0x0047;
    const WM_WINDOWPOSCHANGING: UINT = 0x0046;
    const WM_STYLECHANGED: UINT = 0x007D;
    const WM_STYLECHANGING: UINT = 0x007C;
    const WM_NCCALCSIZE: UINT = 0x0083;
    const WM_NOTIFY: UINT = 0x004E;

    match msgnum {
        LVM_INSERTITEMW | LVM_SETITEMW => {
            if enter == 0 {
                return;
            }
            // fall through to LVM_GETITEMW
            let item = (*sp_e).lParam as *const c_void;
            if !item.is_null() {
                SPY_DumpMem(cstr!("LVITEM"), item as *const UINT, 0);
            }
        }
        LVM_GETITEMW => {
            let item = (*sp_e).lParam as *const c_void;
            if !item.is_null() {
                SPY_DumpMem(cstr!("LVITEM"), item as *const UINT, 0);
            }
        }
        TCM_INSERTITEMW | TCM_INSERTITEMA | TCM_SETITEMW | TCM_SETITEMA => {
            if enter == 0 {
                return;
            }
            let item = (*sp_e).lParam as *const c_void;
            if !item.is_null() {
                SPY_DumpMem(cstr!("TCITEM"), item as *const UINT, 0);
            }
        }
        TCM_GETITEMW | TCM_GETITEMA => {
            let item = (*sp_e).lParam as *const c_void;
            if !item.is_null() {
                SPY_DumpMem(cstr!("TCITEM"), item as *const UINT, 0);
            }
        }
        TCM_ADJUSTRECT | LVM_GETITEMRECT | LVM_GETSUBITEMRECT => {
            let rc = (*sp_e).lParam as *const c_void;
            if !rc.is_null() {
                TRACE!();
            }
        }
        LVM_SETITEMPOSITION32 => {
            if enter == 0 {
                return;
            }
            let point = (*sp_e).lParam as *const c_void;
            if !point.is_null() {
                TRACE!();
            }
        }
        LVM_GETITEMPOSITION | LVM_GETORIGIN => {
            let point = (*sp_e).lParam as *const c_void;
            if !point.is_null() {
                TRACE!();
            }
        }
        SBM_SETRANGE => {
            if enter == 0 && msgnum == SBM_SETRANGE {
                return;
            }
            TRACE!();
        }
        SBM_GETRANGE => {
            if (enter != 0 && msgnum == SBM_GETRANGE) || (enter == 0 && msgnum == SBM_SETRANGE) {
                return;
            }
            let ptmin = (*sp_e).wParam as *const c_void;
            let ptmax = (*sp_e).lParam as *const c_void;
            if !ptmin.is_null() && !ptmax.is_null() {
                TRACE!();
            } else if !ptmin.is_null() {
                TRACE!();
            } else if !ptmax.is_null() {
                TRACE!();
            }
        }
        EM_EXSETSEL => {
            if enter != 0 && (*sp_e).lParam != 0 {
                TRACE!();
            }
        }
        EM_SETCHARFORMAT => {
            if enter != 0 && (*sp_e).lParam != 0 {
                TRACE!();
            }
        }
        WM_DRAWITEM => {
            if enter == 0 {
                return;
            }
            TRACE!();
        }
        WM_MEASUREITEM => {
            TRACE!();
        }
        WM_NCCREATE | WM_CREATE => {
            if enter == 0 {
                return;
            }
            let _unicode = IsWindowUnicode((*sp_e).msg_hwnd);
            let _cs = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_SIZE => {
            if enter == 0 {
                return;
            }
            TRACE!();
        }
        WM_WINDOWPOSCHANGED => {
            if enter == 0 {
                return;
            }
            // fall through to WM_WINDOWPOSCHANGING
            let _lpwp = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_WINDOWPOSCHANGING => {
            let _lpwp = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_STYLECHANGED => {
            if enter == 0 {
                return;
            }
            // fall through to WM_STYLECHANGING
            let _ss = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_STYLECHANGING => {
            let _ss = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_NCCALCSIZE => {
            let _rc = (*sp_e).lParam as *const c_void;
            TRACE!();
        }
        WM_NOTIFY => {
            // pnmh = (NMHDR*)lParam; the code field would be read here. Dead path.
            let pnmh = (*sp_e).lParam as *const c_void;
            if !pnmh.is_null() {
                // p = SPY_Bsearch_Notify(pnmh->code) — pnmh->code read is inert (no deref).
                let _p: *const SPY_NOTIFY = core::ptr::null();
                TRACE!();
            }
        }
        _ => {
            if (*sp_e).data_len > 0 {
                SPY_DumpMem(
                    cstr!("MSG lParam"),
                    (*sp_e).lParam as *const UINT,
                    (*sp_e).data_len,
                );
            }
        }
    }
}

/***********************************************************************
 *           spy_init
 */
unsafe fn spy_init() -> BOOL {
    let mut i: INT;
    let mut buffer: [c_char; 1024] = [0; 1024];
    let mut hkey: HKEY = core::ptr::null_mut();

    if TRACE_ON_message() == 0 {
        return 0; // FALSE — the spy is compiled out in this build.
    }

    if indent_tls_index == TLS_OUT_OF_INDEXES as INT {
        let index = TlsAlloc();
        if InterlockedCompareExchange(
            core::ptr::addr_of_mut!(indent_tls_index) as *mut LONG,
            index as LONG,
            TLS_OUT_OF_INDEXES as LONG,
        ) != TLS_OUT_OF_INDEXES as LONG
        {
            TlsFree(index);
        }
    }

    if !spy_exclude.is_null() {
        return 1;
    }
    let exclude = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (SPY_MAX_MSGNUM + 2) as SIZE_T,
    ) as *mut u8;

    /* @@ Wine registry key: HKCU\Software\Wine\Debug */
    if RegOpenKeyA(HKEY_CURRENT_USER, cstr!("Software\\Wine\\Debug"), &mut hkey) == 0 {
        let mut ty: DWORD = 0;
        let mut count: DWORD = core::mem::size_of_val(&buffer) as DWORD;

        buffer[0] = 0;
        if RegQueryValueExA(
            hkey,
            cstr!("SpyInclude"),
            core::ptr::null_mut(),
            &mut ty,
            buffer.as_mut_ptr() as LPBYTE,
            &mut count,
        ) == 0
            && strcmp(buffer.as_ptr(), cstr!("INCLUDEALL")) != 0
        {
            i = 0;
            while i <= SPY_MAX_MSGNUM as INT {
                let nm = MessageTypeNames.0[i as usize];
                *exclude.offset(i as isize) =
                    (!nm.is_null() && strstr(buffer.as_ptr(), nm).is_null()) as u8;
                i += 1;
            }
        }

        count = core::mem::size_of_val(&buffer) as DWORD;
        buffer[0] = 0;
        if RegQueryValueExA(
            hkey,
            cstr!("SpyExclude"),
            core::ptr::null_mut(),
            &mut ty,
            buffer.as_mut_ptr() as LPBYTE,
            &mut count,
        ) == 0
        {
            if strcmp(buffer.as_ptr(), cstr!("EXCLUDEALL")) == 0 {
                i = 0;
                while i <= SPY_MAX_MSGNUM as INT {
                    *exclude.offset(i as isize) = 1; // TRUE
                    i += 1;
                }
            } else {
                i = 0;
                while i <= SPY_MAX_MSGNUM as INT {
                    let nm = MessageTypeNames.0[i as usize];
                    *exclude.offset(i as isize) =
                        (!nm.is_null() && !strstr(buffer.as_ptr(), nm).is_null()) as u8;
                    i += 1;
                }
            }
        }

        count = core::mem::size_of_val(&buffer) as DWORD;
        if RegQueryValueExA(
            hkey,
            cstr!("SpyExcludeDWP"),
            core::ptr::null_mut(),
            &mut ty,
            buffer.as_mut_ptr() as LPBYTE,
            &mut count,
        ) == 0
        {
            *exclude.add((SPY_MAX_MSGNUM + 1) as usize) = atoi(buffer.as_ptr()) as u8;
        }

        RegCloseKey(hkey);
    }

    if !InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(spy_exclude) as *mut PVOID,
        exclude as PVOID,
        core::ptr::null_mut(),
    )
    .is_null()
    {
        HeapFree(GetProcessHeap(), 0, exclude as LPVOID);
    }

    1 // TRUE
}

/***********************************************************************
 *           SPY_EnterMessage     ****  External function  ****
 */
#[no_mangle]
pub unsafe extern "C" fn SPY_EnterMessage(
    iFlag: INT,
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut sp_e: SPY_INSTANCE = core::mem::zeroed();
    let indent: INT;
    let save_error = GetLastError();

    if spy_init() == 0 || exclude_msg(msg) != 0 {
        return;
    }

    sp_e.msgnum = msg;
    sp_e.msg_hwnd = hWnd;
    sp_e.lParam = lParam;
    sp_e.wParam = wParam;
    SPY_GetWndName(&mut sp_e);
    SPY_GetMsgStuff(&mut sp_e);
    indent = get_indent_level() as INT;

    /* each SPY_SENDMESSAGE must be complemented by call to SPY_ExitMessage */
    match iFlag {
        SPY_DISPATCHMESSAGE => {
            TRACE!();
        }
        SPY_SENDMESSAGE => {
            let mut taskName: [c_char; 20] = [0; 20];
            let tid = GetWindowThreadProcessId(hWnd, core::ptr::null_mut());

            if tid == GetCurrentThreadId() {
                // strcpy(taskName, "self")
                lstrcpynA(taskName.as_mut_ptr(), cstr!("self"), 20);
            } else {
                sprintf(
                    taskName.as_mut_ptr(),
                    cstr!("tid %04ld"),
                    GetCurrentThreadId() as core::ffi::c_long,
                );
            }

            TRACE!();
            SPY_DumpStructure(&sp_e, 1);
        }
        SPY_DEFWNDPROC => {
            if exclude_dwp() != 0 {
                return;
            }
            TRACE!();
        }
        _ => {}
    }
    set_indent_level((indent + SPY_INDENT_UNIT) as INT_PTR);
    SetLastError(save_error);
}

/***********************************************************************
 *           SPY_ExitMessage      ****  External function  ****
 */
#[no_mangle]
pub unsafe extern "C" fn SPY_ExitMessage(
    iFlag: INT,
    hWnd: HWND,
    msg: UINT,
    lReturn: LRESULT,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut sp_e: SPY_INSTANCE = core::mem::zeroed();
    let mut indent: INT;
    let save_error = GetLastError();

    if TRACE_ON_message() == 0
        || exclude_msg(msg) != 0
        || (exclude_dwp() != 0 && iFlag == SPY_RESULT_DEFWND)
    {
        return;
    }

    sp_e.msgnum = msg;
    sp_e.msg_hwnd = hWnd;
    sp_e.lParam = lParam;
    sp_e.wParam = wParam;
    SPY_GetWndName(&mut sp_e);
    SPY_GetMsgStuff(&mut sp_e);

    indent = get_indent_level() as INT;
    if indent != 0 {
        indent -= SPY_INDENT_UNIT;
        set_indent_level(indent as INT_PTR);
    }

    match iFlag {
        SPY_RESULT_DEFWND => {
            TRACE!();
        }
        SPY_RESULT_OK => {
            TRACE!();
            SPY_DumpStructure(&sp_e, 0);
        }
        _ => {}
    }
    SetLastError(save_error);
}
