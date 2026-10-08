use crate::prelude::*;
use crate::dialogs_folder_options::BitmapFromIcon;
use crate::atl::CPathW;
use crate::atl::CSimpleMap;
use crate::shell32::iconcache::ExtractIconExW;
use crate::shell32::CExtractIcon::IconLocation;
use crate::rosctrls::ImageList_RemoveAll;
use crate::rosctrls::ListView_DeleteAllItems;
use crate::rosctrls::ListView_DeleteItem;
use crate::rosctrls::ListView_EnsureVisible;
use crate::rosctrls::ListView_FindItem;
use crate::rosctrls::ListView_GetItem;
use crate::rosctrls::ListView_GetItemText;
use crate::rosctrls::ListView_GetNextItem;
use crate::rosctrls::ListView_SetExtendedListViewStyleEx;
use crate::rosctrls::ListView_SetImageList;
use crate::rosctrls::ListView_SetItem;
use crate::rosctrls::ListView_SetItemState;
use crate::rosctrls::ListView_SortItems;
use crate::default_context_menu_support::RegKeyExists;
use crate::open_with_menu_support::RegSetString;
use crate::default_context_menu_support::RegValueExists;
use crate::open_with_menu_support::StringCbCopyW;
use crate::dialogs_filetypes::_FILE_TYPE_ENTRY_DestroyIcons_body;
use crate::dialogs_filetypes::_FILE_TYPE_ENTRY_Initialize_body;
use crate::dialogs_filetypes::_FILE_TYPE_ENTRY_InvalidateDefaultApp_body;
use crate::dialogs_filetypes::_FILE_TYPE_ENTRY_InvalidateTypeName_body;
use crate::dialogs_filetypes::_FILE_TYPE_ENTRY_IsExtension_body;
macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_073595696440: [WCHAR; 10] = [0x0044, 0x006F, 0x0063, 0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0000];
pub static CPP_COM_WSTR_07E190C927BB: [WCHAR; 13] = [0x0055, 0x0052, 0x004C, 0x0020, 0x0050, 0x0072, 0x006F, 0x0074, 0x006F, 0x0063, 0x006F, 0x006C, 0x0000];
pub static CPP_COM_WSTR_0C71B3CB5580: [WCHAR; 8] = [0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_1B9334FEECE6: [WCHAR; 2] = [0x0022, 0x0000];
pub static CPP_COM_WSTR_2B585AE3D008: [WCHAR; 13] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x0000];
pub static CPP_COM_WSTR_2ED82ED19E49: [WCHAR; 6] = [0x0020, 0x0022, 0x0025, 0x0031, 0x0022, 0x0000];
pub static CPP_COM_WSTR_2F0541D29D81: [WCHAR; 14] = [0x0041, 0x006C, 0x0077, 0x0061, 0x0079, 0x0073, 0x0053, 0x0068, 0x006F, 0x0077, 0x0045, 0x0078, 0x0074, 0x0000];
pub static CPP_COM_WSTR_325A56B0AAB7: [WCHAR; 13] = [0x0042, 0x0072, 0x006F, 0x0077, 0x0073, 0x0065, 0x0072, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_3BE8B25B7ED1: [WCHAR; 12] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_5BFECD9F5862: [WCHAR; 4] = [0x002E, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_65C19E07157A: [WCHAR; 11] = [0x0046, 0x0069, 0x006C, 0x0065, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_7ABCE230ED71: [WCHAR; 5] = [0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7C8F589C7B27: [WCHAR; 3] = [0x0025, 0x0031, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_8419E6E00D5A: [WCHAR; 7] = [0x0066, 0x0074, 0x0025, 0x0030, 0x0036, 0x0075, 0x0000];
pub static CPP_COM_WSTR_8A7837CAF8C9: [WCHAR; 11] = [0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8EFF46569AE7: [WCHAR; 6] = [0x0025, 0x0073, 0x002C, 0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A2E2913C930F: [WCHAR; 10] = [0x0045, 0x0064, 0x0069, 0x0074, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_B730BA9299D0: [WCHAR; 5] = [0x002E, 0x0073, 0x0063, 0x0072, 0x0000];
pub static CPP_COM_WSTR_C245AFF91259: [WCHAR; 2] = [0x002E, 0x0000];
pub static CPP_COM_WSTR_D5B988E0271D: [WCHAR; 4] = [0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_DBE081544FA1: [WCHAR; 17] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E4DEECAD1CA5: [WCHAR; 17] = [0x0046, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x0054, 0x0079, 0x0070, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_EE89605B31B0: [WCHAR; 6] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_FB47CCC40FE6: [WCHAR; 3] = [0x0025, 0x0073, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn RegDeleteValueW(key: HKEY, name: LPCWSTR) -> LONG;
}

// ---- file-scope preamble (translated) ----
pub const FTA_MODIFYMASK: DWORD = FTA_OpenIsSafe;
pub const NOASSOCRESID: u16 = IDI_SHELL_DOCUMENT as WORD;
pub const SUPPORT_EXTENSIONWITHOUTPROGID: u32 = 1;
pub const ASSOC_CCHMAX: usize = 32 + 1;
pub const TYPENAME_CCHMAX: usize = 100;
pub const ICONLOCATION_CCHMAX: usize = (MAX_PATH + 1 + 11) as usize;

#[repr(C)]
pub struct _FILE_TYPE_ENTRY {
    pub FileExtension: [u16; ASSOC_CCHMAX],
    pub FileDescription: [u16; TYPENAME_CCHMAX],
    pub ClassKey: [u16; MAX_PATH as usize],
    pub EditFlags: DWORD,
    pub AppName: [u16; 64],
    pub hIconSmall: HICON,
    pub ProgramPath: [u16; MAX_PATH as usize],
    pub IconPath: [u16; MAX_PATH as usize],
    pub nIconIndex: INT,
}
pub type FILE_TYPE_ENTRY = _FILE_TYPE_ENTRY;
pub type PFILE_TYPE_ENTRY = *mut _FILE_TYPE_ENTRY;

#[repr(C)]
pub struct _FILE_TYPE_GLOBALS {
    pub himlSmall: HIMAGELIST,
    pub IconSize: UINT,
    pub hDefExtIconSmall: HICON,
    pub hOpenWithImage: HBITMAP,
    pub hHeap: HANDLE,
    pub NoneString: [u16; 42],
    pub SortCol: INT8,
    pub SortReverse: INT8,
    pub Restricted: UINT,
}
pub type FILE_TYPE_GLOBALS = _FILE_TYPE_GLOBALS;
pub type PFILE_TYPE_GLOBALS = *mut _FILE_TYPE_GLOBALS;

pub const LISTBOX_MARGIN: i32 = 2;
pub const ETF_ALWAYSEXT: i32 = 1 << 0;
pub const ETF_BROWSESAME: i32 = 1 << 1;

#[repr(C)]
pub struct EDITTYPE_DIALOG {
    pub hwndLV: HWND,
    pub pG: PFILE_TYPE_GLOBALS,
    pub pEntry: PFILE_TYPE_ENTRY,
    pub CommandLineMap: CSimpleMap<CStringW, CStringW>,
    pub ModifiedVerbs: CAtlList<CStringW>,
    pub szIconPath: [u16; MAX_PATH as usize],
    pub nIconIndex: INT,
    pub ChangedIcon: bool,
    pub szDefaultVerb: [u16; VERBKEY_CCHMAX as usize],
    pub TypeName: [u16; TYPENAME_CCHMAX],
}
pub type PEDITTYPE_DIALOG = *mut EDITTYPE_DIALOG;

#[repr(C)]
pub struct NEWEXT_DIALOG {
    pub hwndLV: HWND,
    pub rcDlg: RECT,
    pub bAdvanced: BOOL,
    pub dy: INT,
    pub szExt: [u16; 16],
    pub szFileType: [u16; 64],
}
pub type PNEWEXT_DIALOG = *mut NEWEXT_DIALOG;

#[repr(C)]
pub struct ACTION_DIALOG {
    pub hwndLB: HWND,
    pub pG: PFILE_TYPE_GLOBALS,
    pub pEntry: PFILE_TYPE_ENTRY,
    pub szAction: [u16; VERBKEY_CCHMAX as usize],
    pub szApp: [u16; MAX_PATH as usize],
    pub bUseDDE: BOOL,
}
pub type PACTION_DIALOG = *mut ACTION_DIALOG;

// ---- substrate the islands do not carry, added to the PREAMBLE unit so it survives
// ---- reassembly (the assembled .rs is rewritten in full every time).

/// filetypes.cpp:358 `enum EDITTYPEFLAGS { ETF_ALWAYSEXT = 1 << 0, ETF_BROWSESAME = 1 << 1 };`
/// A file-local enum, used as a by-value parameter and an out-reference.
pub type EDITTYPEFLAGS = INT;  // the C++ enum's underlying type

// The TCHAR-era spellings commctrl.h keeps as aliases of the W structs the island does have.
pub type LV_ITEM = LVITEMW;
pub type LV_FINDINFO = LVFINDINFOW;
pub type _NMLISTVIEW = NMLISTVIEW;

// `LOWORD!` is emitted by rustconv now (render_rewrite_support), not hand-written here.

// shlwapi.h:959 FILETYPEATTRIBUTEFLAGS values the island does not carry. The class slice of
// this TU emits its own copies; a `const` is deliberately never cross-imported (see
// crate_integrate's _NEVER_IMPORTED_KINDS), so the free slice needs its own.
pub const FTA_OpenIsSafe: DWORD = 0x00010000;
pub const FTA_NoEditDesc: DWORD = 0x00000100;
pub const FTA_NoEditDflt: DWORD = 0x00000400;
pub const FTA_NoEditIcon: DWORD = 0x00000200;

// commctrl.h TCHAR-era aliases of the W structs the island does have.
pub type _NMLVDISPINFOW = NMLVDISPINFOW;
pub type _LV_KEYDOWN = NMLVKEYDOWN;

/// utils.h:101 `SHELL32_SHExtractIcon` — an inline, because `SHExtractIconsW` is a FORWARD
/// and shell32 must not call its own forward. Returns NULL rather than an uninitialised
/// handle when PrivateExtractIconsW extracts nothing.
#[inline]
// utils.h declares cx/cy as `int`, but the only caller here is filetypes.cpp's
// `DoExtractIcon(LPCWSTR, INT, UINT cx, UINT cy)`, which leans on C++'s implicit
// UINT->int conversion. Rust has to spell that conversion somewhere, and the forwarder
// is emitted by the assembler rather than banked, so it is absorbed here.
unsafe fn SHELL32_SHExtractIcon(File: LPCWSTR, Index: INT, cx: UINT, cy: UINT) -> HICON {
    let mut hIco: HICON = core::ptr::null_mut();
    let r = PrivateExtractIconsW(File, Index, cx as INT, cy as INT, core::ptr::addr_of_mut!(hIco),
                                 core::ptr::null_mut(), 1, 0);
    if r > 0 { hIco } else { core::ptr::null_mut() }
}

/// utils.h:71 `RegSetOrDelete` — an inline: a NULL payload DELETES the value rather than
/// writing an empty one, which is the whole point of the helper.
#[inline]
unsafe fn RegSetOrDelete(hKey: HKEY, Name: LPCWSTR, Type: DWORD, Data: LPCVOID, Size: DWORD) -> DWORD {
    if !Data.is_null() {
        RegSetValueExW(hKey, Name, 0, Type, Data as LPBYTE, Size) as DWORD
    } else {
        RegDeleteValueW(hKey, Name) as DWORD
    }
}

// ---- the TU's one HAND-WRITTEN body ----
// It lives in the PREAMBLE, not a companion module, because it calls this TU's C
// `static` helpers (DoExtractIcon_2, Normalize, InitializeDefaultIcons) and a C
// static is module-private in Rust by design (abi_enforce ties `pub` to external
// linkage). Inside the preamble it is the same module, so they are simply in scope.
// Function order does not matter in Rust, so being defined before them is fine.
// HAND-WRITTEN from the C++ (filetypes.cpp:1114). The paid worker was refused by the call
// seal — "wrong call site (argument count): DoExtractIcon wants 3, got 4" — and that refusal
// was CORRECT, unlike the two `wants 4, got 4` rejects in the same file, which were an empty
// accepted range on an overloaded callee and now replay at $0.
//
// filetypes.cpp declares DoExtractIcon TWICE:
//     DoExtractIcon(LPCWSTR, INT, UINT cx, UINT cy)             — extract at an exact size
//     DoExtractIcon(LPCWSTR, INT = 0, BOOL bSmall = FALSE)      — derives cx/cy from bSmall
// The C here passes THREE arguments ending in `TRUE`, so it is the second overload, which
// asks GetSystemMetrics for SM_CXSMICON/SM_CYSMICON. The worker instead emitted the
// four-argument form with ONE sealed constant duplicated into both the cx and cy slots —
// which compiles, and extracts the icon at the wrong size.
unsafe extern "C" fn EditTypeDlg_UpdateEntryIcon(
    _hwndDlg: HWND,
    pEditType: PEDITTYPE_DIALOG,
) -> BOOL {
    let pEntry: PFILE_TYPE_ENTRY = (*pEditType).pEntry;
    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // "I want the default icon" — the C++ clears the path before Normalize re-derives it.
    (*pEntry).IconPath[0] = UNICODE_NULL;
    Normalize(pEntry);
    crate::dialogs_filetypes::_FILE_TYPE_ENTRY_DestroyIcons_body(pEntry.cast());
    (*pEntry).hIconSmall = DoExtractIcon_2((*pEntry).IconPath.as_ptr(), (*pEntry).nIconIndex, TRUE);

    if ExpandEnvironmentStringsW(
        (*pEditType).szIconPath.as_ptr(),
        buf.as_mut_ptr(),
        buf.len() as UINT,
    ) != 0
        && buf[0] != 0
    {
        let hIco: HICON = DoExtractIcon_2(buf.as_ptr(), (*pEditType).nIconIndex, TRUE);
        if !hIco.is_null() {
            crate::dialogs_filetypes::_FILE_TYPE_ENTRY_DestroyIcons_body(pEntry.cast());
            (*pEntry).hIconSmall = hIco;
        }
    }

    crate::open_with_menu_support::StringCbCopyW(
        (*pEntry).IconPath.as_mut_ptr(),
        core::mem::size_of_val(&(*pEntry).IconPath),
        (*pEditType).szIconPath.as_ptr(),
    );
    (*pEntry).nIconIndex = (*pEditType).nIconIndex;

    let hListView: HWND = (*pEditType).hwndLV;
    InitializeDefaultIcons((*pEditType).pG);
    let himlSmall: HIMAGELIST = (*(*pEditType).pG).himlSmall;
    let iSmallImage: INT = ImageList_AddIcon!(himlSmall, (*pEntry).hIconSmall);

    let iItem: INT = crate::rosctrls::ListView_GetNextItem(hListView, -1, LVNI_SELECTED);
    if iItem != -1 {
        // C++: `LV_ITEMW Item = { LVIF_IMAGE, iItem };` — aggregate init sets mask and iItem
        // and zeroes the rest, so the zeroed base matters (iSubItem must be 0).
        let mut Item: LVITEMW = core::mem::zeroed();
        Item.mask = LVIF_IMAGE;
        Item.iItem = iItem;
        Item.iImage = iSmallImage;
        crate::rosctrls::ListView_SetItem(hListView, core::ptr::addr_of_mut!(Item));
    }
    TRUE
}

// The TCHAR-era spellings the C++ writes bare, as INLINE WRAPPERS rather than
// `#[link_name]` externs: crate_integrate strips a preamble extern whose target the island
// already provides, which silently removed the alias and left the call unresolved. A wrapper
// survives, and its body can hold the declaration it needs.

/// `RegCreateKeyEx` — the TCHAR macro; shell32 is UNICODE, so it is the W entry point.
#[inline]
unsafe fn RegCreateKeyEx(key: HKEY, sub: LPCWSTR, reserved: DWORD, class: LPWSTR,
                         options: DWORD, sam: REGSAM, sa: LPSECURITY_ATTRIBUTES,
                         out: *mut HKEY, disp: LPDWORD) -> LONG {
    RegCreateKeyExW(key, sub, reserved, class, options, sam, sa, out, disp)
}

/// `RegOpenKeyEx` — likewise.
#[inline]
unsafe fn RegOpenKeyEx(key: HKEY, sub: LPCWSTR, options: DWORD, sam: REGSAM,
                       out: *mut HKEY) -> LONG {
    RegOpenKeyExW(key, sub, options, sam, out)
}

// `_wcsupr` / `_wcslwr` are NOT wrapped here. A wrapper cannot carry its own declaration:
// crate_integrate's strip pass reaches a nested `extern "C"` block too, which left
// `unsafe fn _wcsupr(s) { _wcsupr(s) }` — self-recursion that compiled. They now live in
// the crate prelude's CRT extern block beside `_wcsicmp`, which is where a real link symbol
// belongs.

/// strsafe.h `StringCbPrintfW`, limited to the conversions THIS TU uses and refusing the
/// rest. The crate's existing helpers (move_to_menu.rs, open_with_menu.rs) handle `%s` only;
/// filetypes.cpp also needs `%d` and a ZERO-PADDED `%06u`, and one of those builds a registry
/// KEY NAME (`L"ft%06u"`), so dropping the padding writes to a different key.
///
/// `fmt` is a VARIABLE at one call site — `L"%s,%d"` or `L"%s"` depending on whether an icon
/// index applies — so a trailing argument the format never consumes is normal, not an error.
#[derive(Clone, Copy)]
enum CppComPrintfArg {
    Str(LPCWSTR),
    Int(INT),
    UInt(UINT),
}
trait CppComIntoPrintfArg { fn into_arg(self) -> CppComPrintfArg; }
impl CppComIntoPrintfArg for LPCWSTR { fn into_arg(self) -> CppComPrintfArg { CppComPrintfArg::Str(self) } }
impl CppComIntoPrintfArg for LPWSTR { fn into_arg(self) -> CppComPrintfArg { CppComPrintfArg::Str(self as LPCWSTR) } }
impl CppComIntoPrintfArg for INT { fn into_arg(self) -> CppComPrintfArg { CppComPrintfArg::Int(self) } }
impl CppComIntoPrintfArg for UINT { fn into_arg(self) -> CppComPrintfArg { CppComPrintfArg::UInt(self) } }

unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[CppComPrintfArg],
) -> HRESULT {
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 {
        return E_FAIL;
    }
    let (mut out, mut input, mut argi) = (0usize, 0usize, 0usize);
    // STRSAFE_E_INSUFFICIENT_BUFFER — truncate, NUL-terminate, and say so, as strsafe does.
    let overflow = 0x8007007Au32 as HRESULT;
    macro_rules! put {
        ($unit:expr) => {{
            if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return overflow; }
            *dest.add(out) = $unit; out += 1;
        }};
    }
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit != b'%' as WCHAR { put!(unit); input += 1; continue; }

        // %[0][width](s|d|u)
        let mut j = input + 1;
        let mut zero_pad = false;
        if *format.add(j) == b'0' as WCHAR { zero_pad = true; j += 1; }
        let mut width = 0usize;
        while (*format.add(j) as u8 as char).is_ascii_digit() {
            width = width * 10 + (*format.add(j) as u8 - b'0') as usize;
            j += 1;
        }
        // `%%` is a literal percent. Checked before the width scan would have eaten nothing.
        if *format.add(input + 1) == b'%' as WCHAR { put!(unit); input += 2; continue; }
        let conv = *format.add(j) as u8;
        if conv != b's' && conv != b'd' && conv != b'u' {
            // A conversion this port does not implement: REFUSE (STRSAFE_E_INVALID_PARAMETER,
            // empty result), matching cpp_com's generated helper. Copying it through literally
            // — what this did before — produces wrong text with nothing red anywhere.
            *dest = 0;
            return 0x80070057u32 as HRESULT;
        }
        if argi >= args.len() { return E_FAIL; }
        let arg = args[argi]; argi += 1; input = j + 1;

        match arg {
            CppComPrintfArg::Str(p) => {
                // strsafe treats a NULL string argument as empty rather than faulting.
                if !p.is_null() {
                    let mut k = 0usize;
                    while *p.add(k) != 0 { put!(*p.add(k)); k += 1; }
                }
            }
            CppComPrintfArg::Int(_) | CppComPrintfArg::UInt(_) => {
                let (neg, mut mag) = match arg {
                    CppComPrintfArg::Int(v) => (v < 0, (v as i64).unsigned_abs()),
                    CppComPrintfArg::UInt(v) => (false, v as u64),
                    CppComPrintfArg::Str(_) => unreachable!(),
                };
                let mut digits = [0u8; 20];
                let mut n = 0usize;
                loop { digits[n] = (mag % 10) as u8; mag /= 10; n += 1; if mag == 0 { break; } }
                if neg { put!(b'-' as WCHAR); }
                // `%06u`: pad with zeroes to WIDTH. A sign, when present, counts toward it.
                if zero_pad {
                    for _ in (n + neg as usize)..width { put!(b'0' as WCHAR); }
                }
                while n > 0 { n -= 1; put!((b'0' + digits[n]) as WCHAR); }
            }
        }
    }
    *dest.add(out) = 0;
    S_OK
}

macro_rules! StringCbPrintfW {
    ($dest:expr, $bytes:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $fmt, &[$(CppComIntoPrintfArg::into_arg($arg)),*])
    };
}

// ---- SDK constants the island does not carry ----

/// shlwapi.h FILETYPEATTRIBUTEFLAGS. The island renders the enum's TYPE but not its members.
pub const FTA_Exclude: DWORD = 0x00000001;
pub const FTA_NoEdit: DWORD = 0x00000008;
pub const FTA_NoRemove: DWORD = 0x00000010;
pub const FTA_NoNewVerb: DWORD = 0x00000020;
pub const FTA_NoEditVerb: DWORD = 0x00000040;
pub const FTA_NoRemoveVerb: DWORD = 0x00000080;

/// shlobj.h:1873 — the third member of a block that starts at `REST_NODISCONNECT = 0x41000001`,
/// so it is 0x41000003; the header's own comment says so.
/// commctrl.h:131 `#define NM_DBLCLK (NM_FIRST-3)`, with `NM_FIRST (0U- 0U)`. bindgen keeps
/// `NM_FIRST` as `u32` and drops the derived codes, so this is spelled as the signed value the
/// `NMHDR::code` scrutinee is compared at. Missing, it is a BINDING that matches everything —
/// see `LVN_GETDISPINFO` below for the same trap.
pub const NM_DBLCLK: INT = (NM_FIRST as INT) - 3;

/// commctrl.h:3165 `#define LVN_GETDISPINFO __MINGW_NAME_AW(LVN_GETDISPINFO)` — shell32 is
/// UNICODE, so it is `LVN_GETDISPINFOW`, `LVN_FIRST-77`. The A/W alias is a macro, so it is
/// NOT in the island, and without it `LVN_GETDISPINFO =>` is a fresh BINDING that matches
/// every notification code and swallows the whole `WM_NOTIFY` dispatch. rustc says so only
/// as an `unreachable pattern` warning on the arms behind it.
pub const LVN_GETDISPINFO: INT = LVN_GETDISPINFOW;

pub const REST_NOFILEASSOCIATE: RESTRICTIONS = 0x41000003;

/// shlwapi.h `ASSOCSTR` — a plain C enum starting at `ASSOCSTR_COMMAND = 1`, so this is 2.
pub const ASSOCSTR_EXECUTABLE: ASSOCSTR = 2;

/// shlobj.h `OPEN_AS_INFO_FLAGS`.
pub const OAIF_REGISTER_EXT: UINT = 2;
pub const OAIF_FORCE_REGISTRATION: UINT = 8;

/// commctrl.h:2385 `#define LPSTR_TEXTCALLBACKW ((LPWSTR)-1L)` — the sentinel that tells a
/// list view to ask for the text via LVN_GETDISPINFO rather than store it.
pub const LPSTR_TEXTCALLBACK: LPWSTR = -1isize as LPWSTR;

/// commdlg.h:397 `OPENFILENAME_SIZE_VERSION_400W` is
/// `CDSIZEOF_STRUCT(OPENFILENAMEW, lpTemplateName)` — the offset of that member PLUS its size,
/// i.e. the size of the pre-Win2000 struct. Computed rather than hardcoded because the member
/// is pointer-sized and the offsets differ per arch.
pub const OPENFILENAME_SIZE_VERSION_400W: DWORD =
    (core::mem::offset_of!(OPENFILENAMEW, lpTemplateName) + core::mem::size_of::<LPCWSTR>()) as DWORD;

/// `GetObject` is the TCHAR macro; shell32 is UNICODE.
#[inline]
unsafe fn GetObject(h: HANDLE, c: INT, pv: LPVOID) -> INT {
    GetObjectW(h, c, pv)
}

/// `DBG_UNREFERENCED_LOCAL_VARIABLE!(v)` — a no-op in every build.
macro_rules! DBG_UNREFERENCED_LOCAL_VARIABLE { ($v:expr) => { let _ = &$v; }; }

// TCHAR-era aliases of structs the island does carry.
pub type LV_ITEMW = LVITEMW;

pub type _EDITTYPE_DIALOG = EDITTYPE_DIALOG;
unsafe extern "C" fn GetRegDWORD(
    hKey: HKEY,
    Name: LPCWSTR,
    Value: *mut DWORD,
    DefaultValue: DWORD,
    Strict: BOOL,
) -> DWORD {
    let mut cb: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut type_: DWORD = 0;
    let ec: LRESULT = RegQueryValueExW(
        hKey,
        Name,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(type_),
        Value as *mut u8,
        core::ptr::addr_of_mut!(cb),
    ) as LRESULT;

    if ec == ERROR_SUCCESS as LRESULT {
        if (type_ == REG_DWORD && cb == core::mem::size_of::<DWORD>() as DWORD)
            || (Strict == 0
                && type_ == REG_BINARY
                && cb != 0
                && cb <= core::mem::size_of::<DWORD>() as DWORD)
        {
            *Value &= 0xffffffffu32 >> (32 - cb * 8);
            return ec as DWORD;
        }
    }

    *Value = DefaultValue;
    if ec != 0 {
        ec as DWORD
    } else {
        ERROR_BAD_FORMAT
    }
}

unsafe fn GetRegDWORD_2(hKey: HKEY, Name: LPCWSTR, mut DefaultValue: DWORD) -> DWORD {
    GetRegDWORD(hKey, Name, core::ptr::addr_of_mut!(DefaultValue), DefaultValue, FALSE);
    DefaultValue
}
unsafe extern "C" fn GetClassKey(FTE: *const _FILE_TYPE_ENTRY, SubKey: *mut LPCWSTR) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut path: LPCWSTR = if (*(FTE)).FileExtension[0] == '.' as u16 {
        (*(FTE)).ClassKey.as_ptr()
    } else {
        (*(FTE)).FileExtension.as_ptr()
    };
    if *path == 0 && (*(FTE)).FileExtension[0] == '.' as u16 {
        path = (*(FTE)).FileExtension.as_ptr();
        hr = S_FALSE;
    }
    *SubKey = path;
    hr
}
unsafe extern "C" fn QuoteAppPathForCommand(path: &mut CStringW) {
    if path.Find(' ' as u16, 0) >= 0 && path.Find('"' as u16, 0) < 0 {
        // C++: `path = CStringW(L"\"") + path + L"\"";`. The port has no `operator+`, only
        // `Append` (= `operator+=`), so the concatenation is built left to right in a
        // temporary. It MUST be a temporary: appending `path` to itself would realloc the
        // very buffer being read, which `Append`'s own docs rule out.
        let mut quoted = CStringW::from_ptr(CPP_COM_WSTR_1B9334FEECE6.as_ptr());
        quoted.Append(path.as_ptr());
        quoted.Append(CPP_COM_WSTR_1B9334FEECE6.as_ptr());
        *path = quoted;
    }
}
unsafe extern "C" fn DeleteExt(hwndDlg: HWND, pszExt: LPCWSTR) -> BOOL {
    unsafe {
        let _ = hwndDlg;
        if *pszExt != 0x002Eu16 {
            return FALSE;
        }

        let mut hKey: HKEY = core::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            pszExt,
            0,
            KEY_READ,
            &mut hKey,
        ) != ERROR_SUCCESS as INT
        {
            return FALSE;
        }

        let mut ProgId = [0u16; MAX_PATH as usize];
        let mut cb = core::mem::size_of_val(&ProgId) as u32;
        RegQueryValueExW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (ProgId.as_mut_ptr()) as LPBYTE,
            &mut cb,
        );
        RegCloseKey(hKey);

        if ProgId[0 as usize] != 0 {
            SHDeleteKeyW(HKEY_CLASSES_ROOT, ProgId.as_ptr());
        }

        let ret = (SHDeleteKeyW(HKEY_CLASSES_ROOT, pszExt) == ERROR_SUCCESS as DWORD) as BOOL;

        SHChangeNotify(
            SHCNE_ASSOCCHANGED as INT,
            SHCNF_FLUSHNOWAIT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        ret
    }
}

unsafe extern "C" fn DoExtractIcon(
    IconPath: LPCWSTR,
    iIndex: INT,
    cx: UINT,
    cy: UINT,
) -> HICON {
    SHELL32_SHExtractIcon(IconPath, iIndex, cx, cy)
}

unsafe fn DoExtractIcon_2(IconPath: LPCWSTR, iIndex: INT, bSmall: BOOL) -> HICON {
    let cx = GetSystemMetrics(if bSmall != 0 { SM_CXSMICON as INT } else { SM_CXICON as INT }) as UINT;
    let cy = GetSystemMetrics(if bSmall != 0 { SM_CYSMICON as INT } else { SM_CYICON as INT }) as UINT;
    DoExtractIcon(IconPath, iIndex, cx, cy)
}
unsafe extern "C" fn GetFileTypeIconsEx(
    Entry: PFILE_TYPE_ENTRY,
    IconLocation: LPCWSTR,
    IconSize: UINT,
) -> BOOL {
    (*Entry).hIconSmall = core::ptr::null_mut();
    if lstrcmpW(IconLocation, CPP_COM_WSTR_7C8F589C7B27.as_ptr()) == 0 {
        let ext = (*Entry).FileExtension.as_ptr();
        if lstrcmpiW(ext, CPP_COM_WSTR_7ABCE230ED71.as_ptr()) == 0
            || lstrcmpiW(ext, CPP_COM_WSTR_B730BA9299D0.as_ptr()) == 0
        {
            (*Entry).hIconSmall = (LoadImageW(
                shell32_hInstance,
                MAKEINTRESOURCEW(IDI_SHELL_EXE),
                IMAGE_ICON,
                IconSize as ::core::ffi::c_int,
                IconSize as ::core::ffi::c_int,
                0,
            )) as HICON;
            (*Entry).nIconIndex = -((IDI_SHELL_EXE) as INT);
        }
        StringCbCopyW(
            (*Entry).IconPath.as_mut_ptr(),
            core::mem::size_of_val(&(*Entry).IconPath),
            IconLocation,
        );
    } else {
        if ExpandEnvironmentStringsW(
            IconLocation,
            (*Entry).IconPath.as_mut_ptr(),
            ((*Entry).IconPath).len() as DWORD,
        ) != 0
        {
            (*Entry).nIconIndex = PathParseIconLocationW((*Entry).IconPath.as_mut_ptr());
            (*Entry).hIconSmall = DoExtractIcon(
                (*Entry).IconPath.as_ptr(),
                (*Entry).nIconIndex,
                IconSize,
                IconSize,
            );
        }
    }
    if (*Entry).hIconSmall != core::ptr::null_mut() {
        1
    } else {
        0
    }
}
unsafe extern "C" fn GetFileTypeIconsByKey(
    hKey: HKEY,
    Entry: PFILE_TYPE_ENTRY,
    IconSize: u32,
) -> BOOL {
    (*Entry).hIconSmall = core::ptr::null_mut();

    let mut hDefIconKey: HKEY = core::ptr::null_mut();
    let mut nResult: i32 = RegOpenKeyExW(
        hKey,
        CPP_COM_WSTR_3BE8B25B7ED1.as_ptr(),
        0 as u32,
        KEY_READ as u32,
        &mut hDefIconKey,
    );
    if nResult != ERROR_SUCCESS as i32 {
        return FALSE as BOOL;
    }

    let mut szLocation = [0u16; ICONLOCATION_CCHMAX as usize];
    let mut dwSize: u32 = core::mem::size_of_val(&szLocation) as u32;
    nResult = RegQueryValueExW(
        hDefIconKey,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (szLocation.as_mut_ptr()) as LPBYTE,
        &mut dwSize,
    );
    szLocation[(szLocation.len()) - (1 as usize)] =
        UNICODE_NULL as u16;

    RegCloseKey(hDefIconKey);
    if nResult != ERROR_SUCCESS as i32 || szLocation[0 as usize] == 0 {
        return FALSE as BOOL;
    }

    GetFileTypeIconsEx(Entry, szLocation.as_ptr(), IconSize)
}
unsafe extern "C" fn GetProgramPath(Entry: *mut _FILE_TYPE_ENTRY) -> *const u16 {
    if (*Entry).ProgramPath[1] == 0 && (*Entry).ProgramPath[0] == 0 {
        let mut cch: u32 = ((*Entry).ProgramPath).len() as u32;
        if AssocQueryStringW(
            ASSOCF_INIT_IGNOREUNKNOWN,
            ASSOCSTR_EXECUTABLE,
            core::ptr::addr_of!((*Entry).FileExtension) as *const u16,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*Entry).ProgramPath) as *mut u16,
            &mut cch,
        ) < 0
        {
            (*Entry).ProgramPath[0] = UNICODE_NULL;
            (*Entry).ProgramPath[1] = TRUE as WORD;
        }
    }
    core::ptr::addr_of!((*Entry).ProgramPath) as *const u16
}
unsafe extern "C" fn QueryFileDescription(ProgramPath: LPCWSTR, pszName: LPWSTR, cchName: INT) -> BOOL {
    let mut fi: SHFILEINFOW = core::mem::zeroed();
    fi.szDisplayName[0] = UNICODE_NULL;
    if SHGetFileInfoW(
        ProgramPath,
        0,
        core::ptr::addr_of_mut!(fi),
        core::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_DISPLAYNAME,
    ) != 0
    {
        StringCchCopyW(pszName, cchName as usize, core::ptr::addr_of!(fi.szDisplayName) as LPCWSTR);
        return TRUE;
    }
    return (GetFileTitleW(ProgramPath, pszName, cchName as u16) != 0) as BOOL;
}
unsafe extern "C" fn GetAppName(Entry: *mut _FILE_TYPE_ENTRY) -> *const u16 {
    if (*Entry).AppName[1] == 0 && (*Entry).AppName[0] == 0 {
        let exe = GetProgramPath(Entry);
        if *exe == 0 || QueryFileDescription(exe, (*Entry).AppName.as_mut_ptr(), ((*Entry).AppName).len() as INT) == 0 {
            (*Entry).AppName[0] = UNICODE_NULL;
            (*Entry).AppName[1] = TRUE as WORD;
        }
    }
    (*Entry).AppName.as_ptr()
}
unsafe extern "C" fn GetTypeName(Entry: PFILE_TYPE_ENTRY, pG: PFILE_TYPE_GLOBALS) -> LPWSTR {
    if (*Entry).FileDescription[1] == 0 && (*Entry).FileDescription[0] == 0 {
        (*Entry).FileDescription[1] = TRUE as WORD;
        if (*Entry).FileExtension[0] == '.' as u16 {
            let mut fi: SHFILEINFOW = core::mem::zeroed();
            if SHGetFileInfoW(
                (*Entry).FileExtension.as_ptr(),
                0,
                &mut fi,
                core::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
            ) != 0 && fi.szTypeName[0] != 0
            {
                StringCchCopyW(
                    (*Entry).FileDescription.as_mut_ptr(),
                    ((*Entry).FileDescription).len(),
                    core::ptr::addr_of!(fi.szTypeName) as LPCWSTR,
                );
            }
        } else {
            let mut cb: u32 = core::mem::size_of_val(&(*Entry).FileDescription) as u32;
            let mut Fallback: u32 = TRUE as UINT;
            let mut ClassKey: LPCWSTR = core::ptr::null_mut();
            let hr: HRESULT = GetClassKey(Entry as *const _FILE_TYPE_ENTRY, &mut ClassKey);
            let mut hKey: HKEY = core::ptr::null_mut();
            if hr >= 0 && RegOpenKeyExW(
                HKEY_CLASSES_ROOT,
                ClassKey,
                0,
                KEY_READ,
                &mut hKey,
            ) == 0
            {
                Fallback = (RegQueryValueExW(
                    hKey,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*Entry).FileDescription.as_mut_ptr() as *mut u8,
                    &mut cb,
                ) != ERROR_SUCCESS as INT || (*Entry).FileDescription[0] == 0) as u32;
                RegCloseKey(hKey);
            }
            if Fallback != 0 {
                StringCchCopyW(
                    (*Entry).FileDescription.as_mut_ptr(),
                    ((*Entry).FileDescription).len(),
                    (*Entry).FileExtension.as_ptr(),
                );
            }
        }
    }
    (*Entry).FileDescription.as_mut_ptr()
}
unsafe extern "C" fn InitializeDefaultIcons(pG: PFILE_TYPE_GLOBALS) {
    let ResId: i32 = NOASSOCRESID as INT;
    if (*pG).hDefExtIconSmall.is_null() {
        (*pG).hDefExtIconSmall = (LoadImageW(
            shell32_hInstance,
            MAKEINTRESOURCEW(ResId as UINT),
            IMAGE_ICON,
            (*pG).IconSize as ::core::ffi::c_int,
            (*pG).IconSize as ::core::ffi::c_int,
            0,
        )) as HICON;
    }
    if ImageList_GetImageCount((*pG).himlSmall) == 0 {
        let idx: i32 = ImageList_AddIcon!((*pG).himlSmall, (*pG).hDefExtIconSmall);
        DBG_UNREFERENCED_LOCAL_VARIABLE!(idx);
    }
}
unsafe extern "C" fn Normalize(Entry: PFILE_TYPE_ENTRY) -> BOOL {
    if (*Entry).IconPath[0] == 0 {
        StringCbCopyW(
            (*Entry).IconPath.as_mut_ptr(),
            core::mem::size_of::<[u16; MAX_PATH as usize]>(),
            g_pszShell32,
        );
        (*Entry).nIconIndex = if NOASSOCRESID > 1 {
            -((NOASSOCRESID) as INT)
        } else {
            0
        };
    }
    TRUE
}
unsafe extern "C" fn EditTypeDlg_OnChangeIcon(
    hwndDlg: HWND,
    pEditType: PEDITTYPE_DIALOG,
) {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut IconIndex: INT = (*pEditType).nIconIndex;

    ExpandEnvironmentStringsW(
        (*pEditType).szIconPath.as_ptr(),
        szPath.as_mut_ptr(),
        szPath.len() as UINT,
    );

    if PickIconDlg(
        hwndDlg,
        szPath.as_mut_ptr(),
        szPath.len() as UINT,
        core::ptr::addr_of_mut!(IconIndex),
    ) != 0
    {
        let hIconLarge: HICON = DoExtractIcon_2(
            szPath.as_ptr(),
            IconIndex,
            FALSE,
        );

        let mut szWinDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let lenWinDir: UINT = GetWindowsDirectoryW(
            szWinDir.as_mut_ptr(),
            szWinDir.len() as UINT,
        );

        if (StrStrIW(
            szPath.as_ptr(),
            szWinDir.as_ptr(),
        ) as *const u16) == szPath.as_ptr()
        {
            let mut str = CPathW::from_ptr(CPP_COM_WSTR_2B585AE3D008.as_ptr());
            str.Append(szPath.as_ptr().add(lenWinDir as usize));
            StringCbCopyW(
                szPath.as_mut_ptr(),
                core::mem::size_of::<[u16; MAX_PATH as usize]>(),
                str.as_ptr(),
            );
        }

        StringCbCopyW(
            (*pEditType).szIconPath.as_mut_ptr(),
            core::mem::size_of::<[u16; MAX_PATH as usize]>(),
            szPath.as_ptr(),
        );
        (*pEditType).nIconIndex = IconIndex;
        (*pEditType).ChangedIcon = true;

        let hOld: HICON = SendDlgItemMessageW(
            hwndDlg,
            IDC_EDITTYPE_ICON as INT,
            STM_SETICON,
            hIconLarge as WPARAM,
            0,
        ) as HICON;
        if hOld != core::ptr::null_mut() {
            DestroyIcon(hOld);
        }
    }
}
unsafe extern "C" fn EditTypeDlg_OnDrawItem(
    hwndDlg: HWND,
    pDraw: LPDRAWITEMSTRUCT,
    pEditType: PEDITTYPE_DIALOG,
) -> BOOL {
    let mut szText = [0u16; MAX_PATH as usize];
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hFont2: HFONT = core::ptr::null_mut();

    if pDraw.is_null() {
        return FALSE;
    }

    if ((*pDraw).itemState & ODS_SELECTED) != 0 {
        FillRect(
            (*pDraw).hDC,
            core::ptr::addr_of_mut!((*pDraw).rcItem),
            (COLOR_HIGHLIGHT + 1) as HBRUSH,
        );
        SetTextColor((*pDraw).hDC, GetSysColor(COLOR_HIGHLIGHTTEXT as INT));
        SetBkColor((*pDraw).hDC, GetSysColor(COLOR_HIGHLIGHT as INT));
    } else {
        FillRect(
            (*pDraw).hDC,
            core::ptr::addr_of_mut!((*pDraw).rcItem),
            (COLOR_WINDOW + 1) as HBRUSH,
        );
        SetTextColor((*pDraw).hDC, GetSysColor(COLOR_WINDOWTEXT as INT));
        SetBkColor((*pDraw).hDC, GetSysColor(COLOR_WINDOW as INT));
    }

    let hwndListBox: HWND = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
    SendMessageW(
        hwndListBox,
        LB_GETTEXT,
        (*pDraw).itemID as WPARAM,
        core::ptr::addr_of_mut!(szText) as *mut u16 as LPARAM,
    );

    hFont = SendMessageW(
        hwndListBox,
        WM_GETFONT,
        0,
        0,
    ) as HFONT;

    if lstrcmpiW(
        core::ptr::addr_of!((*pEditType).szDefaultVerb) as *const u16,
        core::ptr::addr_of!(szText) as *const u16,
    ) == 0
    {
        let mut lf: LOGFONTW = core::mem::zeroed();

        GetObject(
            hFont as HANDLE,
            core::mem::size_of::<LOGFONTW>() as i32,
            core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
        );

        lf.lfWeight = FW_BOLD as INT;

        hFont2 = CreateFontIndirectW(core::ptr::addr_of!(lf));

        if !hFont2.is_null() {
            let hFontOld: HGDIOBJ = SelectObject((*pDraw).hDC, hFont2 as HGDIOBJ);

            InflateRect(
                core::ptr::addr_of_mut!((*pDraw).rcItem),
                -LISTBOX_MARGIN,
                -LISTBOX_MARGIN,
            );

            DrawTextW(
                (*pDraw).hDC,
                core::ptr::addr_of!(szText) as *const u16,
                -1,
                core::ptr::addr_of_mut!((*pDraw).rcItem),
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );

            InflateRect(
                core::ptr::addr_of_mut!((*pDraw).rcItem),
                LISTBOX_MARGIN,
                LISTBOX_MARGIN,
            );

            SelectObject((*pDraw).hDC, hFontOld);

            DeleteObject(hFont2 as HGDIOBJ);
        }
    } else {
        InflateRect(
            core::ptr::addr_of_mut!((*pDraw).rcItem),
            -LISTBOX_MARGIN,
            -LISTBOX_MARGIN,
        );

        DrawTextW(
            (*pDraw).hDC,
            core::ptr::addr_of!(szText) as *const u16,
            -1,
            core::ptr::addr_of_mut!((*pDraw).rcItem),
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );

        InflateRect(
            core::ptr::addr_of_mut!((*pDraw).rcItem),
            LISTBOX_MARGIN,
            LISTBOX_MARGIN,
        );
    }

    if ((*pDraw).itemState & ODS_FOCUS) != 0 {
        DrawFocusRect((*pDraw).hDC, core::ptr::addr_of_mut!((*pDraw).rcItem));
    }

    TRUE
}
unsafe extern "C" fn EditTypeDlg_OnMeasureItem(
    hwndDlg: HWND,
    pMeasure: LPMEASUREITEMSTRUCT,
    pEditType: PEDITTYPE_DIALOG,
) -> BOOL {
    if pMeasure.is_null() {
        return FALSE;
    }

    let hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);

    let hDC = GetDC(hwndLB);
    if !hDC.is_null() {
        let mut tm: TEXTMETRICW = core::mem::zeroed();
        GetTextMetricsW(hDC, core::ptr::addr_of_mut!(tm));
        (*pMeasure).itemHeight =
            (tm.tmHeight + LISTBOX_MARGIN * 2) as u32;
        ReleaseDC(hwndLB, hDC);
        return TRUE;
    }
    FALSE
}
unsafe extern "C" fn NewExtDlg_OnAdvanced(hwndDlg: HWND, pNewExt: PNEWEXT_DIALOG) {
    let mut rc: RECT = core::mem::zeroed();
    let mut rc1: RECT = core::mem::zeroed();
    let mut rc2: RECT = core::mem::zeroed();

    GetWindowRect(hwndDlg, core::ptr::addr_of_mut!(rc));
    rc.bottom = rc.top + ((*pNewExt).rcDlg.bottom - (*pNewExt).rcDlg.top);

    GetWindowRect(
        GetDlgItem(hwndDlg, IDOK as INT),
        core::ptr::addr_of_mut!(rc1)
    );
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        core::ptr::addr_of_mut!(rc1) as *mut _,
        2
    );

    GetWindowRect(
        GetDlgItem(hwndDlg, IDCANCEL as INT),
        core::ptr::addr_of_mut!(rc2)
    );
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        core::ptr::addr_of_mut!(rc2) as *mut _,
        2
    );

    let hClassCombo = GetDlgItem(hwndDlg, IDC_NEWEXT_COMBOBOX as INT);

    if (*pNewExt).bAdvanced != 0 {
        rc1.top += (*pNewExt).dy;
        rc1.bottom += (*pNewExt).dy;
        rc2.top += (*pNewExt).dy;
        rc2.bottom += (*pNewExt).dy;

        ShowWindow(
            GetDlgItem(hwndDlg, IDC_NEWEXT_ASSOC as INT),
            SW_SHOWNOACTIVATE as INT
        );
        ShowWindow(hClassCombo, SW_SHOWNOACTIVATE as INT);

        let strLeft = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_NEWEXT_ADVANCED_LEFT));
        SetDlgItemTextW(hwndDlg, IDC_NEWEXT_ADVANCED as INT, strLeft.as_ptr());

        SetFocus(hClassCombo);
    } else {
        rc1.top -= (*pNewExt).dy;
        rc1.bottom -= (*pNewExt).dy;
        rc2.top -= (*pNewExt).dy;
        rc2.bottom -= (*pNewExt).dy;

        ShowWindow(
            GetDlgItem(hwndDlg, IDC_NEWEXT_ASSOC as INT),
            SW_HIDE as INT
        );
        ShowWindow(hClassCombo, SW_HIDE as INT);

        let strRight = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_NEWEXT_ADVANCED_RIGHT));
        SetDlgItemTextW(hwndDlg, IDC_NEWEXT_ADVANCED as INT, strRight.as_ptr());

        rc.bottom -= (*pNewExt).dy;

        SendMessageW(
            hClassCombo,
            CB_SETCURSEL,
            0,
            0
        );
    }

    let mut hDWP = BeginDeferWindowPos(3);

    if hDWP != core::ptr::null_mut() {
        hDWP = DeferWindowPos(
            hDWP,
            GetDlgItem(hwndDlg, IDOK as INT),
            core::ptr::null_mut(),
            rc1.left, rc1.top, rc1.right - rc1.left, rc1.bottom - rc1.top,
            SWP_NOACTIVATE | SWP_NOZORDER
        );
    }
    if hDWP != core::ptr::null_mut() {
        hDWP = DeferWindowPos(
            hDWP,
            GetDlgItem(hwndDlg, IDCANCEL as INT),
            core::ptr::null_mut(),
            rc2.left, rc2.top, rc2.right - rc2.left, rc2.bottom - rc2.top,
            SWP_NOACTIVATE | SWP_NOZORDER
        );
    }
    if hDWP != core::ptr::null_mut() {
        hDWP = DeferWindowPos(
            hDWP,
            hwndDlg,
            core::ptr::null_mut(),
            rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top,
            SWP_NOACTIVATE | SWP_NOZORDER
        );
    }

    if hDWP != core::ptr::null_mut() {
        EndDeferWindowPos(hDWP);
    }
}
unsafe extern "C" fn NewExtDlg_OnInitDialog(hwndDlg: HWND, pNewExt: PNEWEXT_DIALOG) -> BOOL {
    (*pNewExt).bAdvanced = FALSE;

    GetWindowRect(hwndDlg, core::ptr::addr_of_mut!((*pNewExt).rcDlg));

    let mut rc1: RECT = core::mem::zeroed();
    let mut rc2: RECT = core::mem::zeroed();
    GetWindowRect(GetDlgItem(hwndDlg, IDC_NEWEXT_EDIT as INT), core::ptr::addr_of_mut!(rc1));
    GetWindowRect(GetDlgItem(hwndDlg, IDC_NEWEXT_COMBOBOX as INT), core::ptr::addr_of_mut!(rc2));
    (*pNewExt).dy = rc2.top - rc1.top;

    let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_NEWEXT_NEW));
    SendDlgItemMessageW(hwndDlg, IDC_NEWEXT_COMBOBOX as INT, CB_ADDSTRING, 0, strText.as_ptr() as LPARAM);
    SendDlgItemMessageW(hwndDlg, IDC_NEWEXT_COMBOBOX as INT, CB_SETCURSEL, 0, 0);
    SendDlgItemMessageW(hwndDlg, IDC_NEWEXT_EDIT as ::core::ffi::c_int, EM_SETLIMITTEXT, (((*pNewExt).szExt).len() - 1) as WPARAM, 0);

    NewExtDlg_OnAdvanced(hwndDlg, pNewExt);

    return TRUE;
}
unsafe extern "C" fn NewExtDlg_OnOK(hwndDlg: HWND, pNewExt: *mut NEWEXT_DIALOG) -> BOOL {
    let mut find: LV_FINDINFO = core::mem::zeroed();
    let mut iItem: i32;

    GetDlgItemTextW(hwndDlg, IDC_NEWEXT_EDIT as ::core::ffi::c_int, (*pNewExt).szExt.as_mut_ptr(), ((*pNewExt).szExt).len() as ::core::ffi::c_int);
    StrTrimW((*pNewExt).szExt.as_mut_ptr(), g_pszSpace);
    _wcsupr((*pNewExt).szExt.as_mut_ptr());

    (*pNewExt).szFileType[0 as usize] = UNICODE_NULL;

    if (*pNewExt).szExt[0 as usize] == 0 {
        let mut strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_NEWEXT_SPECIFY_EXT));
        let mut strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
        MessageBoxW(hwndDlg, strText.as_ptr(), strTitle.as_ptr(), MB_ICONERROR);
        return FALSE;
    }

    ZeroMemory(core::ptr::addr_of_mut!(find) as *mut core::ffi::c_void, core::mem::size_of::<LV_FINDINFO>());
    find.flags = LVFI_STRING;
    if (*pNewExt).szExt[0 as usize] == 0x002Eu16 {
        find.psz = core::ptr::addr_of_mut!((*pNewExt).szExt[1 as usize]);
    } else {
        find.psz = (*pNewExt).szExt.as_mut_ptr();
    }

    iItem = ListView_FindItem((*pNewExt).hwndLV, -(1 as i32), &find);
    if iItem >= 0 as i32 {
        let mut szFileType: [u16; TYPENAME_CCHMAX as usize] = [0u16; TYPENAME_CCHMAX as usize];
        let mut item: LV_ITEM = core::mem::zeroed();
        ZeroMemory(core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
        item.mask = LVIF_TEXT;
        item.pszText = szFileType.as_mut_ptr();
        item.cchTextMax = szFileType.len() as i32;
        item.iItem = iItem;
        item.iSubItem = 1;
        ListView_GetItem((*pNewExt).hwndLV, &mut item);

        let mut strText = CStringW::default();
        strText.FormatResStr4(IDS_NEWEXT_ALREADY_ASSOC, find.psz as *const u16, szFileType.as_ptr(), find.psz as *const u16, szFileType.as_ptr());

        let mut strTitle = CStringW::default();
        strTitle.LoadStringW(IDS_NEWEXT_EXT_IN_USE);

        if MessageBoxW(hwndDlg, strText.as_ptr(), strTitle.as_ptr(), MB_ICONWARNING | MB_YESNO) == IDNO as INT {
            return FALSE;
        }

        let mut strExt = CStringW::from_ptr(CPP_COM_WSTR_C245AFF91259.as_ptr());
        strExt += find.psz as *const u16;
        strExt.MakeLower();
        if DeleteExt(hwndDlg, strExt.as_ptr()) != 0 {
            ListView_DeleteItem((*pNewExt).hwndLV, iItem);
        }
    }

    EndDialog(hwndDlg, IDOK as INT_PTR);
    return TRUE;
}

unsafe extern "system" fn NewExtDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    static mut s_pNewExt: PNEWEXT_DIALOG = core::ptr::null_mut();

    match uMsg {
        WM_INITDIALOG => {
            unsafe {
                s_pNewExt = lParam as PNEWEXT_DIALOG;
                NewExtDlg_OnInitDialog(hwndDlg, s_pNewExt);
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => unsafe {
            match LOWORD!((wParam) as DWORD) {
                IDOK => {
                    NewExtDlg_OnOK(hwndDlg, s_pNewExt);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                }
                IDC_NEWEXT_ADVANCED => {
                    (*s_pNewExt).bAdvanced = if (*s_pNewExt).bAdvanced == 0 { 1 } else { 0 };
                    NewExtDlg_OnAdvanced(hwndDlg, s_pNewExt);
                }
                _ => {}
            }
        },
        _ => {}
    };

    0
}
unsafe extern "C" fn FileTypesDlg_InsertToLV(hListView: HWND, Assoc: LPCWSTR, iItem: i32, pG: PFILE_TYPE_GLOBALS) -> PFILE_TYPE_ENTRY {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hTemp: HKEY = core::ptr::null_mut();
    let Entry: PFILE_TYPE_ENTRY;
    let mut dwSize: u32;

    if RegOpenKeyExW(HKEY_CLASSES_ROOT, Assoc, 0, KEY_READ, core::ptr::addr_of_mut!(hKey)) != ERROR_SUCCESS as INT {
        return core::ptr::null_mut();
    }
    if *Assoc.add(0 as usize) != 0x002Eu16 && !RegValueExists(hKey, CPP_COM_WSTR_07E190C927BB.as_ptr()) {
        RegCloseKey(hKey);
        return core::ptr::null_mut();
    }

    Entry = HeapAlloc((*pG).hHeap, 0, core::mem::size_of::<FILE_TYPE_ENTRY>() as SIZE_T) as PFILE_TYPE_ENTRY;
    if Entry.is_null() {
        RegCloseKey(hKey);
        return core::ptr::null_mut();
    }
    crate::dialogs_filetypes::_FILE_TYPE_ENTRY_Initialize_body(Entry.cast());

    'exclude: {
        if *Assoc.add(0 as usize) == 0x002Eu16 {
            if PathIsExeW(Assoc) != 0 {
                break 'exclude;
            }

            dwSize = core::mem::size_of::<[u16; MAX_PATH as usize]>() as u32;
            if RegQueryValueExW(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), ((*Entry).ClassKey.as_mut_ptr()) as LPBYTE, core::ptr::addr_of_mut!(dwSize)) != 0 {
                (*Entry).ClassKey[0 as usize] = UNICODE_NULL;
            }
            if (*Entry).ClassKey[0 as usize] != 0 && RegOpenKeyExW(HKEY_CLASSES_ROOT, (*Entry).ClassKey.as_mut_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hTemp)) == 0 {
                RegCloseKey(hKey);
                hKey = hTemp;
            }
        }

        (*Entry).EditFlags = GetRegDWORD_2(hKey, CPP_COM_WSTR_A2E2913C930F.as_ptr(), 0);
        if ((*Entry).EditFlags & FTA_Exclude) != 0 {
            break 'exclude;
        }

        wcscpy((*Entry).FileExtension.as_mut_ptr(), Assoc);

        (*Entry).IconPath[0 as usize] = UNICODE_NULL;
        let defaultIcon: bool = GetFileTypeIconsByKey(hKey, Entry, (*pG).IconSize) == 0;

        RegCloseKey(hKey);

        let mut iSmallImage: i32 = 0;
        if !defaultIcon && !(*Entry).hIconSmall.is_null() {
            iSmallImage = ImageList_AddIcon!((*pG).himlSmall, (*Entry).hIconSmall);
        }

        let mut lvItem: LVITEMW = core::mem::zeroed();
        lvItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_IMAGE;
        lvItem.iItem = iItem;
        lvItem.iSubItem = 0;
        lvItem.lParam = Entry as LPARAM;
        lvItem.iImage = iSmallImage;
        lvItem.pszText = if *Assoc.add(0 as usize) == 0x002Eu16 {
            _wcsupr((*Entry).FileExtension.as_mut_ptr().add(1 as usize))
        } else {
            (*pG).NoneString.as_mut_ptr()
        };
        SendMessageW(hListView, LVM_INSERTITEMW, 0, core::ptr::addr_of_mut!(lvItem) as LPARAM);

        lvItem.mask = LVIF_TEXT;
        lvItem.iItem = iItem;
        lvItem.iSubItem = 1;
        lvItem.pszText = LPSTR_TEXTCALLBACK;
        ListView_SetItem(hListView, &mut lvItem);

        return Entry;
    }

    HeapFree((*pG).hHeap, 0, Entry as PVOID);
    RegCloseKey(hKey);
    return core::ptr::null_mut();
}
unsafe extern "C" fn FileTypesDlg_AddExt(
    hwndDlg: HWND,
    mut pszExt: LPCWSTR,
    mut pszProgId: LPCWSTR,
    pG: PFILE_TYPE_GLOBALS,
) -> BOOL {
    let mut dwValue: DWORD = 1;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut szKey: [u16; 13] = [0u16; 13];
    let mut nResult: LONG;

    if *pszProgId == 0 {
        pszProgId = szKey.as_ptr();

        loop {
            StringCbPrintfW!(
                szKey.as_mut_ptr(),
                core::mem::size_of_val(&szKey),
                CPP_COM_WSTR_8419E6E00D5A.as_ptr(),
                dwValue,
            );

            nResult = RegOpenKeyEx(
                HKEY_CLASSES_ROOT,
                szKey.as_ptr(),
                0,
                KEY_READ,
                core::ptr::addr_of_mut!(hKey),
            );
            if nResult != ERROR_SUCCESS as INT {
                break;
            }

            RegCloseKey(hKey);
            dwValue = dwValue.wrapping_add(1);
            if dwValue == 0 {
                break;
            }
        }

        RegCloseKey(hKey);

        if dwValue == 0 {
            return FALSE;
        }

        nResult = RegCreateKeyEx(
            HKEY_CLASSES_ROOT,
            szKey.as_ptr(),
            0,
            core::ptr::null_mut(),
            0,
            KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hKey),
            core::ptr::null_mut(),
        );
        if ERROR_SUCCESS as LONG != nResult {
            return FALSE;
        }
        RegCloseKey(hKey);
    }

    let mut szExt: [u16; ASSOC_CCHMAX] = [0u16; ASSOC_CCHMAX];
    if *pszExt == 0x002Eu16 {
        pszExt = pszExt.add(1);
    }
    StringCbPrintfW!(
        szExt.as_mut_ptr(),
        core::mem::size_of_val(&szExt),
        CPP_COM_WSTR_5BFECD9F5862.as_ptr(),
        pszExt,
    );
    _wcslwr(szExt.as_mut_ptr());
    nResult = RegCreateKeyEx(
        HKEY_CLASSES_ROOT,
        szExt.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    );
    _wcsupr(szExt.as_mut_ptr());
    if ERROR_SUCCESS as LONG != nResult {
        return FALSE;
    }

    RegSetString(hKey, core::ptr::null(), pszProgId, REG_SZ);

    RegCloseKey(hKey);

    let hListView: HWND = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);
    let iItem: INT = ListView_GetItemCount(hListView);
    if FileTypesDlg_InsertToLV(hListView, szExt.as_ptr(), iItem, pG).is_null() {
        return FALSE;
    }

    ListView_SetItemState(
        hListView,
        iItem,
        -1i32 as UINT,
        (LVIS_FOCUSED | LVIS_SELECTED) as u32,
    );
    ListView_EnsureVisible(hListView, iItem, FALSE);
    return TRUE;
}
unsafe extern "C" fn FileTypesDlg_RemoveExt(hwndDlg: HWND) -> BOOL {
    let hListView = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);

    let iItem: INT = ListView_GetNextItem(hListView, -1, LVNI_SELECTED);
    if iItem == -1 {
        return FALSE;
    }

    let mut szExt: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    szExt[0] = 0x002Eu16;
    ListView_GetItemText(
        hListView,
        iItem,
        0,
        core::ptr::addr_of_mut!(szExt[1]),
        (szExt.len() - 1) as INT,
    );

    _wcslwr(szExt.as_mut_ptr());

    if DeleteExt(hwndDlg, szExt.as_ptr()) != 0 {
        ListView_DeleteItem(hListView, iItem);
    }

    return TRUE;
}
unsafe extern "C" fn ActionDlg_OnBrowse(hwndDlg: HWND, pNewAct: PACTION_DIALOG, bEdit: BOOL) {
    let mut szFile = [0u16; MAX_PATH as usize];
    szFile[0] = UNICODE_NULL;

    let mut szFilter = [0u16; MAX_PATH as usize];
    LoadStringW(shell32_hInstance, IDS_EXE_FILTER, szFilter.as_mut_ptr(), szFilter.len() as INT);

    let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_OPEN_WITH));

    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    ZeroMemory(core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAMEW>());
    ofn.lStructSize = OPENFILENAME_SIZE_VERSION_400W;
    ofn.hwndOwner = hwndDlg;
    ofn.lpstrFilter = szFilter.as_ptr();
    ofn.lpstrFile = szFile.as_mut_ptr();
    ofn.nMaxFile = szFile.len() as DWORD;
    ofn.lpstrTitle = strTitle.as_ptr();
    ofn.Flags = OFN_EXPLORER | OFN_ENABLESIZING | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;
    ofn.lpstrDefExt = CPP_COM_WSTR_D5B988E0271D.as_ptr();

    if GetOpenFileNameW(&mut ofn) != 0 {
        if bEdit != 0 {
            let mut str = CStringW::from_ptr(szFile.as_ptr());
            QuoteAppPathForCommand(&mut str);
            str += CPP_COM_WSTR_2ED82ED19E49.as_ptr();
            SetDlgItemTextW(hwndDlg, IDC_ACTION_APP as INT, str.as_ptr());
        } else {
            SetDlgItemTextW(hwndDlg, IDC_ACTION_APP as INT, szFile.as_ptr());
        }
    }
}
unsafe extern "C" fn NewActionDlg_OnOK(hwndDlg: HWND, pNewAct: PACTION_DIALOG) {
    unsafe {
        GetDlgItemTextW(
            hwndDlg,
            IDC_ACTION_ACTION as ::core::ffi::c_int,
            (*pNewAct).szAction.as_mut_ptr(),
            ((*pNewAct).szAction).len() as ::core::ffi::c_int,
        );
        StrTrimW((*pNewAct).szAction.as_mut_ptr(), g_pszSpace);
        if (*pNewAct).szAction[0] == 0 {
            let hwndCtrl = GetDlgItem(hwndDlg, IDC_ACTION_ACTION as INT);
            SendMessageW(
                hwndCtrl,
                EM_SETSEL,
                0,
                -1,
            );
            SetFocus(hwndCtrl);
            let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_SPECIFY_ACTION));
            let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
            MessageBoxW(
                hwndDlg,
                strText.as_ptr(),
                strTitle.as_ptr(),
                MB_ICONERROR,
            );
            return;
        }

        GetDlgItemTextW(
            hwndDlg,
            IDC_ACTION_APP as ::core::ffi::c_int,
            (*pNewAct).szApp.as_mut_ptr(),
            ((*pNewAct).szApp).len() as ::core::ffi::c_int,
        );
        StrTrimW((*pNewAct).szApp.as_mut_ptr(), g_pszSpace);
        if (*pNewAct).szApp[0] == 0
            || GetFileAttributesW((*pNewAct).szApp.as_ptr()) == INVALID_FILE_ATTRIBUTES
        {
            let hwndCtrl = GetDlgItem(hwndDlg, IDC_ACTION_APP as INT);
            SendMessageW(
                hwndCtrl,
                EM_SETSEL,
                0,
                -1,
            );
            SetFocus(hwndCtrl);
            let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_INVALID_PROGRAM));
            let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
            MessageBoxW(
                hwndDlg,
                strText.as_ptr(),
                strTitle.as_ptr(),
                MB_ICONERROR,
            );
            return;
        }

        EndDialog(hwndDlg, IDOK as INT_PTR);
    }
}

unsafe extern "system" fn NewActionDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    static mut s_pNewAct: PACTION_DIALOG = core::ptr::null_mut();

    match uMsg {
        WM_INITDIALOG => {
            s_pNewAct = lParam as PACTION_DIALOG;
            (*s_pNewAct).bUseDDE = FALSE;
            EnableWindow(
                GetDlgItem(hwndDlg, IDC_ACTION_USE_DDE as INT),
                FALSE,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD!((wParam) as DWORD) {
                IDOK => {
                    NewActionDlg_OnOK(hwndDlg, s_pNewAct);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                }
                IDC_ACTION_BROWSE => {
                    ActionDlg_OnBrowse(hwndDlg, s_pNewAct, FALSE);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return 0;
}
unsafe extern "C" fn EditActionDlg_OnOK(hwndDlg: HWND, pEditAct: PACTION_DIALOG) {
    GetDlgItemTextW(
        hwndDlg,
        IDC_ACTION_ACTION as ::core::ffi::c_int,
        (*pEditAct).szAction.as_mut_ptr(),
        ((*pEditAct).szAction).len() as ::core::ffi::c_int,
    );
    StrTrimW((*pEditAct).szAction.as_mut_ptr(), g_pszSpace);
    if (*pEditAct).szAction[0] == 0 {
        let hwndCtrl = GetDlgItem(hwndDlg, IDC_ACTION_ACTION as INT);
        SendMessageW(
            hwndCtrl,
            EM_SETSEL,
            0,
            -1,
        );
        SetFocus(hwndCtrl);
        let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_SPECIFY_ACTION));
        let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
        MessageBoxW(
            hwndDlg,
            strText.as_ptr(),
            strTitle.as_ptr(),
            MB_ICONERROR,
        );
    }

    GetDlgItemTextW(
        hwndDlg,
        IDC_ACTION_APP as ::core::ffi::c_int,
        (*pEditAct).szApp.as_mut_ptr(),
        ((*pEditAct).szApp).len() as ::core::ffi::c_int,
    );
    StrTrimW((*pEditAct).szApp.as_mut_ptr(), g_pszSpace);
    if (*pEditAct).szApp[0] == 0 {
        let hwndCtrl = GetDlgItem(hwndDlg, IDC_ACTION_APP as INT);
        SendMessageW(
            hwndCtrl,
            EM_SETSEL,
            0,
            -1,
        );
        SetFocus(hwndCtrl);
        let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_INVALID_PROGRAM));
        let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
        MessageBoxW(
            hwndDlg,
            strText.as_ptr(),
            strTitle.as_ptr(),
            MB_ICONERROR,
        );
    }

    EndDialog(hwndDlg, IDOK as INT_PTR);
}
unsafe extern "system" fn EditActionDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut s_pEditAct: PACTION_DIALOG = core::ptr::null_mut();

    match uMsg {
        WM_INITDIALOG => {
            s_pEditAct = lParam as PACTION_DIALOG;
            (*s_pEditAct).bUseDDE = FALSE;
            SetDlgItemTextW(hwndDlg, IDC_ACTION_ACTION as INT, (*s_pEditAct).szAction.as_ptr());
            SetDlgItemTextW(hwndDlg, IDC_ACTION_APP as INT, (*s_pEditAct).szApp.as_ptr());
            EnableWindow(GetDlgItem(hwndDlg, IDC_ACTION_USE_DDE as INT), FALSE);
            EnableWindow(GetDlgItem(hwndDlg, IDC_ACTION_ACTION as INT), FALSE);
            {
                let mut str = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_EDITING_ACTION));
                str += GetTypeName((*s_pEditAct).pEntry, (*s_pEditAct).pG);
                SetWindowTextW(hwndDlg, str.as_ptr());
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD!((wParam) as DWORD) {
                IDOK => {
                    EditActionDlg_OnOK(hwndDlg, s_pEditAct);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                }
                IDC_ACTION_BROWSE => {
                    ActionDlg_OnBrowse(hwndDlg, s_pEditAct, TRUE);
                }
                _ => {}
            }
        }
        _ => {}
    }
    return 0;
}
unsafe extern "C" fn EditTypeDlg_Restrict(hwndDlg: HWND, pEditType: PEDITTYPE_DIALOG) {
    let pEntry = (*pEditType).pEntry;
    static map: [u16; 12] = [
        FTA_NoEditIcon as u16, IDC_EDITTYPE_CHANGE_ICON as u16,
        FTA_NoEditDesc as u16, IDC_EDITTYPE_TEXT as u16,
        FTA_NoNewVerb as u16, IDC_EDITTYPE_NEW as u16,
        FTA_NoEditVerb as u16, IDC_EDITTYPE_EDIT_BUTTON as u16,
        FTA_NoRemoveVerb as u16, IDC_EDITTYPE_REMOVE as u16,
        FTA_NoEditDflt as u16, IDC_EDITTYPE_SET_DEFAULT as u16
    ];
    let mut i = 0 as usize;
    while i < map.len() as usize {
        if ((*pEntry).EditFlags & map[i + 0 as usize] as u32) != 0 {
            EnableWindow(
                GetDlgItem(
                    hwndDlg,
                    map[i + 1 as usize] as i32
                ),
                FALSE as BOOL
            );
        }
        i += 2 as usize;
    }
}
unsafe extern "C" fn EditTypeDlg_WriteClass(
    hwndDlg: HWND,
    pEditType: PEDITTYPE_DIALOG,
    TypeName: LPCWSTR,
    Etf: EDITTYPEFLAGS,
) -> BOOL {
    let pEntry = (*pEditType).pEntry;
    let mut ClassKey: LPCWSTR = core::ptr::null_mut();
    let hr: HRESULT = GetClassKey(pEntry, &mut ClassKey);
    let OnlyExt = hr != S_OK;

    let mut hClassKey: HKEY = core::ptr::null_mut();
    if hr < 0
        || RegCreateKeyExW(
            HKEY_CLASSES_ROOT,
            ClassKey,
            0,
            core::ptr::null_mut(),
            0,
            KEY_QUERY_VALUE | KEY_WRITE,
            core::ptr::null_mut(),
            &mut hClassKey,
            core::ptr::null_mut(),
        ) != ERROR_SUCCESS as INT
    {
        return FALSE;
    }

    let mut dw: DWORD = 0;
    if GetRegDWORD(
        hClassKey,
        CPP_COM_WSTR_A2E2913C930F.as_ptr(),
        &mut dw,
        0,
        FALSE,
    ) == ERROR_SUCCESS as DWORD
    {
        (*pEntry).EditFlags =
            (dw & !FTA_MODIFYMASK) | ((*pEntry).EditFlags & FTA_MODIFYMASK);
    }

    if !OnlyExt {
        let pEditFlags: *mut DWORD = if (*pEntry).EditFlags != 0 {
            core::ptr::addr_of_mut!((*pEntry).EditFlags)
        } else {
            core::ptr::null_mut()
        };
        RegSetOrDelete(
            hClassKey,
            CPP_COM_WSTR_A2E2913C930F.as_ptr(),
            REG_DWORD,
            pEditFlags as LPCVOID,
            4,
        );
        if crate::dialogs_filetypes::_FILE_TYPE_ENTRY_IsExtension_body(pEntry.cast()) {
            let pAlwaysExt: LPCWSTR = if Etf & ETF_ALWAYSEXT != 0 {
                CPP_COM_WSTR_96A296D224F2.as_ptr()
            } else {
                core::ptr::null_mut()
            };
            RegSetOrDelete(
                hClassKey,
                CPP_COM_WSTR_2F0541D29D81.as_ptr(),
                REG_SZ,
                pAlwaysExt as LPCVOID,
                0,
            );
        }
        if RegKeyExists(hClassKey, CPP_COM_WSTR_073595696440.as_ptr()) {
            let ec: LRESULT = GetRegDWORD(
                hClassKey,
                CPP_COM_WSTR_325A56B0AAB7.as_ptr(),
                &mut dw,
                0,
                TRUE,
            ) as LRESULT;
            if ec == ERROR_SUCCESS as LRESULT || ec == ERROR_FILE_NOT_FOUND as LRESULT {
                dw = (dw & !8)
                    | if Etf & ETF_BROWSESAME != 0 {
                        0
                    } else {
                        8
                    };
                RegSetOrDelete(
                    hClassKey,
                    CPP_COM_WSTR_325A56B0AAB7.as_ptr(),
                    REG_DWORD,
                    if dw != 0 { core::ptr::addr_of_mut!(dw) as LPCVOID } else { core::ptr::null() },
                    core::mem::size_of_val(&dw) as DWORD,
                );
            }
        }
    }

    if ((*pEntry).EditFlags & FTA_NoEditDesc) == 0 {
        if !OnlyExt {
            RegDeleteValueW(hClassKey, core::ptr::null_mut());
        }
        if OnlyExt {
            RegDeleteValueW(hClassKey, CPP_COM_WSTR_E4DEECAD1CA5.as_ptr());
        }
        if *TypeName.offset(0) != 0 {
            RegSetString(
                hClassKey,
                if OnlyExt {
                    CPP_COM_WSTR_E4DEECAD1CA5.as_ptr()
                } else {
                    core::ptr::null_mut()
                },
                TypeName,
                REG_SZ,
            );
        }
        crate::dialogs_filetypes::_FILE_TYPE_ENTRY_InvalidateTypeName_body(pEntry.cast());
    }

    if (*pEntry).IconPath[0] != 0
        && ((*pEntry).EditFlags & FTA_NoEditIcon) == 0
        && (*pEditType).ChangedIcon
    {
        let mut hDefaultIconKey: HKEY = core::ptr::null_mut();
        if RegCreateKeyExW(
            hClassKey,
            CPP_COM_WSTR_3BE8B25B7ED1.as_ptr(),
            0,
            core::ptr::null_mut(),
            0,
            KEY_WRITE,
            core::ptr::null_mut(),
            &mut hDefaultIconKey,
            core::ptr::null_mut(),
        ) == ERROR_SUCCESS as INT
        {
            let mut icon_type: DWORD = REG_SZ;
            let mut buf = [0u16; ICONLOCATION_CCHMAX];
            let mut fmt: LPCWSTR = CPP_COM_WSTR_8EFF46569AE7.as_ptr();
            if lstrcmpW(
                (*pEntry).IconPath.as_ptr(),
                CPP_COM_WSTR_7C8F589C7B27.as_ptr(),
            ) == 0
            {
                fmt = CPP_COM_WSTR_FB47CCC40FE6.as_ptr();
            } else if !StrChrW((*pEntry).IconPath.as_ptr(), 0x0025u16).is_null() {
                icon_type = REG_EXPAND_SZ;
            }
            StringCbPrintfW!(
                buf.as_mut_ptr(),
                core::mem::size_of_val(&buf),
                fmt,
                (*pEntry).IconPath.as_ptr(),
                (*pEntry).nIconIndex,
            );
            RegSetString(
                hDefaultIconKey,
                core::ptr::null_mut(),
                buf.as_ptr(),
                icon_type,
            );
            RegCloseKey(hDefaultIconKey);
        }
    }

    let mut hShellKey: HKEY = core::ptr::null_mut();
    if RegCreateKeyExW(
        hClassKey,
        CPP_COM_WSTR_EE89605B31B0.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hShellKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS as INT
    {
        RegCloseKey(hClassKey);
        return FALSE;
    }

    if ((*pEntry).EditFlags & FTA_NoEditDflt) == 0 {
        if (*pEditType).szDefaultVerb[0] != 0 {
            RegSetString(
                hShellKey,
                core::ptr::null_mut(),
                (*pEditType).szDefaultVerb.as_ptr(),
                REG_SZ,
            );
        } else {
            RegDeleteValueW(hShellKey, core::ptr::null_mut());
        }
    }

    let mut szVerbName = [0u16; VERBKEY_CCHMAX as usize];
    let mut dwIndex: DWORD = 0;
    while RegEnumKeyW(
        hShellKey,
        dwIndex,
        szVerbName.as_mut_ptr(),
        szVerbName.len() as DWORD,
    ) == ERROR_SUCCESS as INT
    {
        if (*pEditType).CommandLineMap.FindKey(&CStringW::from_ptr(szVerbName.as_ptr())) == -1 {
            if SHDeleteKeyW(hShellKey, szVerbName.as_ptr()) == ERROR_SUCCESS as DWORD {
                dwIndex = dwIndex.wrapping_sub(1);
            }
        }
        dwIndex = dwIndex.wrapping_add(1);
    }

    let nCount: INT = (*pEditType).CommandLineMap.GetSize() as INT;
    let mut i: INT = 0;
    while i < nCount {
        let key = (*pEditType).CommandLineMap.GetKeyAt(i);
        let cmd = (*pEditType).CommandLineMap.GetValueAt(i);
        if !(*pEditType).ModifiedVerbs.Find(&key) {
            i += 1;
            continue;
        }

        let mut hVerbKey: HKEY = core::ptr::null_mut();
        if RegCreateKeyExW(
            hShellKey,
            key.as_ptr(),
            0,
            core::ptr::null_mut(),
            0,
            KEY_WRITE,
            core::ptr::null_mut(),
            &mut hVerbKey,
            core::ptr::null_mut(),
        ) == ERROR_SUCCESS as INT
        {
            let mut hCommandKey: HKEY = core::ptr::null_mut();
            if RegCreateKeyExW(
                hVerbKey,
                CPP_COM_WSTR_0C71B3CB5580.as_ptr(),
                0,
                core::ptr::null_mut(),
                0,
                KEY_WRITE,
                core::ptr::null_mut(),
                &mut hCommandKey,
                core::ptr::null_mut(),
            ) == ERROR_SUCCESS as INT
            {
                let dwSize: DWORD =
                    ((cmd.GetLength() + 1) * core::mem::size_of::<u16>() as i32) as DWORD;
                let mut dwType: DWORD = REG_SZ;
                let exp = cmd.Find('%' as u16, 0);
                if exp >= 0
                    && cmd.Find('%' as u16, exp + 1) >= 0
                {
                    dwType = REG_EXPAND_SZ;
                }
                RegSetValueExW(
                    hCommandKey,
                    core::ptr::null_mut(),
                    0,
                    dwType,
                    (cmd).as_ptr() as LPBYTE,
                    dwSize,
                );
                RegCloseKey(hCommandKey);
            }
            RegCloseKey(hVerbKey);
        }
        i += 1;
    }

    RegCloseKey(hShellKey);
    RegCloseKey(hClassKey);
    return TRUE;
}
unsafe extern "C" fn EditTypeDlg_ReadClass(hwndDlg: HWND, pEditType: *mut _EDITTYPE_DIALOG, Etf: *mut EDITTYPEFLAGS) -> BOOL {
    let pEntry: *mut _FILE_TYPE_ENTRY = (*pEditType).pEntry;
    let mut ClassKey: LPCWSTR = core::ptr::null_mut();
    let hr: HRESULT = GetClassKey(pEntry, &mut ClassKey);
    let mut hClassKey: HKEY = core::ptr::null_mut();
    if (hr < 0) || (RegOpenKeyExW(HKEY_CLASSES_ROOT, ClassKey, 0, KEY_READ, &mut hClassKey) != 0)
    {
        return FALSE;
    }

    let mut etfbits: UINT = if RegValueExists(hClassKey, CPP_COM_WSTR_2F0541D29D81.as_ptr()) { ETF_ALWAYSEXT as UINT } else { 0 };

    let docobj: BOOL = RegKeyExists(hClassKey, CPP_COM_WSTR_073595696440.as_ptr()) as BOOL;
    EnableWindow(GetDlgItem(hwndDlg, IDC_EDITTYPE_SAME_WINDOW as INT), docobj);
    etfbits |= (if docobj != 0 && (GetRegDWORD_2(hClassKey, CPP_COM_WSTR_325A56B0AAB7.as_ptr(), 0) & 8) != 0 { 0 } else { ETF_BROWSESAME }) as UINT;

    *Etf = ((etfbits) as EDITTYPEFLAGS);

    let mut hShellKey: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(hClassKey, CPP_COM_WSTR_EE89605B31B0.as_ptr(), 0, KEY_READ, &mut hShellKey) != ERROR_SUCCESS as INT
    {
        RegCloseKey(hClassKey);
        return FALSE;
    }

    let mut DefaultVerb: [u16; VERBKEY_CCHMAX as usize] = [0; VERBKEY_CCHMAX as usize];
    let mut dwSize: DWORD = core::mem::size_of_val(&DefaultVerb) as DWORD;
    if RegQueryValueExW(hShellKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), (DefaultVerb.as_mut_ptr()) as LPBYTE, &mut dwSize) == ERROR_SUCCESS as INT
    {
        StringCbCopyW((*pEditType).szDefaultVerb.as_mut_ptr(), core::mem::size_of_val(&(*pEditType).szDefaultVerb), DefaultVerb.as_ptr());
    }
    else
    {
        StringCbCopyW((*pEditType).szDefaultVerb.as_mut_ptr(), core::mem::size_of_val(&(*pEditType).szDefaultVerb), CPP_COM_WSTR_82DB442E7A60.as_ptr());
    }

    let mut szVerbName: [u16; VERBKEY_CCHMAX as usize] = [0; VERBKEY_CCHMAX as usize];
    let mut dwIndex: DWORD = 0;
    while RegEnumKeyW(hShellKey, dwIndex, szVerbName.as_mut_ptr(), szVerbName.len() as DWORD) == ERROR_SUCCESS as INT
    {
        let mut hVerbKey: HKEY = core::ptr::null_mut();
        let mut nResult: LONG = RegOpenKeyExW(hShellKey, szVerbName.as_ptr(), 0, KEY_READ, &mut hVerbKey);
        if nResult == ERROR_SUCCESS as LONG as INT
        {
            let mut hCommandKey: HKEY = core::ptr::null_mut();
            nResult = RegOpenKeyExW(hVerbKey, CPP_COM_WSTR_0C71B3CB5580.as_ptr(), 0, KEY_READ, &mut hCommandKey);
            if nResult == ERROR_SUCCESS as LONG as INT
            {
                let mut szValue: [u16; (MAX_PATH * 2) as usize] = [0; (MAX_PATH * 2) as usize];
                dwSize = core::mem::size_of_val(&szValue) as DWORD;
                nResult = RegQueryValueExW(hCommandKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), (szValue.as_mut_ptr()) as LPBYTE, &mut dwSize);
                if nResult == ERROR_SUCCESS as LONG as INT
                {
                    (*pEditType).CommandLineMap.SetAtOwned(CStringW::from_ptr(szVerbName.as_ptr()), CStringW::from_ptr(szValue.as_ptr()));
                }

                RegCloseKey(hCommandKey);
            }

            RegCloseKey(hVerbKey);
        }
        SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_LISTBOX as INT, LB_ADDSTRING, 0, (szVerbName.as_ptr()) as LPARAM);
        dwIndex += 1;
    }

    let mut TypeName: [u16; TYPENAME_CCHMAX as usize] = [0; TYPENAME_CCHMAX as usize];
    dwSize = core::mem::size_of_val(&TypeName) as DWORD;
    if RegQueryValueExW(hClassKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), (TypeName.as_mut_ptr()) as LPBYTE, &mut dwSize) == 0
    {
        TypeName[(TypeName.len() as usize) - (1 as usize)] = UNICODE_NULL as u16;
        SetDlgItemTextW(hwndDlg, IDC_EDITTYPE_TEXT as INT, TypeName.as_ptr());
    }

    RegCloseKey(hShellKey);
    RegCloseKey(hClassKey);
    return TRUE;
}
unsafe extern "C" fn EditTypeDlg_OnOK(hwndDlg: HWND, pEditType: PEDITTYPE_DIALOG) {
    let pEntry: PFILE_TYPE_ENTRY = (*pEditType).pEntry;

    let mut TypeName = [0u16; TYPENAME_CCHMAX as usize];
    let type_name_len = TypeName.len() as i32;
    GetDlgItemTextW(hwndDlg, IDC_EDITTYPE_TEXT as INT, TypeName.as_mut_ptr(), type_name_len);
    StrTrimW(TypeName.as_mut_ptr(), g_pszSpace);

    let mut etf: u32 = 0;
    (*pEntry).EditFlags &= !FTA_MODIFYMASK;
    if SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_CONFIRM_OPEN as INT, BM_GETCHECK, 0, 0) == 0 {
        (*pEntry).EditFlags |= FTA_OpenIsSafe;
    }
    if SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_SHOW_EXT as INT, BM_GETCHECK, 0, 0) != 0 {
        etf |= ETF_ALWAYSEXT as UINT;
    }
    if SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_SAME_WINDOW as INT, BM_GETCHECK, 0, 0) != 0 {
        etf |= ETF_BROWSESAME as UINT;
    }

    EditTypeDlg_UpdateEntryIcon(hwndDlg, pEditType);

    EditTypeDlg_WriteClass(hwndDlg, pEditType, TypeName.as_ptr(), etf as EDITTYPEFLAGS);

    (*pEntry).ProgramPath[0] = 0;
    (*pEntry).ProgramPath[1] = 0;
    (*pEntry).AppName[0] = 0;
    (*pEntry).AppName[1] = 0;

    SHChangeNotify(SHCNE_ASSOCCHANGED as INT, SHCNF_FLUSHNOWAIT, core::ptr::null_mut(), core::ptr::null_mut());

    EndDialog(hwndDlg, IDOK as INT_PTR);
}
unsafe extern "C" fn EditTypeDlg_OnRemove(hwndDlg: HWND, pEditType: PEDITTYPE_DIALOG) -> BOOL {
    let iItem = SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_LISTBOX as INT, LB_GETCURSEL, 0, 0) as i32;
    if iItem == LB_ERR {
        return FALSE;
    }

    let strText = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_REMOVE_ACTION));
    let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
    if MessageBoxW(hwndDlg, strText.as_ptr(), strTitle.as_ptr(), MB_ICONINFORMATION | MB_YESNO) == IDNO as INT {
        return FALSE;
    }

    let mut szText: [u16; VERBKEY_CCHMAX as usize] = [0; VERBKEY_CCHMAX as usize];
    szText[0] = 0;
    SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_LISTBOX as INT, LB_GETTEXT, iItem as WPARAM, szText.as_mut_ptr() as LPARAM);
    StrTrimW(szText.as_mut_ptr(), g_pszSpace);

    ((*pEditType).CommandLineMap).Remove(&CStringW::from_ptr(szText.as_ptr()));
    SendDlgItemMessageW(hwndDlg, IDC_EDITTYPE_LISTBOX as INT, LB_DELETESTRING, iItem as WPARAM, 0);
    TRUE
}
unsafe extern "C" fn EditTypeDlg_OnCommand(
    hwndDlg: HWND,
    id: UINT,
    code: UINT,
    pEditType: PEDITTYPE_DIALOG,
) {
    let mut iItem: INT = 0;
    let mut iIndex: INT = 0;
    let mut action: ACTION_DIALOG = core::mem::zeroed();

    if id == IDOK {
        EditTypeDlg_OnOK(hwndDlg, pEditType);
    } else if id == IDCANCEL {
        EndDialog(hwndDlg, IDCANCEL as INT_PTR);
    } else if id == IDC_EDITTYPE_CHANGE_ICON {
        EditTypeDlg_OnChangeIcon(hwndDlg, pEditType);
    } else if id == IDC_EDITTYPE_NEW {
        action.bUseDDE = FALSE;
        action.hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
        action.pEntry = (*pEditType).pEntry;
        if IDOK as INT_PTR == DialogBoxParamW(
            shell32_hInstance,
            MAKEINTRESOURCEW(IDD_ACTION),
            hwndDlg,
            Some(NewActionDlgProc),
            core::ptr::addr_of!(action) as LPARAM,
        ) {
            if SendMessageW(
                action.hwndLB,
                LB_FINDSTRING,
                -1i32 as WPARAM,
                action.szAction.as_ptr() as LPARAM,
            ) != LB_ERR as LRESULT
            {
                let hwndCtrl = GetDlgItem(hwndDlg, IDC_ACTION_ACTION as INT);
                SendMessageW(
                    hwndCtrl,
                    EM_SETSEL,
                    0,
                    -1,
                );
                SetFocus(hwndCtrl);

                let mut strText = CStringW::new();
                let strTitle = MAKEINTRESOURCEW(IDS_FILE_TYPES);
                strText.FormatResStr(IDS_ACTION_EXISTS, action.szAction.as_ptr());
                MessageBoxW(hwndDlg, strText.as_ptr(), strTitle, MB_ICONERROR);
            } else {
                let mut strCommandLine = CStringW::from_ptr(action.szApp.as_ptr());
                QuoteAppPathForCommand(&mut strCommandLine);
                strCommandLine += CPP_COM_WSTR_2ED82ED19E49.as_ptr();
                (*pEditType).CommandLineMap.SetAtOwned(CStringW::from_ptr(action.szAction.as_ptr()), strCommandLine);
                (*pEditType).ModifiedVerbs.AddHead(CStringW::from_ptr(action.szAction.as_ptr()));
                SendMessageW(
                    action.hwndLB,
                    LB_ADDSTRING,
                    0,
                    (action.szAction.as_ptr()) as LPARAM,
                );
                if SendMessageW(
                    action.hwndLB,
                    LB_GETCOUNT,
                    0,
                    0,
                ) == 1
                {
                    StringCbCopyW(
                        (*pEditType).szDefaultVerb.as_mut_ptr(),
                        core::mem::size_of_val(&(*pEditType).szDefaultVerb),
                        action.szAction.as_ptr(),
                    );
                    InvalidateRect(
                        action.hwndLB,
                        core::ptr::null_mut(),
                        TRUE,
                    );
                }
            }
        }
    } else if id == IDC_EDITTYPE_LISTBOX {
        if code == LBN_SELCHANGE {
            action.hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
            let iItem = SendMessageW(
                action.hwndLB,
                LB_GETCURSEL,
                0,
                0,
            ) as INT;
            SendMessageW(
                action.hwndLB,
                LB_GETTEXT,
                iItem as WPARAM,
                (action.szAction.as_ptr()) as LPARAM,
            );
            if lstrcmpiW(
                action.szAction.as_ptr(),
                (*pEditType).szDefaultVerb.as_ptr(),
            ) == 0
            {
                EnableWindow(
                    GetDlgItem(hwndDlg, IDC_EDITTYPE_SET_DEFAULT as INT),
                    FALSE,
                );
            } else {
                EnableWindow(
                    GetDlgItem(hwndDlg, IDC_EDITTYPE_SET_DEFAULT as INT),
                    TRUE,
                );
            }
            EditTypeDlg_Restrict(hwndDlg, pEditType);
        } else if code != LBN_DBLCLK {
        } else {
            'fallthrough_0040: {
                action.bUseDDE = FALSE;
                action.hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
                action.pG = (*pEditType).pG;
                action.pEntry = (*pEditType).pEntry;
                iItem = SendMessageW(
                    action.hwndLB,
                    LB_GETCURSEL,
                    0,
                    0,
                ) as INT;
                if iItem == LB_ERR {
                    break 'fallthrough_0040;
                }
                SendMessageW(
                    action.hwndLB,
                    LB_GETTEXT,
                    iItem as WPARAM,
                    (action.szAction.as_ptr()) as LPARAM,
                );
                {
                    iIndex = (*pEditType).CommandLineMap.FindKey(&CStringW::from_ptr(action.szAction.as_ptr()));
                    let str = (*pEditType).CommandLineMap.GetValueAt(iIndex);
                    StringCbCopyW(
                        action.szApp.as_mut_ptr(),
                        core::mem::size_of_val(&action.szApp),
                        (str).as_ptr(),
                    );
                }
                if IDOK as INT_PTR == DialogBoxParamW(
                    shell32_hInstance,
                    MAKEINTRESOURCEW(IDD_ACTION),
                    hwndDlg,
                    Some(EditActionDlgProc),
                    core::ptr::addr_of!(action) as LPARAM,
                ) {
                    SendMessageW(
                        action.hwndLB,
                        LB_DELETESTRING,
                        iItem as WPARAM,
                        0,
                    );
                    SendMessageW(
                        action.hwndLB,
                        LB_INSERTSTRING,
                        iItem as WPARAM,
                        (action.szAction.as_ptr()) as LPARAM,
                    );
                    (*pEditType).CommandLineMap.SetAtOwned(CStringW::from_ptr(action.szAction.as_ptr()), CStringW::from_ptr(action.szApp.as_ptr()));
                    (*pEditType).ModifiedVerbs.AddHead(CStringW::from_ptr(action.szAction.as_ptr()));
                }
            }
        }
    } else if id == IDC_EDITTYPE_EDIT_BUTTON {
        'case_0040: {
            action.bUseDDE = FALSE;
            action.hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
            action.pG = (*pEditType).pG;
            action.pEntry = (*pEditType).pEntry;
            iItem = SendMessageW(
                action.hwndLB,
                LB_GETCURSEL,
                0,
                0,
            ) as INT;
            if iItem == LB_ERR {
                break 'case_0040;
            }
            SendMessageW(
                action.hwndLB,
                LB_GETTEXT,
                iItem as WPARAM,
                (action.szAction.as_ptr()) as LPARAM,
            );
            {
                iIndex = (*pEditType).CommandLineMap.FindKey(&CStringW::from_ptr(action.szAction.as_ptr()));
                let str = (*pEditType).CommandLineMap.GetValueAt(iIndex);
                StringCbCopyW(
                    action.szApp.as_mut_ptr(),
                    core::mem::size_of_val(&action.szApp),
                    (str).as_ptr(),
                );
            }
            if IDOK as INT_PTR == DialogBoxParamW(
                shell32_hInstance,
                MAKEINTRESOURCEW(IDD_ACTION),
                hwndDlg,
                Some(EditActionDlgProc),
                core::ptr::addr_of!(action) as LPARAM,
            ) {
                SendMessageW(
                    action.hwndLB,
                    LB_DELETESTRING,
                    iItem as WPARAM,
                    0,
                );
                SendMessageW(
                    action.hwndLB,
                    LB_INSERTSTRING,
                    iItem as WPARAM,
                    (action.szAction.as_ptr()) as LPARAM,
                );
                (*pEditType).CommandLineMap.SetAtOwned(CStringW::from_ptr(action.szAction.as_ptr()), CStringW::from_ptr(action.szApp.as_ptr()));
                (*pEditType).ModifiedVerbs.AddHead(CStringW::from_ptr(action.szAction.as_ptr()));
            }
        }
    } else if id == IDC_EDITTYPE_REMOVE {
        EditTypeDlg_OnRemove(hwndDlg, pEditType);
    } else if id == IDC_EDITTYPE_SET_DEFAULT {
        'case_0054: {
            action.hwndLB = GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT);
            iItem = SendMessageW(
                action.hwndLB,
                LB_GETCURSEL,
                0,
                0,
            ) as INT;
            if iItem == LB_ERR {
                break 'case_0054;
            }
            SendMessageW(
                action.hwndLB,
                LB_GETTEXT,
                iItem as WPARAM,
                (action.szAction.as_ptr()) as LPARAM,
            );
            StringCbCopyW(
                (*pEditType).szDefaultVerb.as_mut_ptr(),
                core::mem::size_of_val(&(*pEditType).szDefaultVerb),
                action.szAction.as_ptr(),
            );
            EnableWindow(
                GetDlgItem(hwndDlg, IDC_EDITTYPE_SET_DEFAULT as INT),
                FALSE,
            );
            InvalidateRect(
                action.hwndLB,
                core::ptr::null_mut(),
                TRUE,
            );
        }
    }
}
unsafe extern "C" fn EditTypeDlg_OnInitDialog(
    hwndDlg: HWND,
    pEditType: PEDITTYPE_DIALOG,
) -> BOOL {
    let pEntry: PFILE_TYPE_ENTRY = (*pEditType).pEntry;
    let mut Etf: EDITTYPEFLAGS = 0;

    ExpandEnvironmentStringsW(
        (*pEntry).IconPath.as_ptr(),
        (*pEditType).szIconPath.as_mut_ptr(),
        ((*pEditType).szIconPath).len() as DWORD,
    );
    (*pEditType).nIconIndex = (*pEntry).nIconIndex;
    StringCbCopyW(
        (*pEditType).szDefaultVerb.as_mut_ptr(),
        core::mem::size_of_val(&(*pEditType).szDefaultVerb),
        CPP_COM_WSTR_82DB442E7A60.as_ptr(),
    );
    (*pEditType).ChangedIcon = false;

    let hIco: HICON =
        DoExtractIcon_2(
            (*pEditType).szIconPath.as_ptr(),
            (*pEditType).nIconIndex,
            FALSE,
        );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_ICON as INT,
        STM_SETICON,
        hIco as WPARAM,
        0,
    );

    EditTypeDlg_ReadClass(
        hwndDlg,
        pEditType,
        core::ptr::addr_of_mut!(Etf),
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_CONFIRM_OPEN as INT,
        BM_SETCHECK,
        (((*pEntry).EditFlags & FTA_OpenIsSafe) == 0) as WPARAM,
        0,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_SHOW_EXT as INT,
        BM_SETCHECK,
        ((Etf & ETF_ALWAYSEXT) != 0) as WPARAM,
        0,
    );

    EnableWindow(
        GetDlgItem(hwndDlg, IDC_EDITTYPE_SHOW_EXT as INT),
        crate::dialogs_filetypes::_FILE_TYPE_ENTRY_IsExtension_body(pEntry.cast()) as BOOL,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_SAME_WINDOW as INT,
        BM_SETCHECK,
        ((Etf & ETF_BROWSESAME) != 0) as WPARAM,
        0,
    );

    InvalidateRect(
        GetDlgItem(hwndDlg, IDC_EDITTYPE_LISTBOX as INT),
        core::ptr::null_mut(),
        TRUE,
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_LISTBOX as INT,
        LB_SETCURSEL,
        0,
        0,
    );

    if SendDlgItemMessageW(
        hwndDlg,
        IDC_EDITTYPE_LISTBOX as INT,
        LB_GETCOUNT,
        0,
        0,
    ) == 0
    {
        EnableWindow(
            GetDlgItem(hwndDlg, IDC_EDITTYPE_EDIT_BUTTON as INT),
            FALSE,
        );
        EnableWindow(
            GetDlgItem(hwndDlg, IDC_EDITTYPE_REMOVE as INT),
            FALSE,
        );
        EnableWindow(
            GetDlgItem(hwndDlg, IDC_EDITTYPE_SET_DEFAULT as INT),
            FALSE,
        );
    }

    EditTypeDlg_Restrict(hwndDlg, pEditType);

    return TRUE;
}

unsafe extern "system" fn EditTypeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    static mut s_pEditType: PEDITTYPE_DIALOG = core::ptr::null_mut();

    unsafe {
        match uMsg {
            WM_INITDIALOG => {
                s_pEditType = lParam as PEDITTYPE_DIALOG;
                return EditTypeDlg_OnInitDialog(hwndDlg, s_pEditType) as INT_PTR;
            }
            WM_DESTROY => {
                let hOld: HICON = SendDlgItemMessageW(
                    hwndDlg,
                    IDC_EDITTYPE_ICON as INT,
                    STM_GETICON,
                    0,
                    0,
                ) as HICON;
                if !hOld.is_null() {
                    DestroyIcon(hOld);
                }
            }
            WM_DRAWITEM => {
                let pDraw: LPDRAWITEMSTRUCT = (lParam) as LPDRAWITEMSTRUCT;
                return EditTypeDlg_OnDrawItem(hwndDlg, pDraw, s_pEditType) as INT_PTR;
            }
            WM_MEASUREITEM => {
                let pMeasure: LPMEASUREITEMSTRUCT = (lParam) as LPMEASUREITEMSTRUCT;
                return EditTypeDlg_OnMeasureItem(hwndDlg, pMeasure, s_pEditType) as INT_PTR;
            }
            WM_COMMAND => {
                EditTypeDlg_OnCommand(
                    hwndDlg,
                    LOWORD!((wParam) as DWORD),
                    HIWORD(wParam as DWORD) as UINT,
                    s_pEditType,
                );
            }
            _ => {}
        }

        0
    }
}

unsafe extern "system" fn FileTypesDlg_CompareItems(
    lParam1: LPARAM,
    lParam2: LPARAM,
    lParamSort: LPARAM,
) -> i32 {
    unsafe {
        let pG: PFILE_TYPE_GLOBALS = lParamSort as PFILE_TYPE_GLOBALS;
        let entry1: PFILE_TYPE_ENTRY = lParam1 as PFILE_TYPE_ENTRY;
        let entry2: PFILE_TYPE_ENTRY = lParam2 as PFILE_TYPE_ENTRY;
        let mut x: i32 = 0;

        if (*pG).SortCol == 1 {
            x = StrCmpIW(
                GetTypeName(entry1, pG),
                GetTypeName(entry2, pG),
            );
        }

        if x == 0 {
            x = ((*entry1).FileExtension[0] == ('.' as u16)) as i32
                - ((*entry2).FileExtension[0] == ('.' as u16)) as i32;
            if x == 0 {
                x = StrCmpIW(
                    (*entry1).FileExtension.as_ptr(),
                    (*entry2).FileExtension.as_ptr(),
                );
            }
        }

        x * ((*pG).SortReverse as i32)
    }
}
unsafe extern "C" fn FileTypesDlg_Sort(pG: PFILE_TYPE_GLOBALS, hListView: HWND, Column: i32) {
    (*pG).SortReverse = if ((*pG).SortCol) as INT == Column {
        ((((*pG).SortReverse as i32) * -(1 as i32)) as i8)
    } else {
        1 as i8
    };
    (*pG).SortCol = if Column < 0 {
        0 as i8
    } else {
        Column as i8
    };
    ListView_SortItems(hListView, Some(FileTypesDlg_CompareItems), pG as LPARAM);
}
unsafe extern "C" fn FileTypesDlg_InitListView(hwndDlg: HWND, hListView: HWND) {
    unsafe {
        let _ = hwndDlg;
        let mut clientRect: RECT = core::mem::zeroed();
        let mut columnName: *const u16;
        let mut szBuf: [u16; 50] = [0u16; 50];

        let mut col: LVCOLUMNW = core::mem::zeroed();
        col.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;
        col.fmt = 0;

        GetClientRect(hListView, core::ptr::addr_of_mut!(clientRect));
        let column0Size = (clientRect.right - clientRect.left) / 4;

        columnName = CPP_COM_WSTR_8A7837CAF8C9.as_ptr();
        if LoadStringW(shell32_hInstance, IDS_COLUMN_EXTENSION, szBuf.as_mut_ptr(), szBuf.len() as i32) != 0 {
            columnName = szBuf.as_ptr();
        }
        col.pszText = columnName as *mut u16;
        col.iSubItem = 0;
        col.cx = column0Size;
        SendMessageW(hListView, LVM_INSERTCOLUMNW, 0, core::ptr::addr_of_mut!(col) as LPARAM);

        columnName = CPP_COM_WSTR_65C19E07157A.as_ptr();
        if LoadStringW(shell32_hInstance, IDS_FILE_TYPES, szBuf.as_mut_ptr(), szBuf.len() as i32) != 0 {
            columnName = szBuf.as_ptr();
        }
        col.pszText = columnName as *mut u16;
        col.iSubItem = 1;
        col.cx = clientRect.right - clientRect.left - column0Size - GetSystemMetrics(SM_CYVSCROLL as INT);
        SendMessageW(hListView, LVM_INSERTCOLUMNW, 1, core::ptr::addr_of_mut!(col) as LPARAM);

        let lvexstyle: u32 = LVS_EX_FULLROWSELECT | LVS_EX_LABELTIP;
        ListView_SetExtendedListViewStyleEx(hListView, lvexstyle, lvexstyle);
    }
}
unsafe extern "C" fn FileTypesDlg_SetGroupboxText(hwndDlg: HWND, Assoc: LPCWSTR) {
    let _ = IDS_FILE_DETAILS;
    SetDlgItemTextW(hwndDlg, IDC_FILETYPES_DETAILS_GROUPBOX as INT, Assoc);
}
unsafe extern "C" fn FileTypesDlg_Refresh(hwndDlg: HWND, hListView: HWND, pG: PFILE_TYPE_GLOBALS) {
    ListView_DeleteAllItems(hListView);
    ImageList_RemoveAll((*pG).himlSmall);
    InitializeDefaultIcons(pG);
    FileTypesDlg_SetGroupboxText(hwndDlg, CPP_COM_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hwndDlg, IDC_FILETYPES_DESCRIPTION as INT, CPP_COM_WSTR_96A296D224F2.as_ptr());
    SetDlgItemTextW(hwndDlg, IDC_FILETYPES_APPNAME as INT, CPP_COM_WSTR_96A296D224F2.as_ptr());
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FILETYPES_DELETE as INT),
        FALSE,
    );
    RedrawWindow(
        hwndDlg,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_UPDATENOW,
    );

    let statTickStart: u32 = GetTickCount();
    let mut iItem: i32 = 0;
    let mut szName = [0u16; ASSOC_CCHMAX];
    let mut dwName: u32 = szName.len() as u32;
    let mut dwIndex: u32 = 0;

    SendMessage(hListView, WM_SETREDRAW, FALSE as WPARAM, 0);

    while RegEnumKeyExW(
        HKEY_CLASSES_ROOT,
        {
            let old = dwIndex;
            dwIndex = dwIndex.wrapping_add(1);
            old
        },
        szName.as_mut_ptr(),
        &mut dwName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == ERROR_SUCCESS as INT
    {
        if !FileTypesDlg_InsertToLV(hListView, szName.as_ptr(), iItem, pG).is_null() {
            iItem += 1;
        }
        dwName = szName.len() as u32;
    }

    FileTypesDlg_Sort(pG, hListView, -1);
    SendMessage(hListView, WM_SETREDRAW, TRUE as WPARAM, 0);
    RedrawWindow(
        hListView,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        RDW_ALLCHILDREN | RDW_ERASE | RDW_FRAME | RDW_INVALIDATE,
    );

    DbgPrint(
        b"FT loaded %u (%ums)\n\0".as_ptr() as PCHAR,
        iItem,
        GetTickCount().wrapping_sub(statTickStart),
    );

    ListView_SetItemState(
        hListView,
        0,
        -1i32 as UINT,
        LVIS_FOCUSED | LVIS_SELECTED,
    );
}
unsafe extern "C" fn FileTypesDlg_Initialize(hwndDlg: HWND) -> *mut _FILE_TYPE_GLOBALS {
    let hListView: HWND = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);
    let pG: *mut _FILE_TYPE_GLOBALS = SHAlloc(core::mem::size_of::<_FILE_TYPE_GLOBALS>() as SIZE_T) as *mut _FILE_TYPE_GLOBALS;
    if pG.is_null()
    {
        return pG;
    }

    (*pG).SortReverse = 1 as i8;
    (*pG).hDefExtIconSmall = core::ptr::null_mut();
    (*pG).hOpenWithImage = core::ptr::null_mut();
    (*pG).IconSize = GetSystemMetrics(SM_CXSMICON as INT) as u32;
    (*pG).himlSmall = ImageList_Create((*pG).IconSize as ::core::ffi::c_int, (*pG).IconSize as ::core::ffi::c_int, ILC_COLOR32 | ILC_MASK, 256, 20);
    (*pG).hHeap = GetProcessHeap();

    (*pG).NoneString[0 as usize] = UNICODE_NULL as u16;
    LoadStringW(shell32_hInstance, IDS_NONE, (*pG).NoneString.as_mut_ptr(), ((*pG).NoneString).len() as ::core::ffi::c_int);

    let restricted: u32 = SHRestricted(REST_NOFILEASSOCIATE) as u32;
    (*pG).Restricted = restricted;
    if restricted == 0
    {
        let mut hKey: HKEY = core::ptr::null_mut();
        if RegCreateKeyEx(HKEY_LOCAL_MACHINE, CPP_COM_WSTR_DBE081544FA1.as_ptr(), 0, core::ptr::null_mut(), 0, KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut()) != 0
        {
            (*pG).Restricted = TRUE as u32;
        }
        else
        {
            RegCloseKey(hKey);
        }
    }

    FileTypesDlg_InitListView(hwndDlg, hListView);
    ListView_SetImageList(hListView, (*pG).himlSmall, LVSIL_SMALL as INT);
    EnableWindow(GetDlgItem(hwndDlg, IDC_FILETYPES_NEW as INT), if (*pG).Restricted == 0 { 1 } else { 0 });

    PostMessage(hListView, WM_KEYDOWN, VK_F5 as WPARAM, 0);
    return pG;
}
unsafe extern "C" fn FileTypesDlg_GetEntry(hListView: HWND, mut iItem: INT) -> PFILE_TYPE_ENTRY {
    if(iItem == -1)
    {
        iItem = ListView_GetNextItem(hListView, -1, LVNI_SELECTED);
        if(iItem == -1)
        {
            return core::ptr::null_mut();
        }
    }

    let mut lvItem: LV_ITEMW = core::mem::zeroed();
    lvItem.mask = LVIF_PARAM;
    lvItem.iItem = iItem;
    if ListView_GetItem(hListView, core::ptr::addr_of_mut!(lvItem)) != 0
    {
        return lvItem.lParam as PFILE_TYPE_ENTRY;
    }

    return core::ptr::null_mut();
}
unsafe extern "C" fn FileTypesDlg_OnDelete(hwndDlg: HWND) {
    let mut strRemoveExt = CStringW::from_resource(IDS_REMOVE_EXT as UINT);
    let strTitle = CStringW::from_ptr(MAKEINTRESOURCEW(IDS_FILE_TYPES));
    if MessageBoxW(
        hwndDlg,
        strRemoveExt.as_ptr(),
        strTitle.as_ptr(),
        MB_ICONQUESTION | MB_YESNO,
    ) == IDYES as INT
    {
        FileTypesDlg_RemoveExt(hwndDlg);

        // Select first item (Win2k3 does it)
        let hListView = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);
        ListView_SetItemState(
            hListView,
            0,
            -1i32 as UINT,
            LVIS_FOCUSED | LVIS_SELECTED,
        );
    }
}
unsafe extern "C" fn FileTypesDlg_OnItemChanging(
    hwndDlg: HWND,
    pEntry: PFILE_TYPE_ENTRY,
    pG: PFILE_TYPE_GLOBALS,
) {
    let hbmProgram: *mut HBITMAP =
        core::ptr::addr_of_mut!((*pG).hOpenWithImage);
    let is_extension = (*pEntry).FileExtension[0] == '.' as u16;
    let DispAssoc: LPCWSTR = if is_extension {
        (*pEntry).FileExtension.as_ptr().add(1)
    } else {
        (*pEntry).FileExtension.as_ptr()
    };
    let TypeName: LPCWSTR = GetTypeName(pEntry, pG) as LPCWSTR;
    let mut buf = CStringW::new();

    FileTypesDlg_SetGroupboxText(hwndDlg, DispAssoc);
    if is_extension {
        buf.FormatResStr3(IDS_FILE_DETAILSADV as UINT, DispAssoc, TypeName, TypeName);
    } else {
        buf = CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr());
    }
    SetDlgItemTextW(hwndDlg, IDC_FILETYPES_DESCRIPTION as INT, buf.GetString());

    if !(*hbmProgram).is_null() {
        DeleteObject(*hbmProgram as HGDIOBJ);
        *hbmProgram = core::ptr::null_mut();
    }

    let appname: LPCWSTR = GetAppName(pEntry);
    SetDlgItemTextW(hwndDlg, IDC_FILETYPES_APPNAME as INT, appname);

    let mut hIconSm: HICON = core::ptr::null_mut();
    let exe: LPCWSTR = GetProgramPath(pEntry);
    if *exe != 0u16 {
        ExtractIconExW(
            exe,
            0,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hIconSm),
            1,
        );
    }
    *hbmProgram = BitmapFromIcon(
        hIconSm,
        16,
        16,
    );
    DestroyIcon(hIconSm);
    SendDlgItemMessageW(
        hwndDlg,
        IDC_FILETYPES_ICON as INT,
        STM_SETIMAGE,
        IMAGE_BITMAP as WPARAM,
        (*hbmProgram) as LPARAM,
    );

    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FILETYPES_CHANGE as INT),
        (((*pG).Restricted == 0) && is_extension) as BOOL,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FILETYPES_ADVANCED as INT),
        ((((*pEntry).EditFlags & FTA_NoEdit) == 0) && ((*pG).Restricted == 0)) as BOOL,
    );
    EnableWindow(
        GetDlgItem(hwndDlg, IDC_FILETYPES_DELETE as INT),
        ((((*pEntry).EditFlags & FTA_NoRemove) == 0) && ((*pG).Restricted == 0) && is_extension) as BOOL,
    );
}
#[no_mangle]
pub unsafe extern "system" fn FolderOptionsFileTypesDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pGlobals = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PFILE_TYPE_GLOBALS;
    if pGlobals.is_null() && uMsg != WM_INITDIALOG {
        return FALSE as INT_PTR;
    }
    let mut lppl: *mut _NMLISTVIEW = core::ptr::null_mut();
    let mut pEntry: PFILE_TYPE_ENTRY = core::ptr::null_mut();
    let mut newext: NEWEXT_DIALOG = core::mem::zeroed();
    let mut edittype = core::mem::MaybeUninit::<EDITTYPE_DIALOG>::uninit();

    match uMsg {
        WM_INITDIALOG => {
            pGlobals = FileTypesDlg_Initialize(hwndDlg);
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pGlobals as LONG_PTR);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            SetWindowLongPtrW(hwndDlg, DWLP_USER, 0);
            if !pGlobals.is_null() {
                DestroyIcon((*pGlobals).hDefExtIconSmall);
                DeleteObject((*pGlobals).hOpenWithImage as HGDIOBJ);
                SHFree(pGlobals as crate::island::LPVOID);
            }
        }

        WM_COMMAND => {
            match LOWORD!((wParam) as DWORD) {
                IDC_FILETYPES_NEW => {
                    newext.hwndLV = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);
                    if IDOK as INT_PTR == DialogBoxParamW(
                        shell32_hInstance,
                        MAKEINTRESOURCEW(IDD_NEWEXTENSION),
                        hwndDlg,
                        Some(NewExtDlgProc),
                        &mut newext as *mut NEWEXT_DIALOG as LPARAM,
                    ) {
                        FileTypesDlg_AddExt(
                            hwndDlg,
                            newext.szExt.as_ptr() as LPCWSTR,
                            newext.szFileType.as_ptr() as LPCWSTR,
                            pGlobals,
                        );
                    }
                }

                IDC_FILETYPES_DELETE => {
                    FileTypesDlg_OnDelete(hwndDlg);
                }

                IDC_FILETYPES_CHANGE => {
                    pEntry = FileTypesDlg_GetEntry(GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT), -1);
                    if !pEntry.is_null() {
                        let mut oai: OPENASINFO = core::mem::zeroed();
                        oai.pcszFile = (*pEntry).FileExtension.as_ptr() as LPCWSTR;
                        oai.pcszClass = 0 as LPCWSTR;
                        oai.oaifInFlags = OAIF_FORCE_REGISTRATION as INT | OAIF_REGISTER_EXT as INT;
                        if SHOpenWithDialog(hwndDlg, &mut oai) >= 0 {
                            crate::dialogs_filetypes::_FILE_TYPE_ENTRY_InvalidateDefaultApp_body(pEntry.cast());
                            FileTypesDlg_OnItemChanging(hwndDlg, pEntry, pGlobals);
                        }
                    }
                }

                IDC_FILETYPES_ADVANCED => {
                    let edittype_ptr = edittype.as_mut_ptr();
                    (*edittype_ptr).hwndLV = GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT);
                    (*edittype_ptr).pG = pGlobals;
                    (*edittype_ptr).pEntry = FileTypesDlg_GetEntry((*edittype_ptr).hwndLV, -1);
                    if Normalize((*edittype_ptr).pEntry) != 0 {
                        DialogBoxParamW(
                            shell32_hInstance,
                            MAKEINTRESOURCEW(IDD_EDITTYPE),
                            hwndDlg,
                            Some(EditTypeDlgProc),
                            edittype_ptr as LPARAM,
                        );
                        FileTypesDlg_OnItemChanging(hwndDlg, (*edittype_ptr).pEntry, pGlobals);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            lppl = lParam as *mut _NMLISTVIEW;
            match (*lppl).hdr.code as INT {
                LVN_GETDISPINFO => {
                    let pLVDI = lParam as *mut _NMLVDISPINFOW;
                    let entry = (*pLVDI).item.lParam as PFILE_TYPE_ENTRY;
                    if !entry.is_null() && ((*pLVDI).item.mask & LVIF_TEXT) != 0 {
                        if (*pLVDI).item.iSubItem == 1 {
                            (*pLVDI).item.pszText = GetTypeName(entry, pGlobals);
                            (*pLVDI).item.mask |= LVIF_DI_SETITEM;
                        }
                    }
                }

                LVN_KEYDOWN => {
                    let pKeyDown = lParam as *mut _LV_KEYDOWN;
                    match (*pKeyDown).wVKey as UINT {
                        VK_DELETE => {
                            FileTypesDlg_OnDelete(hwndDlg);
                        }
                        VK_F5 => {
                            FileTypesDlg_Refresh(hwndDlg, (*pKeyDown).hdr.hwndFrom, pGlobals);
                        }
                        _ => {}
                    }
                }

                NM_DBLCLK => {
                    SendMessage(hwndDlg, WM_COMMAND, IDC_FILETYPES_ADVANCED as WPARAM, 0);
                }

                LVN_DELETEALLITEMS => {
                    return FALSE as INT_PTR;
                }

                LVN_DELETEITEM => {
                    pEntry = FileTypesDlg_GetEntry((*lppl).hdr.hwndFrom, (*lppl).iItem);
                    if !pEntry.is_null() {
                        crate::dialogs_filetypes::_FILE_TYPE_ENTRY_DestroyIcons_body(pEntry.cast());
                        HeapFree((*pGlobals).hHeap, 0, pEntry as PVOID);
                    }
                    return FALSE as INT_PTR;
                }

                LVN_ITEMCHANGING => {
                    pEntry = FileTypesDlg_GetEntry((*lppl).hdr.hwndFrom, (*lppl).iItem);
                    if pEntry.is_null() {
                        return TRUE as INT_PTR;
                    }

                    if ((*lppl).uOldState & LVIS_FOCUSED) == 0 && ((*lppl).uNewState & LVIS_FOCUSED) != 0 {
                        FileTypesDlg_OnItemChanging(hwndDlg, pEntry, pGlobals);
                    }
                }

                LVN_COLUMNCLICK => {
                    FileTypesDlg_Sort(pGlobals, (*lppl).hdr.hwndFrom, (*lppl).iSubItem);
                }

                PSN_SETACTIVE => {
                    SetFocus(GetDlgItem(hwndDlg, IDC_FILETYPES_LISTVIEW as INT));
                }

                _ => {}
            }
        }

        _ => {}
    }

    FALSE as INT_PTR
}
