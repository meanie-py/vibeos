//! FLIPPED: this TU is wired into the crate and shipping; the C++ original is
//! commented out of dll/win32/shell32/CMakeLists.txt.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_2CBECA8C1D10: [WCHAR; 3] = [0x002A, 0x003F, 0x0000];
pub static CPP_COM_WSTR_77ECA80E282A: [WCHAR; 3] = [0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_7F8732C542BA: [WCHAR; 4] = [0x0041, 0x003A, 0x005C, 0x0000];
pub static CPP_COM_WSTR_926483D22C70: [WCHAR; 6] = [0x0020, 0x0028, 0x0025, 0x0064, 0x0029, 0x0000];
pub static CPP_COM_WSTR_955300DD21E1: [WCHAR; 4] = [0x002A, 0x002E, 0x002A, 0x0000];
pub static CPP_COM_WSTR_A5DBFDEBD401: [WCHAR; 12] = [0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0x0000];
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];
pub static CPP_COM_WSTR_C245AFF91259: [WCHAR; 2] = [0x002E, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

use core::ffi::c_void;
use crate::prelude::*;
use crate::shell32::sdk_consts::*;

// ---- file-scope preamble (translated) ----
pub const FO_MASK: u32 = 0xF;

pub const NEW_FILENAME_ON_COPY_TRIES: i32 = 100;

#[repr(C)]
pub struct FILE_OPERATION {
    pub req: *mut SHFILEOPSTRUCTW,
    pub dwYesToAllMask: u32,
    pub bManyItems: i32,
    pub bCancelled: i32,
    pub progress: *mut IProgressDialog,
    pub completedSize: ULARGE_INTEGER,
    pub totalSize: ULARGE_INTEGER,
    pub szBuilderString: [u16; 50],
    pub Callback: FILEOPCALLBACK,
    pub CallerCallbackData: *mut core::ffi::c_void,
    pub hWndOwner: HWND,
    pub bHasDisplayedError: i32,
}

pub const ERROR_SHELL_INTERNAL_FILE_NOT_FOUND: u32 = 1026;

#[repr(C)]
pub struct FILE_ENTRY {
    pub attributes: u32,
    pub szDirectory: LPWSTR,
    pub szFilename: LPWSTR,
    pub szFullPath: LPWSTR,
    pub bFromWildcard: i32,
    pub bFromRelative: i32,
    pub bExists: i32,
}

/// `IProgressDialog` — shlobj.h:983. The bindgen islands render it OPAQUE
/// (`IProgressDialog__bindgen_vtable(c_void)`), because it is a C++ pure-virtual class, so
/// there are no method bindings to call and the translation's `p.SetTitle(..)` method syntax
/// resolves to nothing. Spell the vtable out and dispatch through it, as the landed COM call
/// sites do. A file-local item SHADOWS the prelude's glob, which is how `recycle_bin` and
/// friends carry their own interface structs without colliding.
///
/// **The field ORDER is the ABI.** It follows shlobj.h exactly — IUnknown's three first, then
/// the eleven IProgressDialog methods. Reordering silently calls the wrong slot.
#[repr(C)]
pub struct IProgressDialogVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IProgressDialog, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IProgressDialog) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IProgressDialog) -> ULONG,
    pub StartProgressDialog: unsafe extern "system" fn(
        *mut IProgressDialog,
        HWND,
        *mut c_void,
        DWORD,
        *const c_void,
    ) -> HRESULT,
    pub StopProgressDialog: unsafe extern "system" fn(*mut IProgressDialog) -> HRESULT,
    pub SetTitle: unsafe extern "system" fn(*mut IProgressDialog, LPCWSTR) -> HRESULT,
    pub SetAnimation:
        unsafe extern "system" fn(*mut IProgressDialog, HINSTANCE, UINT) -> HRESULT,
    pub HasUserCancelled: unsafe extern "system" fn(*mut IProgressDialog) -> BOOL,
    pub SetProgress: unsafe extern "system" fn(*mut IProgressDialog, DWORD, DWORD) -> HRESULT,
    pub SetProgress64:
        unsafe extern "system" fn(*mut IProgressDialog, ULONGLONG, ULONGLONG) -> HRESULT,
    pub SetLine: unsafe extern "system" fn(
        *mut IProgressDialog,
        DWORD,
        LPCWSTR,
        BOOL,
        *const c_void,
    ) -> HRESULT,
    pub SetCancelMsg:
        unsafe extern "system" fn(*mut IProgressDialog, LPCWSTR, *const c_void) -> HRESULT,
    pub Timer: unsafe extern "system" fn(*mut IProgressDialog, DWORD, *const c_void) -> HRESULT,
}

#[repr(C)]
pub struct IProgressDialog {
    pub lpVtbl: *const IProgressDialogVtbl,
}

/// `#define IsDotDir(x) ((x[0]=='.') && ((x[1]==0) || ((x[1]=='.') && (x[2]==0))))`
/// — shlfileop.cpp:32, file-local. Every call site already passes a `*const u16` and compares
/// the result against 0, so this stays an inline fn returning BOOL rather than a macro.
#[inline]
unsafe fn IsDotDir(x: *const u16) -> BOOL {
    (*x == b'.' as u16 && (*x.add(1) == 0 || (*x.add(1) == b'.' as u16 && *x.add(2) == 0))) as BOOL
}

/// `SHELL_ErrorBox` is already bound crate-wide against its `_RustBridge` WINAPI shim; import
/// it rather than re-declaring, so there is one spelling of the shim's signature.
use crate::win32::SHELL_ErrorBox;

/// `CLSCTX_INPROC_SERVER` — com-sys spells enum constants `tag<ENUM>_<NAME>`, and the island
/// re-exports the enum TYPES only, not their constants. Alias the bare name, as the prelude
/// does for `REST_CLASSICSHELL`.
use com_sys::tagCLSCTX_CLSCTX_INPROC_SERVER as CLSCTX_INPROC_SERVER;

extern "system" {
    /// shlwapi, `WINAPI` — hence `extern "system"`, matching how `recycle_bin` binds it.
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
}

extern "C" {
    /// The genuine NARROW CRT `strcpy`: `SheGetDirA` and `IsNetDrive` operate on `char`
    /// buffers, so this is not the `strcpyW` A/W alias the rest of the file uses.
    fn strcpy(dst: *mut core::ffi::c_char, src: *const core::ffi::c_char)
        -> *mut core::ffi::c_char;
}

extern "C" {
    /// `folders/CRecycleBin.h` — plain C++ free functions with no `WINAPI`, so they are
    /// __cdecl and belong in an `extern "C"` block, NOT `extern "system"`. The header now
    /// declares them `EXTERN_C` so the C++ definition agrees on the unmangled name.
    /// (`extern "C"` is LINKAGE, `WINAPI` is CALLING CONVENTION; i386 is the only arch that
    /// catches getting the second one wrong.)
    fn TRASH_CanTrashFile(wszPath: LPCWSTR) -> BOOL;
    fn TRASH_TrashFile(wszPath: LPCWSTR) -> BOOL;
}

/// `ntbasedef.h:939 #define MAKELANGID(p, s) ((((USHORT)(s)) << 10) | (USHORT)(p))`.
/// Each half is narrowed to USHORT BEFORE the shift, as the macro does.
#[inline]
fn MAKELANGID(p: USHORT, s: USHORT) -> DWORD {
    (((s as DWORD) << 10) | (p as DWORD)) as DWORD
}

/// `commctrl.h:4831 #define DSA_GetItemCount(hdsa) (*(int *) (hdsa))` — a MACRO reading the
/// count out of the DSA header, NOT a comctl32 export. Declaring it `extern` would compile
/// and then fail to link against a symbol that does not exist.
#[inline]
unsafe fn DSA_GetItemCount(hdsa: HDSA) -> INT {
    *(hdsa as *const INT)
}

/// `strsafe.h` — the byte-counted `StringCchCopyW`. An inline in the header, so there is no
/// import to bind; `dialogs/filedefext.rs` carries the same shim.
#[inline]
unsafe fn StringCbCopyW(dst: LPWSTR, bytes: usize, src: LPCWSTR) -> HRESULT {
    StringCchCopyW(dst, bytes / core::mem::size_of::<WCHAR>(), src)
}

/// `#define GET_FILENAME(fe) ((fe)->szFilename[0] ? (fe)->szFilename : (fe)->szFullPath)`
/// — shlfileop.cpp:1995, file-local. A `macro_rules!`, not a fn: an out-of-line no-op or
/// wrapper for a C macro is the optimization barrier that uncloaks latent ABI mismatches.
macro_rules! GET_FILENAME {
    ($fe:expr) => {{
        let fe = $fe;
        if *(*fe).szFilename != 0 {
            (*fe).szFilename
        } else {
            (*fe).szFullPath
        }
    }};
}

#[repr(C)]
pub struct FILE_LIST {
    pub feFiles: *mut FILE_ENTRY,
    pub num_alloc: u32,
    pub dwNumFiles: u32,
    pub bAnyFromWildcard: i32,
    pub bAnyDirectories: i32,
    pub bAnyDontExist: i32,
}

#[repr(C)]
pub struct confirm_msg_info {
    pub lpszText: LPWSTR,
    pub lpszCaption: LPWSTR,
    pub hIcon: HICON,
    pub bYesToAll: i32,
}

#[repr(C)]
pub struct SHELL_ConfirmIDstruc {
    pub hIconInstance: HINSTANCE,
    pub icon_resource_id: u32,
    pub caption_resource_id: u32,
    pub text_resource_id: u32,
}
unsafe extern "C" fn SHELL32_FileOpErrorToHResult(err: i32, any_operations_aborted: BOOLEAN) -> HRESULT {
    let _ = FALSE;
    let de_min: i32 = (DE_SAMEFILE) as i32;
    let de_max: i32 = 0xB7;
    let is_de_err: bool = (err & (!ERRORONDEST) as i32) >= de_min
        && (err & (!ERRORONDEST) as i32) <= de_max;
    if err == (DE_OPCANCELLED) as i32 || any_operations_aborted != 0 {
        return HRESULT_FROM_WIN32((ERROR_CANCELLED) as i32);
    }
    if err == (DE_ACCESSDENIEDSRC) as i32 {
        return HRESULT_FROM_WIN32((ERROR_ACCESS_DENIED) as i32);
    }
    if err == (DE_FILENAMETOOLONG) as i32 {
        return HRESULT_FROM_WIN32((ERROR_FILENAME_EXCED_RANGE) as i32);
    }
    if err == (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32 {
        return HRESULT_FROM_WIN32((ERROR_FILE_NOT_FOUND) as i32);
    }
    if err == (ERROR_ALREADY_EXISTS) as i32 {
        return HRESULT_FROM_WIN32(err);
    }
    return if is_de_err { E_FAIL } else { HRESULT_FROM_WIN32(err) };
}
unsafe extern "C" fn CanShowFileOpErrorUI(fos: *const SHFILEOPSTRUCTW) -> BOOL {
    if ((*fos).fFlags & (FOF_SILENT) as u16) != 0 && (*fos).hwnd.is_null() {
        return FALSE;
    }
    if ((*fos).fFlags & (FOF_NOERRORUI) as u16) == 0 { 1 } else { 0 }
}
unsafe extern "C" fn HandleDesktopIniOp(Path: PCWSTR) {
    let Name = PathFindFileNameW(Path);
    let idx = 0 as usize;
    if ((*Name.add(idx) | 32) != 0x0064u16 || _wcsicmp(Name, CPP_COM_WSTR_A5DBFDEBD401.as_ptr()) != 0) {
        return;
    }
    let mut Dir: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    if FAILED_UNEXPECTEDLY!(StringCchCopyW(Dir.as_mut_ptr(), _countof!(&Dir) as usize, Path)) {
        return;
    }
    PathRemoveFileSpecW(Dir.as_mut_ptr());
    let attrs = GetFileAttributesW(Dir.as_ptr());
    if attrs & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY) == 0 {
        return;
    }
    SHChangeNotify((SHCNE_UPDATEITEM) as i32, SHCNF_PATHW, (Dir.as_ptr()).cast(), core::ptr::null_mut());
}
unsafe extern "C" fn confirm_msg_move_button(hDlg: HWND, iId: INT, xPos: *mut INT, yOffset: INT, bShow: BOOL) {
    let hButton = GetDlgItem(hDlg, iId);
    if bShow != 0 {
        let mut r_uninit = core::mem::MaybeUninit::<RECT>::uninit();
        GetWindowRect(hButton, r_uninit.as_mut_ptr());
        let r = r_uninit.assume_init();
        let width = r.right - r.left;
        let mut pt = POINT { x: r.left, y: r.top };
        ScreenToClient(hDlg, &mut pt);
        MoveWindow(
            hButton,
            *xPos - width,
            pt.y - yOffset,
            width,
            r.bottom - r.top,
            FALSE,
        );
        *xPos -= width + 5;
    } else {
        ShowWindow(hButton, (SW_HIDE) as i32);
    }
}
unsafe extern "C" fn ConfirmMsgBox_Paint(hDlg: HWND) -> INT_PTR {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let hdc: HDC;
    let hOldFont: HFONT;

    BeginPaint(hDlg, &mut ps);
    hdc = (ps.hdc).cast();
    SetBkMode(hdc, TRANSPARENT as i32);
    SetTextColor(hdc, GetSysColor((COLOR_BTNTEXT) as i32));

    GetClientRect(GetDlgItem(hDlg, (IDC_YESTOALL_MESSAGE) as i32), &mut r);
    MapWindowPoints(
        GetDlgItem(hDlg, (IDC_YESTOALL_MESSAGE) as i32),
        hDlg,
        (&mut r as *mut RECT) as *mut POINT,
        2,
    );

    hOldFont = (SelectObject(
        hdc,
        (SendDlgItemMessageW(
            hDlg,
            (IDC_YESTOALL_MESSAGE) as i32,
            WM_GETFONT,
            0,
            0,
        ) as HFONT).cast(),
    )).cast();

    DrawTextW(
        hdc,
        GetWindowLongPtrW(hDlg, DWLP_USER) as *mut u16,
        -(1),
        &mut r,
        DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_WORDBREAK,
    );

    SelectObject(hdc, (hOldFont).cast());
    EndPaint(hDlg, &mut ps);

    (TRUE) as INT_PTR
}
unsafe extern "C" fn ConfirmMsgBox_Init(hDlg: HWND, lParam: LPARAM) -> INT_PTR {
    let info = lParam as *mut confirm_msg_info;
    let mut xPos: INT = 0;
    let mut yOffset: INT;
    let mut width: i32;
    let mut height: i32;
    let hOldFont: HFONT;
    let hdc: HDC;
    let mut r: RECT = core::mem::zeroed();

    SetWindowTextW(hDlg, (*info).lpszCaption);
    ShowWindow(GetDlgItem(hDlg, (IDC_YESTOALL_MESSAGE) as i32), (SW_HIDE) as i32);
    SetWindowLongPtrW(hDlg, DWLP_USER as i32, (*info).lpszText as LONG_PTR);
    // `as WPARAM`, not `as u64`: WPARAM is UINT_PTR, so a literal 64 bakes amd64's width and
    // does not even compile on i386. Name the semantic alias for every pointer-width cast.
    SendDlgItemMessageW(hDlg, (IDC_YESTOALL_ICON) as i32, STM_SETICON, ((*info).hIcon as usize) as WPARAM, 0);

    GetClientRect(GetDlgItem(hDlg, (IDC_YESTOALL_MESSAGE) as i32), core::ptr::addr_of_mut!(r));
    hdc = GetDC(hDlg);
    yOffset = r.bottom;
    hOldFont = SelectObject(hdc, (SendDlgItemMessageW(hDlg, (IDC_YESTOALL_MESSAGE) as i32, WM_GETFONT, 0, 0) as HFONT).cast()) as HFONT;
    DrawTextW(hdc, (*info).lpszText, -1, core::ptr::addr_of_mut!(r), DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_WORDBREAK | DT_CALCRECT);
    SelectObject(hdc, (hOldFont).cast());
    yOffset -= r.bottom;
    yOffset = min(yOffset, 35);
    ReleaseDC(hDlg, hdc);

    GetClientRect(hDlg, core::ptr::addr_of_mut!(r));
    xPos = r.right - 7;
    GetWindowRect(hDlg, core::ptr::addr_of_mut!(r));
    width = r.right - r.left;
    height = r.bottom - r.top - yOffset;
    MoveWindow(hDlg, (GetSystemMetrics((SM_CXSCREEN) as i32) - width) / 2,
        (GetSystemMetrics((SM_CYSCREEN) as i32) - height) / 2, width, height, FALSE);

    confirm_msg_move_button(hDlg, (IDCANCEL) as i32, core::ptr::addr_of_mut!(xPos), yOffset, (*info).bYesToAll);
    confirm_msg_move_button(hDlg, (IDNO) as i32, core::ptr::addr_of_mut!(xPos), yOffset, TRUE);
    confirm_msg_move_button(hDlg, (IDC_YESTOALL) as i32, core::ptr::addr_of_mut!(xPos), yOffset, (*info).bYesToAll);
    confirm_msg_move_button(hDlg, (IDYES) as i32, core::ptr::addr_of_mut!(xPos), yOffset, TRUE);

    (TRUE) as INT_PTR
}
unsafe extern "system" fn ConfirmMsgBoxProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return ConfirmMsgBox_Init(hDlg, lParam);
        }
        WM_PAINT => {
            return ConfirmMsgBox_Paint(hDlg);
        }
        WM_COMMAND => {
            EndDialog(hDlg, (wParam) as INT_PTR);
        }
        WM_CLOSE => {
            EndDialog(hDlg, (IDCANCEL) as INT_PTR);
        }
        _ => {}
    }
    (FALSE) as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_ConfirmMsgBox(
    hWnd: HWND,
    lpszText: LPWSTR,
    lpszCaption: LPWSTR,
    hIcon: HICON,
    bYesToAll: BOOL,
) -> i32 {
    let mut info = confirm_msg_info {
        lpszText,
        lpszCaption,
        hIcon,
        bYesToAll,
    };
    DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDD_YESTOALL_MSGBOX),
        hWnd,
        Some(ConfirmMsgBoxProc),
        &mut info as *mut confirm_msg_info as LPARAM,
    ) as i32
}
unsafe extern "C" fn SHELL_ConfirmIDs(nKindOfDialog: i32, ids: *mut SHELL_ConfirmIDstruc) -> BOOL {
    (*ids).hIconInstance = shell32_hInstance;
    // Cast the SCRUTINEE, never the patterns: a cast is not legal in a pattern, and the
    // `ASK_*` are island `u32` #defines against C's `int nKindOfDialog`. Arch-independent
    // (all three islands agree), so widening here is not a width decision.
    match nKindOfDialog as u32 {
        ASK_DELETE_FILE => {
            (*ids).icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_DELETEITEM_TEXT;
            TRUE
        }
        ASK_DELETE_FOLDER => {
            (*ids).icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
            (*ids).caption_resource_id = IDS_DELETEFOLDER_CAPTION;
            (*ids).text_resource_id = IDS_DELETEITEM_TEXT;
            TRUE
        }
        ASK_DELETE_MULTIPLE_ITEM => {
            (*ids).icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_DELETEMULTIPLE_TEXT;
            TRUE
        }
        ASK_TRASH_FILE => {
            (*ids).icon_resource_id = IDI_SHELL_TRASH_FILE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_TRASHITEM_TEXT;
            TRUE
        }
        ASK_TRASH_FOLDER => {
            (*ids).icon_resource_id = IDI_SHELL_TRASH_FILE;
            (*ids).caption_resource_id = IDS_DELETEFOLDER_CAPTION;
            (*ids).text_resource_id = IDS_TRASHFOLDER_TEXT;
            TRUE
        }
        ASK_TRASH_MULTIPLE_ITEM => {
            (*ids).icon_resource_id = IDI_SHELL_TRASH_FILE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_TRASHMULTIPLE_TEXT;
            TRUE
        }
        ASK_CANT_TRASH_ITEM => {
            (*ids).icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_CANTTRASH_TEXT;
            TRUE
        }
        ASK_DELETE_SELECTED => {
            (*ids).icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
            (*ids).caption_resource_id = IDS_DELETEITEM_CAPTION;
            (*ids).text_resource_id = IDS_DELETESELECTED_TEXT;
            TRUE
        }
        ASK_OVERWRITE_FILE => {
            (*ids).icon_resource_id = IDI_SHELL_FOLDER_MOVE2;
            (*ids).caption_resource_id = IDS_OVERWRITEFILE_CAPTION;
            (*ids).text_resource_id = IDS_OVERWRITEFILE_TEXT;
            TRUE
        }
        ASK_OVERWRITE_FOLDER => {
            (*ids).icon_resource_id = IDI_SHELL_FOLDER_MOVE2;
            (*ids).caption_resource_id = IDS_OVERWRITEFILE_CAPTION;
            (*ids).text_resource_id = IDS_OVERWRITEFOLDER_TEXT;
            TRUE
        }
        _ => {
            FIXME!(" Unhandled nKindOfDialog %d stub\n", nKindOfDialog);
            FALSE
        }
    }
}
unsafe extern "C" fn SHELL_ConfirmDialogW(
    hWnd: HWND,
    nKindOfDialog: i32,
    szDir: LPCWSTR,
    op: *mut FILE_OPERATION,
) -> BOOL {
    let mut szCaption: [u16; 255 as usize] = [0u16; 255 as usize];
    let mut szText: [u16; 255 as usize] = [0u16; 255 as usize];
    let mut szBuffer: [u16; (MAX_PATH + 256) as usize] = [0u16; (MAX_PATH + 256) as usize];
    let mut ids: SHELL_ConfirmIDstruc = core::mem::zeroed();
    let mut args: [DWORD_PTR; 1 as usize] = [0 as DWORD_PTR; 1 as usize];
    let hIcon: HICON;
    let mut ret: i32;

    assert!(nKindOfDialog >= 0 && nKindOfDialog < 32);

    if !op.is_null() && ((*op).dwYesToAllMask & (1 << nKindOfDialog)) != 0 {
        return TRUE;
    }

    if SHELL_ConfirmIDs(nKindOfDialog, &mut ids as *mut _) == 0 {
        return FALSE;
    }

    LoadStringW(
        shell32_hInstance,
        ids.caption_resource_id,
        szCaption.as_mut_ptr(),
        (core::mem::size_of_val(&szCaption) / core::mem::size_of::<u16>()) as i32,
    );
    LoadStringW(
        shell32_hInstance,
        ids.text_resource_id,
        szText.as_mut_ptr(),
        (core::mem::size_of_val(&szText) / core::mem::size_of::<u16>()) as i32,
    );

    args[0 as usize] = szDir as DWORD_PTR;
    FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ARGUMENT_ARRAY,
        szText.as_ptr() as *const c_void,
        0,
        0,
        szBuffer.as_mut_ptr() as PWSTR,
        core::mem::size_of_val(&szBuffer) as u32,
        args.as_mut_ptr() as *mut va_list,
    );

    hIcon = LoadIconW(
        ids.hIconInstance,
        MAKEINTRESOURCE(ids.icon_resource_id) as LPWSTR,
    );

    ret = SHELL_ConfirmMsgBox(
        hWnd,
        szBuffer.as_mut_ptr() as LPWSTR,
        szCaption.as_mut_ptr() as LPWSTR,
        hIcon,
        if !op.is_null() { (*op).bManyItems } else { (0u8) as i32 },
    );

    if !op.is_null() {
        if ret == (IDC_YESTOALL) as i32 {
            (*op).dwYesToAllMask |= 1 << nKindOfDialog;
            ret = (IDYES) as i32;
        }
        if ret == (IDCANCEL) as i32 {
            (*op).bCancelled = TRUE;
        }
        if ret != (IDYES) as i32 {
            (*(*op).req).fAnyOperationsAborted = TRUE;
        }
    }

    (ret == (IDYES) as i32) as BOOL
}

#[no_mangle]
pub unsafe extern "C" fn SHELL_ConfirmYesNoW(
    hWnd: HWND,
    nKindOfDialog: i32,
    szDir: LPCWSTR,
) -> BOOL {
    SHELL_ConfirmDialogW(hWnd, nKindOfDialog, szDir, core::ptr::null_mut())
}
unsafe extern "C" fn SHELL32_AnsiToUnicodeBuf(aPath: *const u8, wPath: *mut PWSTR, minChars: u32) -> u32 {
    let mut len: u32 = (MultiByteToWideChar(
        CP_ACP,
        0,
        (aPath).cast(),
        -(1 as i32),
        core::ptr::null_mut(),
        0,
    )) as u32;
    if len < minChars {
        len = minChars;
    }
    *wPath = HeapAlloc(
        GetProcessHeap(),
        0,
        // C: `len * sizeof(WCHAR)`. The translation dropped the sizeof to a literal `0`, so
        // this asked the heap for ZERO bytes and the MultiByteToWideChar below then wrote
        // `len` wide chars into it. Compiles clean, overflows the heap on first ANSI call.
        (len as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as PWSTR;
    if !(*wPath).is_null() {
        MultiByteToWideChar(
            CP_ACP,
            0,
            (aPath).cast(),
            -(1 as i32),
            *wPath,
            (len) as i32,
        );
        return NO_ERROR;
    }
    (E_OUTOFMEMORY) as u32
}
unsafe extern "C" fn SHELL32_FreeUnicodeBuf(wPath: PWSTR) {
    HeapFree(GetProcessHeap(), 0, (wPath).cast());
}

#[no_mangle]
pub unsafe extern "system" fn SHIsFileAvailableOffline(
    path: LPCWSTR,
    status: LPDWORD,
) -> HRESULT {
    // FIXME("(%s, %p) stub\n", debugstr_w(path), status);
    E_FAIL
}
unsafe extern "C" fn FileOpCallback(op: *mut FILE_OPERATION, Event: FILEOPCALLBACKEVENT, Source: PCWSTR, Destination: PCWSTR, Attributes: UINT, hrOp: HRESULT) -> HRESULT {
    let _ = S_OK;
    if (Attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_SYSTEM)) == FILE_ATTRIBUTE_SYSTEM {
        if Event == FOCE_POSTDELETEITEM {
            HandleDesktopIniOp(Source);
        }
        if Event == FOCE_POSTCOPYITEM {
            HandleDesktopIniOp(Destination);
        }
        if Event == FOCE_POSTMOVEITEM || Event == FOCE_POSTRENAMEITEM {
            HandleDesktopIniOp(Source);
            HandleDesktopIniOp(Destination);
        }
    }
    let mut hr: HRESULT = S_OK;
    if let Some(callback) = (*op).Callback {
        hr = callback(Event, Source, Destination, Attributes, hrOp, (*op).CallerCallbackData);
        if hr == HRESULT_FROM_WIN32((ERROR_CANCELLED) as i32) {
            (*op).bCancelled = TRUE;
        }
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_DeleteDirectoryW(
    op: *mut FILE_OPERATION,
    pszDir: PCWSTR,
    bShowUI: BOOL,
) -> BOOL {
    let mut ret: BOOL = TRUE;
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut szTemp: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    PathCombineW(szTemp.as_mut_ptr(), pszDir, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());
    let hFind = FindFirstFileW(szTemp.as_ptr(), &mut wfd);
    if hFind == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    if FileOpCallback(
        op,
        FOCE_PREDELETEITEM,
        pszDir,
        core::ptr::null(),
        wfd.dwFileAttributes,
        S_OK,
    ) < 0
    {
        return FALSE;
    }

    if bShowUI == 0 || {
        ret = SHELL_ConfirmDialogW(
            (*op).hWndOwner,
            (ASK_DELETE_FOLDER) as i32,
            pszDir,
            core::ptr::null_mut(),
        );
        ret != 0
    } {
        loop {
            if IsDotDir(wfd.cFileName.as_ptr()) != 0 {
                // skip file/dir processing – check condition directly
            } else {
                PathCombineW(szTemp.as_mut_ptr(), pszDir, wfd.cFileName.as_ptr());
                if (FILE_ATTRIBUTE_DIRECTORY & wfd.dwFileAttributes) != 0 {
                    ret = SHELL_DeleteDirectoryW(op, szTemp.as_ptr(), FALSE);
                } else {
                    ret = (SHNotifyDeleteFileW(op, szTemp.as_ptr()) == (ERROR_SUCCESS) as u32) as BOOL;
                }

                if !(*op).progress.is_null() {
                    (*op).bCancelled |= ((*(*(*op).progress).lpVtbl).HasUserCancelled)((*op).progress);
                }
            }

            if ret == 0
                || FindNextFileW(hFind, &mut wfd) == 0
                || (*op).bCancelled != 0
            {
                break;
            }
        }
    }

    FindClose(hFind);

    if ret != 0 {
        ret = (SHNotifyRemoveDirectoryW(pszDir) == (ERROR_SUCCESS) as u32) as BOOL;
    }

    FileOpCallback(
        op,
        FOCE_POSTDELETEITEM,
        pszDir,
        core::ptr::null(),
        wfd.dwFileAttributes,
        if ret != 0 {
            S_OK
        } else {
            E_FAIL
        },
    );

    ret
}
unsafe extern "C" fn SHNotifyCreateDirectoryW(path: PCWSTR, sec: *mut SECURITY_ATTRIBUTES) -> u32 {
    if CreateDirectoryW(path, sec) != 0 {
        SHChangeNotify((SHCNE_MKDIR) as i32, SHCNF_PATHW, (path).cast(), core::ptr::null_mut());
        return (ERROR_SUCCESS) as u32;
    }
    GetLastError()
}

#[no_mangle]
pub unsafe extern "system" fn Win32CreateDirectoryW(
    path: PCWSTR,
    sec: *mut SECURITY_ATTRIBUTES,
) -> i32 {
    if SHNotifyCreateDirectoryW(path, sec) == (ERROR_SUCCESS) as u32 {
        1i32
    } else {
        0i32
    }
}
unsafe extern "C" fn SHNotifyRemoveDirectoryW(path: PCWSTR) -> u32 {
    let mut ret: i32 = RemoveDirectoryW(path);
    if ret == 0 {
        let dwAttr = GetFileAttributesW(path);
        if IsAttrib!(dwAttr, FILE_ATTRIBUTE_READONLY) {
            if SetFileAttributesW(path, dwAttr & !FILE_ATTRIBUTE_READONLY) != 0 {
                ret = RemoveDirectoryW(path);
            }
        }
    }
    if ret != 0 {
        SHChangeNotify((SHCNE_RMDIR) as i32, SHCNF_PATHW, (path).cast(), core::ptr::null_mut::<core::ffi::c_void>());
        return (ERROR_SUCCESS) as u32;
    }
    return GetLastError();
}

#[no_mangle]
pub unsafe extern "system" fn Win32RemoveDirectoryW(path: PCWSTR) -> i32 {
    (SHNotifyRemoveDirectoryW(path) == (ERROR_SUCCESS) as u32) as i32
}
unsafe extern "C" fn _SetOperationTitle(op: *mut FILE_OPERATION) {
    if (*op).progress.is_null() {
        return;
    }
    let progress = (*op).progress;
    let mut szTitle: [u16; 50] = [0u16; 50];
    let mut szPreflight: [u16; 50] = [0u16; 50];
    let mut animation_id: u32 = 0;

    match (*((*op).req)).wFunc {
        FO_COPY => {
            LoadStringW(shell32_hInstance, IDS_FILEOOP_COPYING, szTitle.as_mut_ptr(), (core::mem::size_of_val(&szTitle) / 2) as i32);
            LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM_TO, (*op).szBuilderString.as_mut_ptr(), 50);
            animation_id = IDA_SHELL_COPY;
        },
        FO_DELETE => {
            LoadStringW(shell32_hInstance, IDS_FILEOOP_DELETING, szTitle.as_mut_ptr(), (core::mem::size_of_val(&szTitle) / 2) as i32);
            LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM, (*op).szBuilderString.as_mut_ptr(), 50);
            animation_id = IDA_SHELL_DELETE;
        },
        FO_MOVE => {
            LoadStringW(shell32_hInstance, IDS_FILEOOP_MOVING, szTitle.as_mut_ptr(), (core::mem::size_of_val(&szTitle) / 2) as i32);
            LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM_TO, (*op).szBuilderString.as_mut_ptr(), 50);
            animation_id = IDA_SHELL_COPY;
        },
        _ => return,
    };

    LoadStringW(shell32_hInstance, IDS_FILEOOP_PREFLIGHT, szPreflight.as_mut_ptr(), (core::mem::size_of_val(&szPreflight) / 2) as i32);

    ((*(*progress).lpVtbl).SetTitle)(progress, szTitle.as_ptr());
    ((*(*progress).lpVtbl).SetLine)(progress, 1, szPreflight.as_ptr(), 0, core::ptr::null());
    ((*(*progress).lpVtbl).SetAnimation)(progress, shell32_hInstance, animation_id);
}
unsafe extern "C" fn _SetOperationTexts(op: *mut FILE_OPERATION, src: PCWSTR, dest: PCWSTR) {
    if (*op).progress.is_null() || src.is_null() {
        return;
    }
    let mut szFolderS: [u16; 50] = [0; 50];
    let mut szFolderD: [u16; 50] = [0; 50];
    let mut szFinalString: [u16; 260] = [0; 260];
    let mut args: [DWORD_PTR; 2] = [0; 2];

    let mut fileSpecS: *const u16 = src;
    let mut pathSpecS: *const u16 = src;

    let mut ptr: *const u16 = src;
    while *ptr != 0 {
        if *ptr == ('\\' as u16) {
            pathSpecS = fileSpecS;
            fileSpecS = ptr.offset(1);
        }
        ptr = ptr.offset(1);
    }

    let diff = ((fileSpecS as usize) - (pathSpecS as usize)) / core::mem::size_of::<u16>();
    let length = min(50, diff as core::ffi::c_int);
    lstrcpynW(szFolderS.as_mut_ptr(), pathSpecS, length);
    args[0] = szFolderS.as_ptr() as DWORD_PTR;

    match (*(*op).req).wFunc {
        FO_COPY | FO_MOVE => {
            if dest.is_null() {
                return;
            }
            let mut fileSpecD: *const u16 = dest;
            let mut pathSpecD: *const u16 = dest;
            let mut ptr: *const u16 = dest;
            while *ptr != 0 {
                if *ptr == ('\\' as u16) {
                    pathSpecD = fileSpecD;
                    fileSpecD = ptr.offset(1);
                }
                ptr = ptr.offset(1);
            }
            let diff = ((fileSpecD as usize) - (pathSpecD as usize)) / core::mem::size_of::<u16>();
            let length = min(50, diff as core::ffi::c_int);
            lstrcpynW(szFolderD.as_mut_ptr(), pathSpecD, length);
            args[1] = szFolderD.as_ptr() as DWORD_PTR;
        }
        FO_DELETE => {}
        _ => { return; }
    }

    FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ARGUMENT_ARRAY,
        (*op).szBuilderString.as_ptr() as *const c_void,
        0,
        0,
        szFinalString.as_mut_ptr(),
        core::mem::size_of_val(&szFinalString) as u32,
        args.as_ptr() as *mut va_list,
    );

    let progress = (*op).progress;
    ((*(*progress).lpVtbl).SetLine)(progress, 1, fileSpecS, 0, core::ptr::null());
    ((*(*progress).lpVtbl).SetLine)(progress, 2, szFinalString.as_mut_ptr(), 0, core::ptr::null());
}
/// The body. Kept separate from the exported entry point below so there is exactly one copy.
unsafe fn SHCopyProgress(
    TotalFileSize: i64,
    TotalBytesTransferred: i64,
    dwCallbackReason: u32,
    lpData: *mut core::ffi::c_void,
) -> DWORD {
    let op = lpData as *mut FILE_OPERATION;
    if !((*op).progress).is_null() {
        if dwCallbackReason & CALLBACK_STREAM_SWITCH != 0 {
            (*op).completedSize.QuadPart += (TotalFileSize) as u64;
        }
        ((*(*(*op).progress).lpVtbl).SetProgress64)(
            (*op).progress,
            ((*op).completedSize.QuadPart - (TotalFileSize) as u64 + (TotalBytesTransferred) as u64) as u64,
            ((*op).totalSize.QuadPart) as u64,
        );
        (*op).bCancelled = ((*(*(*op).progress).lpVtbl).HasUserCancelled)((*op).progress);
    }
    0
}

// LPPROGRESS_ROUTINE is `extern "stdcall"` on i386 and `extern "C"` on amd64/arm64 — which is
// precisely what `extern "system"` means, but rustc keeps the two spellings as DISTINCT
// fn-pointer types, so `Some(routine)` is rejected unless the callback is written with the
// island's own per-arch spelling. Hence the cfg twins, the same idiom the dialog procs use.
// The params are also the island's real types (LARGE_INTEGER, not i64) for the same reason.
#[no_mangle]
pub unsafe extern "system" fn SHCopyProgressRoutine(
    TotalFileSize: LARGE_INTEGER,
    TotalBytesTransferred: LARGE_INTEGER,
    _StreamSize: LARGE_INTEGER,
    _StreamBytesTransferred: LARGE_INTEGER,
    _dwStreamNumber: DWORD,
    dwCallbackReason: DWORD,
    _hSourceFile: HANDLE,
    _hDestinationFile: HANDLE,
    lpData: LPVOID,
) -> DWORD {
    SHCopyProgress(TotalFileSize.QuadPart, TotalBytesTransferred.QuadPart, dwCallbackReason, lpData)
}
unsafe extern "C" fn SHNotifyDeleteFileW(op: *mut FILE_OPERATION, path: PCWSTR) -> u32 {
    let mut ret: BOOL;
    unsafe { _SetOperationTexts(op, path, core::ptr::null_mut()); }

    let mut filesize: i64 = 0;

    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfile: HANDLE = unsafe { FindFirstFileW(path, &mut wfd) };
    if hfile != INVALID_HANDLE_VALUE && IsAttribFile!(wfd.dwFileAttributes) {
        filesize = ((wfd.nFileSizeHigh as i64) << 32) | (wfd.nFileSizeLow as i64);
    }
    let attrib: u32 = if hfile != INVALID_HANDLE_VALUE { wfd.dwFileAttributes } else { 0 };
    let aborted: BOOL = (unsafe { FileOpCallback(op, FOCE_PREDELETEITEM, path, core::ptr::null(), attrib, S_OK) } < 0) as BOOL;
    unsafe { FindClose(hfile); }
    if aborted != 0 {
        return ERROR_CANCELLED;
    }

    ret = unsafe { DeleteFileW(path) };
    if ret == 0 {
        let dwattr = unsafe { GetFileAttributesW(path) };
        if IsAttrib!(dwattr, FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM) {
            if unsafe { SetFileAttributesW(path, dwattr & !(FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM)) } != 0 {
                ret = unsafe { DeleteFileW(path) };
            }
        }
    }
    unsafe {
        FileOpCallback(
            op,
            FOCE_POSTDELETEITEM,
            path,
            core::ptr::null(),
            attrib,
            if ret != 0 { S_OK } else { E_FAIL },
        );
    }
    if ret != 0 {
        unsafe {
            SHCopyProgress(filesize, filesize, CALLBACK_STREAM_SWITCH, op as *mut core::ffi::c_void);
            SHChangeNotify((SHCNE_DELETE) as i32, SHCNF_PATHW, (path).cast(), core::ptr::null());
        }
        return (ERROR_SUCCESS) as u32;
    }
    unsafe { GetLastError() }
}

#[no_mangle]
pub unsafe extern "system" fn Win32DeleteFileW(path: PCWSTR) -> u32 {
    (SHNotifyDeleteFileW(core::ptr::null_mut(), path) == (ERROR_SUCCESS) as u32) as u32
}
unsafe extern "C" fn CheckForError(op: *mut FILE_OPERATION, error: u32, src: PCWSTR) -> u32 {
    // `CStringW strTitle;` etc. — a C++ RAII local. The translation lowers it to an
    // UNINITIALISED binding, which runs neither the constructor nor the destructor; spell
    // the default ctor out so the Drop impl also runs. (`strMask` is declared and never
    // used in the C++ too, so it is dropped here rather than carried as a dead binding.)
    let mut strTitle = CStringW::new();
    let mut strText = CStringW::new();
    let mut lpMsgBuffer: PWSTR = core::ptr::null_mut();

    if error == (ERROR_SUCCESS) as u32 || CanShowFileOpErrorUI((*op).req) == 0 {
        return error;
    }

    strTitle.LoadStringW(if (*(*op).req).wFunc == FO_COPY { IDS_COPYERRORTITLE } else { IDS_MOVEERRORTITLE });

    FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
        core::ptr::null_mut(),
        error,
        MAKELANGID(LANG_NEUTRAL as u16, SUBLANG_DEFAULT as u16) as u32,
        core::ptr::addr_of_mut!(lpMsgBuffer) as PWSTR,
        0,
        core::ptr::null_mut(),
    );

    strText.FormatResStr2(
        if (*(*op).req).wFunc == FO_COPY { IDS_COPYERROR } else { IDS_MOVEERROR },
        PathFindFileNameW(src),
        lpMsgBuffer,
    );

    MessageBoxW((*op).hWndOwner, strText.as_ptr(), strTitle.as_ptr(), MB_ICONERROR);
    LocalFree((lpMsgBuffer).cast());

    (*op).bHasDisplayedError += 1;

    error
}
unsafe extern "C" fn SHNotifyMoveFileW(op: *mut FILE_OPERATION, src: PCWSTR, dest: PCWSTR, isdir: BOOL) -> u32 {
    _SetOperationTexts(op, src, dest);
    let is_ren: bool = (*(*op).req).wFunc == FO_RENAME;
    let mut attrib: u32 = GetFileAttributesW(src);
    if attrib == INVALID_FILE_ATTRIBUTES {
        attrib = if isdir != 0 { FILE_ATTRIBUTE_DIRECTORY } else { FILE_ATTRIBUTE_NORMAL };
    }
    let event = if is_ren { FOCE_PRERENAMEITEM } else { FOCE_PREMOVEITEM };
    if FileOpCallback(op, event, src, dest, attrib, S_OK) < 0 {
        return ERROR_CANCELLED;
    }
    let mut ret: i32 = MoveFileWithProgressW(src, dest, Some(SHCopyProgressRoutine), op as *mut c_void, MOVEFILE_REPLACE_EXISTING);
    if ret == 0 {
        ret = MoveFileW(src, dest);
    }
    let mut last_error: u32 = GetLastError();
    if ret == 0 {
        let mut dw_attr: u32 = SHFindAttrW(dest, FALSE);
        if dw_attr == INVALID_FILE_ATTRIBUTES {
            dw_attr = GetFileAttributesW(src);
            if IsAttrib!(dw_attr, FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM) {
                if SetFileAttributesW(src, dw_attr & !(FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM)) != 0 {
                    ret = MoveFileW(src, dest);
                    last_error = GetLastError();
                    SetFileAttributesW(if ret != 0 { dest } else { src }, dw_attr);
                }
            }
        }
    }
    FileOpCallback(
        op,
        if is_ren { FOCE_POSTRENAMEITEM } else { FOCE_POSTMOVEITEM },
        src,
        dest,
        attrib,
        if ret != 0 { S_OK } else { E_FAIL }
    );
    if ret != 0 {
        if is_ren {
            SHChangeNotify(
                if isdir != 0 { (SHCNE_RENAMEFOLDER) as i32 } else { (SHCNE_RENAMEITEM) as i32 },
                SHCNF_PATHW,
                src as *const c_void,
                dest as *const c_void
            );
        } else {
            SHChangeNotify(
                if isdir != 0 { (SHCNE_MKDIR) as i32 } else { (SHCNE_CREATE) as i32 },
                SHCNF_PATHW,
                (dest).cast(),
                core::ptr::null_mut()
            );
            SHChangeNotify(
                if isdir != 0 { (SHCNE_RMDIR) as i32 } else { (SHCNE_DELETE) as i32 },
                SHCNF_PATHW,
                (src).cast(),
                core::ptr::null_mut()
            );
        }
        return (ERROR_SUCCESS) as u32;
    }
    CheckForError(op, last_error, src)
}
unsafe extern "C" fn SHIsCdRom(path: PCWSTR) -> i32 {
    let _ = CPP_COM_WSTR_7F8732C542BA.as_ptr();
    if path.is_null() || *path.offset(0 as isize) == 0 {
        return FALSE;
    }
    if *path.offset(1 as isize) != UNICODE_NULL
        && *path.offset(1 as isize) != ':' as u16
    {
        return FALSE;
    }
    let mut tmp = [0u16; 4];
    tmp[0 as usize] = *path.offset(0 as isize);
    let drive_type = GetDriveTypeW(tmp.as_ptr());
    (drive_type == DRIVE_CDROM) as i32
}
unsafe extern "C" fn SHNotifyCopyFileW(op: *mut FILE_OPERATION, src: PCWSTR, dest: PCWSTR, bFailIfExists: BOOL) -> u32 {
    let mut ret: BOOL;
    let mut attribs: u32;

    _SetOperationTexts(op, src, dest);

    attribs = GetFileAttributesW(dest);
    if FileOpCallback(op, FOCE_PRECOPYITEM, src, dest, attribs, S_OK) < 0 {
        return ERROR_CANCELLED;
    }

    if IsAttrib!(attribs, FILE_ATTRIBUTE_READONLY) {
        SetFileAttributesW(dest, attribs & !FILE_ATTRIBUTE_READONLY);
    }

    if (GetFileAttributesW(dest) & FILE_ATTRIBUTE_READONLY) != 0 {
        SetFileAttributesW(dest, attribs & !FILE_ATTRIBUTE_READONLY);
        if (GetFileAttributesW(dest) & FILE_ATTRIBUTE_READONLY) != 0 {
            // empty
        }
    }

    ret = CopyFileExW(src, dest, Some(SHCopyProgressRoutine), op as *mut c_void, core::ptr::addr_of_mut!((*op).bCancelled), bFailIfExists as u32);
    let error: u32 = if ret != 0 { 0 } else { GetLastError() };
    FileOpCallback(op, FOCE_POSTCOPYITEM, src, dest, attribs, HRESULT_FROM_WIN32((error) as i32));
    if error == 0 {
        if SHIsCdRom(src) != 0 {
            attribs = GetFileAttributesW(dest);
            attribs &= !FILE_ATTRIBUTE_READONLY;
            SetFileAttributesW(dest, attribs);
        }
        SHChangeNotify((SHCNE_CREATE) as i32, SHCNF_PATHW, (dest).cast(), core::ptr::null_mut());
        return (ERROR_SUCCESS) as u32;
    }
    return CheckForError(op, error, src);
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDirectory(hWnd: HWND, path: LPCWSTR) -> i32 {
    SHCreateDirectoryExW(hWnd, path, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDirectoryExA(
    hWnd: HWND,
    path: *const u8,
    sec: *mut SECURITY_ATTRIBUTES,
) -> i32 {
    let mut w_path: PWSTR = core::ptr::null_mut();
    let mut ret_code = SHELL32_AnsiToUnicodeBuf(
        path,
        core::ptr::addr_of_mut!(w_path),
        0,
    ) as i32;
    if ret_code == 0 {
        ret_code = SHCreateDirectoryExW(hWnd, w_path, sec);
        SHELL32_FreeUnicodeBuf(w_path);
    }
    ret_code
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDirectoryExW(
    hWnd: HWND,
    path: PCWSTR,
    sec: *mut SECURITY_ATTRIBUTES,
) -> i32 {
    let mut ret: i32 = (ERROR_BAD_PATHNAME) as i32;
    if PathIsRelativeW(path) != 0 {
        SetLastError((ret) as u32);
    } else {
        ret = (SHNotifyCreateDirectoryW(path, sec)) as i32;
        if ret != ERROR_SUCCESS as i32
            && ret != (ERROR_FILE_EXISTS) as i32
            && ret != (ERROR_ALREADY_EXISTS) as i32
            && ret != (ERROR_FILENAME_EXCED_RANGE) as i32
        {
            let mut szTemp: [u16; (MAX_PATH + 1) as usize] =
                [0u16; (MAX_PATH + 1) as usize];
            lstrcpynW(
                szTemp.as_mut_ptr(),
                path,
                (MAX_PATH) as i32,
            );
            let pEnd: PCWSTR = PathAddBackslashW(szTemp.as_mut_ptr());
            let mut pSlash: PWSTR = szTemp.as_mut_ptr().add(3 as usize);
            while *pSlash != 0 {
                while *pSlash != 0 && *pSlash != 0x5Cu16 {
                    pSlash = pSlash.add(1);
                }
                if *pSlash != 0 {
                    *pSlash = 0;
                    ret = (SHNotifyCreateDirectoryW(
                        szTemp.as_mut_ptr(),
                        if (pSlash as *const u16).add(1 as usize) == pEnd {
                            sec
                        } else {
                            core::ptr::null_mut()
                        },
                    )) as i32;
                }
                *pSlash = 0x5Cu16;
                pSlash = pSlash.add(1);
            }
        }
        if ret != 0
            && !hWnd.is_null()
            && (ERROR_CANCELLED != (ret) as u32 && ERROR_ALREADY_EXISTS != (ret) as u32)
        {
            // precomp.h:63 is `#define ShellMessageBoxW ShellMessageBoxWrapW`, so the C call
            // here never reached shlwapi's ShellMessageBoxW at all. Calling the island's
            // ShellMessageBoxW instead links against a symbol shell32 does not import
            // (i386 reports it as `_ShellMessageBoxW`, cdecl-decorated and absent).
            ShellMessageBoxWrapW(
                shell32_hInstance,
                hWnd,
                MAKEINTRESOURCEW(IDS_CREATEFOLDER_DENIED),
                MAKEINTRESOURCEW(IDS_CREATEFOLDER_CAPTION),
                MB_ICONEXCLAMATION | MB_OK,
                path,
            );
            ret = (ERROR_CANCELLED) as i32;
        }
    }
    ret
}
unsafe extern "C" fn SHFindAttrW(pName: PCWSTR, fileOnly: BOOL) -> u32 {
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let b_FileMask = fileOnly != 0 && !StrPBrkW(pName, CPP_COM_WSTR_2CBECA8C1D10.as_ptr()).is_null();
    let mut dwAttr: u32 = INVALID_FILE_ATTRIBUTES;
    let hFind = FindFirstFileW(pName, &mut wfd);

    if hFind != INVALID_HANDLE_VALUE {
        loop {
            if b_FileMask && IsAttribDir!(wfd.dwFileAttributes) {
                // continue
            } else {
                dwAttr = wfd.dwFileAttributes;
                break;
            }
            if FindNextFileW(hFind, &mut wfd) == 0 {
                break;
            }
        }
        FindClose(hFind);
    }
    dwAttr
}
unsafe extern "C" fn _ConvertAtoW(
    strSrc: LPCSTR,
    pStrDest: *mut PCWSTR,
    isList: i32,
) -> u32 {
    (*pStrDest) = core::ptr::null_mut();

    if strSrc.is_null() {
        return 0;
    }

    let mut sizeA: i32 = 0;
    if isList != 0 {
        let mut tmpSrc: LPCSTR = strSrc;
        loop {
            let size: i32 =
                lstrlenA(tmpSrc) + 1;
            sizeA += size;
            tmpSrc = tmpSrc.offset(size as isize);
            if size == 1 {
                break;
            }
        }
    } else {
        sizeA = lstrlenA(strSrc) + 1;
    }

    let sizeW: i32 = MultiByteToWideChar(
        CP_ACP,
        0,
        strSrc,
        sizeA,
        core::ptr::null_mut(),
        0,
    );
    if sizeW == 0 {
        return GetLastError();
    }

    let strDest: *mut u16 = HeapAlloc(
        GetProcessHeap(),
        0,
        ((sizeW as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if strDest.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    let err: i32 = MultiByteToWideChar(
        CP_ACP,
        0,
        strSrc,
        sizeA,
        strDest,
        sizeW,
    );
    if err == 0 {
        HeapFree(
            GetProcessHeap(),
            0,
            strDest as *mut core::ffi::c_void,
        );
        return GetLastError();
    }

    (*pStrDest) = strDest;
    0
}

#[no_mangle]
pub unsafe extern "system" fn SHFileOperationA(lpFileOp: LPSHFILEOPSTRUCTA) -> i32 {
    let mut errCode: i32;
    let mut retCode: i32 = 0;
    let mut nFileOp: SHFILEOPSTRUCTW = core::mem::zeroed();

    nFileOp.hwnd = (*lpFileOp).hwnd;
    nFileOp.wFunc = (*lpFileOp).wFunc;
    nFileOp.fFlags = (*lpFileOp).fFlags;

    errCode = (_ConvertAtoW((*lpFileOp).pFrom, core::ptr::addr_of_mut!(nFileOp.pFrom), TRUE)) as i32;
    if errCode != 0 {
        loop {
            if !nFileOp.pFrom.is_null() {
                HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as PVOID);
            }
            if !nFileOp.pTo.is_null() {
                HeapFree(GetProcessHeap(), 0, nFileOp.pTo as PVOID);
            }
            if !nFileOp.lpszProgressTitle.is_null() {
                HeapFree(GetProcessHeap(), 0, nFileOp.lpszProgressTitle as PVOID);
            }

            if errCode != 0 {
                (*lpFileOp).fAnyOperationsAborted = TRUE;
                SetLastError((errCode) as u32);
                return errCode;
            }

            (*lpFileOp).hNameMappings = nFileOp.hNameMappings;
            (*lpFileOp).fAnyOperationsAborted = nFileOp.fAnyOperationsAborted;
            return retCode;
        }
    }

    if FO_DELETE != (nFileOp.wFunc & FO_MASK) {
        errCode = (_ConvertAtoW((*lpFileOp).pTo, core::ptr::addr_of_mut!(nFileOp.pTo), TRUE)) as i32;
        if errCode != 0 {
            loop {
                if !nFileOp.pFrom.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as PVOID);
                }
                if !nFileOp.pTo.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.pTo as PVOID);
                }
                if !nFileOp.lpszProgressTitle.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.lpszProgressTitle as PVOID);
                }

                if errCode != 0 {
                    (*lpFileOp).fAnyOperationsAborted = TRUE;
                    SetLastError((errCode) as u32);
                    return errCode;
                }

                (*lpFileOp).hNameMappings = nFileOp.hNameMappings;
                (*lpFileOp).fAnyOperationsAborted = nFileOp.fAnyOperationsAborted;
                return retCode;
            }
        }
    }

    if nFileOp.fFlags & (FOF_SIMPLEPROGRESS) as u16 != 0 {
        errCode = (_ConvertAtoW((*lpFileOp).lpszProgressTitle, core::ptr::addr_of_mut!(nFileOp.lpszProgressTitle), FALSE)) as i32;
        if errCode != 0 {
            loop {
                if !nFileOp.pFrom.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as PVOID);
                }
                if !nFileOp.pTo.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.pTo as PVOID);
                }
                if !nFileOp.lpszProgressTitle.is_null() {
                    HeapFree(GetProcessHeap(), 0, nFileOp.lpszProgressTitle as PVOID);
                }

                if errCode != 0 {
                    (*lpFileOp).fAnyOperationsAborted = TRUE;
                    SetLastError((errCode) as u32);
                    return errCode;
                }

                (*lpFileOp).hNameMappings = nFileOp.hNameMappings;
                (*lpFileOp).fAnyOperationsAborted = nFileOp.fAnyOperationsAborted;
                return retCode;
            }
        }
    }

    retCode = SHFileOperationW(&mut nFileOp);

    loop {
        if !nFileOp.pFrom.is_null() {
            HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as PVOID);
        }
        if !nFileOp.pTo.is_null() {
            HeapFree(GetProcessHeap(), 0, nFileOp.pTo as PVOID);
        }
        if !nFileOp.lpszProgressTitle.is_null() {
            HeapFree(GetProcessHeap(), 0, nFileOp.lpszProgressTitle as PVOID);
        }

        if errCode != 0 {
            (*lpFileOp).fAnyOperationsAborted = TRUE;
            SetLastError((errCode) as u32);
            return errCode;
        }

        (*lpFileOp).hNameMappings = nFileOp.hNameMappings;
        (*lpFileOp).fAnyOperationsAborted = nFileOp.fAnyOperationsAborted;
        return retCode;
    }
}
unsafe extern "C" fn grow_list(list: *mut FILE_LIST) {
    let newx = HeapReAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*list).feFiles as *mut core::ffi::c_void,
        (((*list).num_alloc * 2) as usize * core::mem::size_of::<FILE_ENTRY>()) as SIZE_T,
    ) as *mut FILE_ENTRY;
    (*list).feFiles = newx;
    (*list).num_alloc *= 2;
}
unsafe extern "C" fn add_file_to_entry(feFile: *mut FILE_ENTRY, szFile: PCWSTR) {
    let mut dwLen = lstrlenW(szFile) + 1;
    let mut ptr: PCWSTR;
    let ptr2: PCWSTR;

    (*feFile).szFullPath = HeapAlloc(
        GetProcessHeap(),
        0,
        ((dwLen as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;
    lstrcpyW((*feFile).szFullPath, szFile);

    ptr = StrRChrW(szFile, core::ptr::null(), '\\' as u16);
    ptr2 = StrRChrW(szFile, core::ptr::null(), '/' as u16);
    if ptr.is_null() || (ptr as usize) < (ptr2 as usize) {
        ptr = ptr2;
    }
    if !ptr.is_null() {
        let char_offset = (ptr as usize - szFile as usize) / core::mem::size_of::<u16>();
        dwLen = ((char_offset as u32) + 1) as i32;
        (*feFile).szDirectory = HeapAlloc(
            GetProcessHeap(),
            0,
            ((dwLen as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        lstrcpynW((*feFile).szDirectory, szFile, dwLen);

        let path_char_len = lstrlenW((*feFile).szFullPath) as usize;
        let filename_char_len = path_char_len - (dwLen as usize) + 1 as usize;
        (*feFile).szFilename = HeapAlloc(
            GetProcessHeap(),
            0,
            (filename_char_len * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        lstrcpyW(
            (*feFile).szFilename,
            ptr.offset(1 as isize),
        );
    }
    (*feFile).bFromWildcard = FALSE;
}
unsafe extern "C" fn wildcard_to_file(szWildCard: PCWSTR, szFileName: PCWSTR) -> PWSTR {
    let ptr: PCWSTR = StrRChrW(szWildCard, core::ptr::null_mut::<u16>(), b'\\' as u16);
    let ptr2: PCWSTR = StrRChrW(szWildCard, core::ptr::null_mut::<u16>(), b'/' as u16);
    let mut ptr = ptr;
    if ptr.is_null() || ptr < ptr2 {
        ptr = ptr2;
    }
    // C `ptr - szWildCard + 1` counts WCHARs. Subtracting addresses counted bytes, so
    // lstrcpynW copied past the directory into the wildcard's file part and the result
    // (sized from it) could overflow (found by scan_ptr_byte_diff).
    let dwDirLen: u32 = (ptr.offset_from(szWildCard) + 1) as u32;

    let dwFullLen: u32 = dwDirLen + lstrlenW(szFileName) as u32 + 1;
    let szFullPath: PWSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        ((dwFullLen as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;

    lstrcpynW(szFullPath, szWildCard, (dwDirLen + 1 as u32) as i32);
    lstrcatW(szFullPath, szFileName);

    szFullPath
}
unsafe extern "C" fn parse_wildcard_files(flList: *mut FILE_LIST, szFile: PCWSTR, pdwListIndex: *mut u32) {
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFile: HANDLE = FindFirstFileW(szFile, &mut wfd);
    let mut file: *mut FILE_ENTRY;
    let mut szFullPath: PWSTR;
    let mut res: BOOL;

    if hFile == INVALID_HANDLE_VALUE {
        return;
    }

    res = TRUE;
    while res != 0 {
        if IsDotDir(wfd.cFileName.as_ptr()) == 0 {
            if *pdwListIndex >= (*flList).num_alloc {
                grow_list(flList);
            }

            szFullPath = wildcard_to_file(szFile, wfd.cFileName.as_ptr());
            file = (*flList).feFiles.offset(*pdwListIndex as isize);
            *pdwListIndex = (*pdwListIndex).wrapping_add(1);
            add_file_to_entry(file, szFullPath);
            (*file).bFromWildcard = TRUE;
            (*file).attributes = wfd.dwFileAttributes;

            if IsAttribDir!((*file).attributes) {
                (*flList).bAnyDirectories = TRUE;
            }

            HeapFree(GetProcessHeap(), 0, (szFullPath).cast());
        }

        res = FindNextFileW(hFile, &mut wfd);
    }

    FindClose(hFile);
}
unsafe extern "C" fn parse_file_list(flList: *mut FILE_LIST, szFiles: PCWSTR) -> i32 {
    let mut ptr = szFiles;
    let mut szCurFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut i: u32 = 0;

    if szFiles.is_null() {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    (*flList).bAnyFromWildcard = FALSE;
    (*flList).bAnyDirectories = FALSE;
    (*flList).bAnyDontExist = FALSE;
    (*flList).num_alloc = 32;
    (*flList).dwNumFiles = 0;

    if *szFiles.offset(0) == 0 {
        return (ERROR_ACCESS_DENIED) as i32;
    }

    (*flList).feFiles = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
                                ((*flList).num_alloc * core::mem::size_of::<FILE_ENTRY>() as u32) as SIZE_T) as *mut FILE_ENTRY;

    while *ptr != 0 {
        if i >= (*flList).num_alloc {
            grow_list(flList);
        }

        if PathIsRelativeW(ptr) != 0 {
            GetCurrentDirectoryW((MAX_PATH) as u32, szCurFile.as_mut_ptr());
            PathCombineW(szCurFile.as_mut_ptr(), szCurFile.as_mut_ptr(), ptr);
            (*flList).feFiles.offset(i as isize).write_bytes(0, 1);
            (*(*flList).feFiles.offset(i as isize)).bFromRelative = TRUE;
        } else {
            lstrcpyW(szCurFile.as_mut_ptr(), ptr);
            (*flList).feFiles.offset(i as isize).write_bytes(0, 1);
            (*(*flList).feFiles.offset(i as isize)).bFromRelative = FALSE;
        }

        if !StrPBrkW(szCurFile.as_ptr(), CPP_COM_WSTR_2CBECA8C1D10.as_ptr()).is_null() {
            parse_wildcard_files(flList, szCurFile.as_ptr(), core::ptr::addr_of_mut!(i));
            (*flList).bAnyFromWildcard = TRUE;
            i = i.wrapping_sub(1);
        } else {
            let file = (*flList).feFiles.offset(i as isize);
            add_file_to_entry(file, szCurFile.as_ptr());
            (*file).attributes = GetFileAttributesW((*file).szFullPath);
            (*file).bExists = ((*file).attributes != INVALID_FILE_ATTRIBUTES) as i32;

            if (*file).bExists == 0 {
                (*flList).bAnyDontExist = TRUE;
            }

            if IsAttribDir!((*file).attributes) {
                (*flList).bAnyDirectories = TRUE;
            }
        }

        ptr = ptr.offset(lstrlenW(ptr) as isize + 1);
        i = i.wrapping_add(1);
    }
    (*flList).dwNumFiles = i;

    S_OK
}
unsafe extern "C" fn destroy_file_list(flList: *mut FILE_LIST) {
    if flList.is_null() || (*flList).feFiles.is_null() {
        return;
    }

    for i in 0..(*flList).dwNumFiles {
        let entry = (*flList).feFiles.add((i) as usize);
        HeapFree(
            GetProcessHeap(),
            0,
            ((*entry).szDirectory).cast()
        );
        HeapFree(
            GetProcessHeap(),
            0,
            ((*entry).szFilename).cast()
        );
        HeapFree(
            GetProcessHeap(),
            0,
            ((*entry).szFullPath).cast()
        );
    }

    HeapFree(
        GetProcessHeap(),
        0,
        ((*flList).feFiles).cast()
    );
}
/// `static` in the C++ and returns a `CStringW` by value, so it stays a plain `unsafe fn`:
/// giving it `extern "C"` would put a non-FFI-safe return type on a C ABI for no caller.
unsafe fn try_find_new_name(szDestPath: *const u16) -> CStringW {
    let mut mask = CStringW::from_ptr(szDestPath);
    let ext = CStringW::from_ptr(PathFindExtensionW(szDestPath));
    if !ext.IsEmpty() {
        mask = mask.Left(mask.GetLength() - ext.GetLength());
    }
    // `mask += L" (%d)" + ext` — ATL's `operator+(PCXSTR, const CStringT&)` builds the
    // concatenation first; appending the two pieces in order gives the same string.
    mask.Append(CPP_COM_WSTR_926483D22C70.as_ptr());
    mask.Append(ext.as_ptr());
    let mut newName = CStringW::new();
    for i in 1 .. NEW_FILENAME_ON_COPY_TRIES {
        newName.FormatInt(mask.as_ptr(), i);
        // `!PathFileExistsW(...)` — the C++ negates a BOOL. Rust `!` on an i32 is bitwise
        // NOT, so this MUST be an explicit `== 0` or every path reads as "does not exist".
        if PathFileExistsW(newName.as_ptr()) == 0 {
            return newName;
        }
    }
    return CStringW::new();
}
unsafe extern "C" fn copy_dir_to_dir(op: *mut FILE_OPERATION, feFrom: *const FILE_ENTRY, szDestPath: PCWSTR) {
    let mut szFrom: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut szTo: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut flFromNew: FILE_LIST = core::mem::zeroed();
    let mut flToNew: FILE_LIST = core::mem::zeroed();

    let filename = (*feFrom).szFilename;
    if !filename.is_null() && IsDotDir(filename as *const u16) != 0 {
        return;
    }

    if PathFileExistsW(szDestPath) != 0 {
        PathCombineW(szTo.as_mut_ptr(), szDestPath, (*feFrom).szFilename as *const u16);
    } else {
        lstrcpyW(szTo.as_mut_ptr(), szDestPath);
    }

    if PathFileExistsW(szTo.as_ptr()) != 0 {
        if ((*(*op).req).fFlags & (FOF_RENAMEONCOLLISION) as u16) != 0 {
            let newPath: CStringW = try_find_new_name(szTo.as_ptr());
            if !newPath.IsEmpty() {
                StringCchCopyW(szTo.as_mut_ptr(), szTo.len(), newPath.as_ptr());
            }
        } else if ((*(*op).req).fFlags & (FOF_NOCONFIRMATION) as u16) == 0 {
            if SHELL_ConfirmDialogW((*op).hWndOwner, (ASK_OVERWRITE_FOLDER) as i32, (*feFrom).szFilename as *const u16, op) == 0 {
                if (*op).bManyItems == 0 {
                    (*op).bCancelled = TRUE;
                }
                return;
            }
        }
    }

    {
        let len = lstrlenW(szTo.as_ptr()) as usize;
        szTo[len + 1 as usize] = 0u16;
    }

    SHNotifyCreateDirectoryW(szTo.as_ptr(), core::ptr::null_mut());

    PathCombineW(szFrom.as_mut_ptr(), (*feFrom).szFullPath as *const u16, CPP_COM_WSTR_955300DD21E1.as_ptr());
    {
        let len = lstrlenW(szFrom.as_ptr()) as usize;
        szFrom[len + 1 as usize] = 0u16;
    }

    ZeroMemory(&raw mut flFromNew as *mut c_void, core::mem::size_of::<FILE_LIST>());
    ZeroMemory(&raw mut flToNew as *mut c_void, core::mem::size_of::<FILE_LIST>());
    parse_file_list(&raw mut flFromNew, szFrom.as_ptr());
    parse_file_list(&raw mut flToNew, szTo.as_ptr());

    copy_files(op, FALSE, &flFromNew, &raw mut flToNew);

    destroy_file_list(&raw mut flFromNew);
    destroy_file_list(&raw mut flToNew);
}
unsafe extern "C" fn copy_file_to_file(op: *mut FILE_OPERATION, szFrom: PCWSTR, szTo: PCWSTR) -> BOOL {
    if PathFileExistsW(szTo) != 0 {
        if ((*(*op).req).fFlags & (FOF_RENAMEONCOLLISION) as u16) != 0 {
            let newPath = try_find_new_name(szTo);
            if !newPath.IsEmpty() {
                return if SHNotifyCopyFileW(op, szFrom, newPath.as_ptr(), FALSE) == 0 { 1 } else { 0 };
            }
        } else if ((*(*op).req).fFlags & (FOF_NOCONFIRMATION) as u16) == 0 {
            if SHELL_ConfirmDialogW((*op).hWndOwner, (ASK_OVERWRITE_FILE) as i32, PathFindFileNameW(szTo), op) == 0 {
                return FALSE;
            }
        }
    }
    return if SHNotifyCopyFileW(op, szFrom, szTo, FALSE) == 0 { 1 } else { 0 };
}
unsafe extern "C" fn copy_to_dir(op: *mut FILE_OPERATION, feFrom: *const FILE_ENTRY, feTo: *const FILE_ENTRY) {
    if PathFileExistsW((*feTo).szFullPath) == 0 {
        SHNotifyCreateDirectoryW((*feTo).szFullPath, core::ptr::null_mut());
    }

    if IsAttribFile!((*feFrom).attributes) {
        let mut szDestPath = [0u16; MAX_PATH as usize];
        PathCombineW(szDestPath.as_mut_ptr(), (*feTo).szFullPath, (*feFrom).szFilename);
        copy_file_to_file(op, (*feFrom).szFullPath, szDestPath.as_ptr());
    } else if !((*(*op).req).fFlags & (FOF_FILESONLY) as u16 != 0 && (*feFrom).bFromWildcard != 0) {
        copy_dir_to_dir(op, feFrom, (*feTo).szFullPath);
    }
}
unsafe extern "C" fn create_dest_dirs(szDestDir: PCWSTR) {
    let mut dir = [0u16; MAX_PATH as usize];
    let mut ptr = StrChrW(szDestDir, '\\' as u16);
    while ptr.is_null() == false && {
        let new_ptr = StrChrW(ptr.add(1), '\\' as u16);
        ptr = new_ptr;
        !new_ptr.is_null()
    } {
        lstrcpynW(
            dir.as_mut_ptr(),
            szDestDir,
            ((ptr.offset_from(szDestDir) as usize) + 1) as i32,
        );
        if PathFileExistsW(dir.as_ptr()) == 0 {
            SHNotifyCreateDirectoryW(dir.as_ptr(), core::ptr::null_mut());
        }
    }
    if PathFileExistsW(szDestDir) == 0 {
        SHNotifyCreateDirectoryW(szDestDir, core::ptr::null_mut());
    }
}
unsafe extern "C" fn copy_files(op: *mut FILE_OPERATION, multiDest: BOOL, flFrom: *const FILE_LIST, flTo: *mut FILE_LIST) -> HRESULT {
    let mut i: u32;
    let mut entryToCopy: *const FILE_ENTRY;
    let mut fileDest: *const FILE_ENTRY = unsafe { (*flTo).feFiles.add(0) as *const FILE_ENTRY };

    if unsafe { (*flFrom).bAnyDontExist != 0 } {
        return (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32;
    }

    if unsafe { (*flTo).dwNumFiles == 0 } {
        let mut curdir: [u16; (MAX_PATH + 1) as usize] = [0u16; (MAX_PATH + 1) as usize];
        unsafe {
            GetCurrentDirectoryW((MAX_PATH) as u32, curdir.as_mut_ptr());
        }
        let len = unsafe { lstrlenW(curdir.as_ptr()) as usize };
        curdir[len + (1 as usize)] = 0 as u16;
        unsafe {
            destroy_file_list(flTo);
            ZeroMemory(flTo as *mut _, core::mem::size_of::<FILE_LIST>());
            parse_file_list(flTo, curdir.as_ptr());
        }
        fileDest = unsafe { (*flTo).feFiles.add(0 as usize) as *const FILE_ENTRY };
    }

    if multiDest != 0 {
        if unsafe { (*flFrom).bAnyFromWildcard != 0 } {
            return (ERROR_CANCELLED) as i32;
        }

        if unsafe { (*flFrom).dwNumFiles != (*flTo).dwNumFiles } {
            if unsafe { (*flFrom).dwNumFiles != 1 && !IsAttribDir!((*fileDest).attributes) } {
                return (ERROR_CANCELLED) as i32;
            }

            i = 1;
            while i < unsafe { (*flTo).dwNumFiles } {
                unsafe {
                    HeapFree(GetProcessHeap(), 0, ((*(*flTo).feFiles.add(i as usize)).szDirectory).cast());
                    HeapFree(GetProcessHeap(), 0, ((*(*flTo).feFiles.add(i as usize)).szFilename).cast());
                    HeapFree(GetProcessHeap(), 0, ((*(*flTo).feFiles.add(i as usize)).szFullPath).cast());
                }
                i += 1;
            }

            unsafe {
                (*flTo).dwNumFiles = 1;
            }
        } else if unsafe { IsAttribDir!((*fileDest).attributes) } {
            i = 1;
            while i < unsafe { (*flTo).dwNumFiles } {
                if unsafe { !IsAttribDir!((*(*flTo).feFiles.add(i as usize)).attributes) ||
                      !IsAttribDir!((*(*flFrom).feFiles.add(i as usize)).attributes) } {
                    return (ERROR_CANCELLED) as i32;
                }
                i += 1;
            }
        }
    } else if unsafe { (*flFrom).dwNumFiles != 1 } {
        if unsafe { (*flTo).dwNumFiles != 1 && !IsAttribDir!((*fileDest).attributes) } {
            return (ERROR_CANCELLED) as i32;
        }

        if unsafe { PathFileExistsW((*fileDest).szFullPath) != 0 &&
              IsAttribFile!((*fileDest).attributes) } {
            return (ERROR_CANCELLED) as i32;
        }

        if unsafe { (*flTo).dwNumFiles == 1 && (*fileDest).bFromRelative != 0 &&
              PathFileExistsW((*fileDest).szFullPath) == 0 } {
            return ERROR_CANCELLED as HRESULT;
        }
    }

    i = 0;
    while i < unsafe { (*flFrom).dwNumFiles } {
        entryToCopy = unsafe { (*flFrom).feFiles.add(i as usize) as *const FILE_ENTRY };

        if multiDest != 0 && unsafe { (*flTo).dwNumFiles > 1 } {
            fileDest = unsafe { (*flTo).feFiles.add(i as usize) as *const FILE_ENTRY };
        }

        if unsafe { IsAttribDir!((*entryToCopy).attributes) &&
              lstrcmpiW((*entryToCopy).szFullPath, (*fileDest).szDirectory) == 0 } {
            return ERROR_SUCCESS as HRESULT;
        }

        unsafe {
            create_dest_dirs((*fileDest).szDirectory);
        }

        if unsafe { lstrcmpiW((*entryToCopy).szFullPath, (*fileDest).szFullPath) == 0 } {
            if unsafe { IsAttribFile!((*entryToCopy).attributes) } {
                return ERROR_NO_MORE_SEARCH_HANDLES as HRESULT;
            } else {
                return ERROR_SUCCESS as HRESULT;
            }
        }

        if (unsafe { (*flFrom).dwNumFiles > 1 && (*flTo).dwNumFiles == 1 }) ||
            unsafe { IsAttribDir!((*fileDest).attributes) } {
            unsafe {
                copy_to_dir(op, entryToCopy, fileDest);
            }
        } else if unsafe { IsAttribDir!((*entryToCopy).attributes) } {
            unsafe {
                copy_dir_to_dir(op, entryToCopy, (*fileDest).szFullPath);
            }
        } else {
            if unsafe { copy_file_to_file(op, (*entryToCopy).szFullPath, (*fileDest).szFullPath) == 0 } {
                unsafe {
                    (*(*op).req).fAnyOperationsAborted = TRUE;
                }
                return ERROR_CANCELLED as HRESULT;
            }
        }

        unsafe {
            if !(*op).progress.is_null() {
                (*op).bCancelled |= ((*(*(*op).progress).lpVtbl).HasUserCancelled)((*op).progress);
            }
        }

        if unsafe { (*op).bCancelled != 0 } {
            return ERROR_CANCELLED as HRESULT;
        }

        i += 1;
    }

    ERROR_SUCCESS as HRESULT
}
unsafe extern "C" fn confirm_delete_list(
    hWnd: HWND,
    fFlags: DWORD,
    fTrash: BOOL,
    flFrom: *const FILE_LIST,
) -> BOOL {
    if (*flFrom).dwNumFiles > 1 {
        let mut tmp: [u16; 8] = [0u16; 8];
        wnsprintfW(
            tmp.as_mut_ptr(),
            tmp.len() as i32,
            CPP_COM_WSTR_A9EDBA2FC474.as_ptr(),
            (*flFrom).dwNumFiles,
        );
        return SHELL_ConfirmDialogW(
            hWnd,
            if fTrash != 0 { (ASK_TRASH_MULTIPLE_ITEM) as i32 } else { (ASK_DELETE_MULTIPLE_ITEM) as i32 },
            tmp.as_ptr(),
            core::ptr::null_mut(),
        );
    } else {
        let fileEntry: *const FILE_ENTRY = (*flFrom).feFiles;
        if IsAttribFile!((*fileEntry).attributes) {
            return SHELL_ConfirmDialogW(
                hWnd,
                if fTrash != 0 { (ASK_TRASH_FILE) as i32 } else { (ASK_DELETE_FILE) as i32 },
                (*fileEntry).szFullPath,
                core::ptr::null_mut(),
            );
        } else if !((fFlags & FOF_FILESONLY) != 0 && (*fileEntry).bFromWildcard != 0) {
            return SHELL_ConfirmDialogW(
                hWnd,
                if fTrash != 0 { (ASK_TRASH_FOLDER) as i32 } else { (ASK_DELETE_FOLDER) as i32 },
                (*fileEntry).szFullPath,
                core::ptr::null_mut(),
            );
        }
    }
    TRUE
}
unsafe extern "C" fn delete_files(op: *mut FILE_OPERATION, flFrom: *const FILE_LIST) -> HRESULT {
    let mut bPathExists: BOOL;
    let bTrash: BOOL;

    if (*flFrom).dwNumFiles == 0 {
        return ERROR_SUCCESS as HRESULT;
    }

    let first_fe = (*flFrom).feFiles.offset(0 as isize);
    bTrash = (((*(*op).req).fFlags & (FOF_ALLOWUNDO) as u16) != 0
        && TRASH_CanTrashFile((*first_fe).szFullPath) != 0) as BOOL;

    // C has this as `BOOL confirm`; a Rust `bool` is the faithful shape here because every
    // use is a condition, and it keeps the assignment below from being a silent int.
    let mut confirm = ((*(*op).req).fFlags & (FOF_NOCONFIRMATION) as u16) == 0;
    if bTrash != 0 && SHELL_GetSetting!(SSF_NOCONFIRMRECYCLE, fNoConfirmRecycle) != 0 {
        confirm = false;
    }
    if confirm || (bTrash == 0 && ((*(*op).req).fFlags & (FOF_WANTNUKEWARNING) as u16) != 0) {
        if confirm_delete_list((*op).hWndOwner, ((*(*op).req).fFlags) as u32, bTrash, flFrom) == 0 {
            (*(*op).req).fAnyOperationsAborted = TRUE;
            return 0;
        }
    }

    for i in 0..(*flFrom).dwNumFiles {
        let file_entry = (*flFrom).feFiles.offset(i as isize);
        if (*file_entry).attributes == INVALID_FILE_ATTRIBUTES {
            return (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32;
        }
    }

    for i in 0..(*flFrom).dwNumFiles {
        bPathExists = TRUE;
        let file_entry = (*flFrom).feFiles.offset(i as isize);
        if !IsAttribFile!((*file_entry).attributes)
            && ((*(*op).req).fFlags & (FOF_FILESONLY) as u16) != 0
            && (*file_entry).bFromWildcard != 0
        {
            continue;
        }

        if bTrash != 0 {
            if TRASH_TrashFile((*file_entry).szFullPath) != 0 {
                let event = if IsAttribFile!((*file_entry).attributes) {
                    SHCNE_DELETE
                } else {
                    SHCNE_RMDIR
                };
                SHChangeNotify(
                    (event) as i32,
                    SHCNF_PATHW,
                    (*file_entry).szFullPath as *const c_void,
                    core::ptr::null_mut(),
                );
                continue;
            }

            let bDelete;
            if ((*(*op).req).fFlags & (FOF_NOCONFIRMATION) as u16) == 0
                || ((*(*op).req).fFlags & (FOF_WANTNUKEWARNING) as u16) != 0
            {
                bDelete = SHELL_ConfirmDialogW(
                    (*op).hWndOwner,
                    (ASK_CANT_TRASH_ITEM) as i32,
                    (*file_entry).szFullPath,
                    core::ptr::null_mut(),
                );
            } else {
                bDelete = TRUE;
            }
            if bDelete == 0 {
                (*(*op).req).fAnyOperationsAborted = TRUE;
                break;
            }
        }

        if IsAttribFile!((*file_entry).attributes) {
            let res = SHNotifyDeleteFileW(op, (*file_entry).szFullPath);
            bPathExists = (ERROR_SUCCESS == res) as BOOL;
        } else {
            bPathExists = SHELL_DeleteDirectoryW(op, (*file_entry).szFullPath, FALSE);
        }

        if bPathExists == 0 {
            let err = GetLastError();
            if ERROR_FILE_NOT_FOUND == err {
                return (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32;
            } else {
                return (err) as i32;
            }
        }

        if !(*op).progress.is_null() {
            (*op).bCancelled |= ((*(*(*op).progress).lpVtbl).HasUserCancelled)((*op).progress);
        }
        if (*op).bCancelled != 0 {
            return (ERROR_CANCELLED) as i32;
        }
    }

    ERROR_SUCCESS as HRESULT
}
unsafe extern "C" fn move_dir_to_dir(op: *mut FILE_OPERATION, feFrom: *const FILE_ENTRY, szDestPath: PCWSTR) {
    let mut szFrom: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut szTo: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    if !(*feFrom).szFilename.is_null() && IsDotDir((*feFrom).szFilename) != 0 {
        return;
    }

    let attrib: u32 = FILE_ATTRIBUTE_DIRECTORY;
    if FileOpCallback(op, FOCE_PREMOVEITEM, (*feFrom).szFullPath, szDestPath, attrib, S_OK) < 0 {
        return;
    }

    SHNotifyCreateDirectoryW(szDestPath as *mut u16, core::ptr::null_mut());

    PathCombineW(szFrom.as_mut_ptr(), (*feFrom).szFullPath, CPP_COM_WSTR_955300DD21E1.as_ptr());
    *szFrom.as_mut_ptr().add(lstrlenW(szFrom.as_ptr()) as usize + 1 as usize) = 0;

    lstrcpyW(szTo.as_mut_ptr(), szDestPath);
    *szTo.as_mut_ptr().add(lstrlenW(szDestPath) as usize + 1 as usize) = 0;

    let mut flFromNew: FILE_LIST = core::mem::MaybeUninit::zeroed().assume_init();
    let mut flToNew: FILE_LIST = core::mem::MaybeUninit::zeroed().assume_init();
    ZeroMemory(core::ptr::addr_of_mut!(flFromNew) as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_LIST>());
    ZeroMemory(core::ptr::addr_of_mut!(flToNew) as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_LIST>());

    parse_file_list(core::ptr::addr_of_mut!(flFromNew), szFrom.as_ptr());
    parse_file_list(core::ptr::addr_of_mut!(flToNew), szTo.as_ptr());

    move_files(op, FALSE, &raw const flFromNew, &raw const flToNew);

    destroy_file_list(core::ptr::addr_of_mut!(flFromNew));
    destroy_file_list(core::ptr::addr_of_mut!(flToNew));

    let mut success: BOOL = FALSE;
    if PathIsDirectoryEmptyW((*feFrom).szFullPath) != 0 {
        success = Win32RemoveDirectoryW((*feFrom).szFullPath) as BOOL;
    }
    FileOpCallback(op, FOCE_POSTMOVEITEM, (*feFrom).szFullPath, szDestPath, attrib, if success != 0 { S_OK } else { E_FAIL });
}
unsafe extern "C" fn move_file_to_file(op: *mut FILE_OPERATION, szFrom: PCWSTR, szTo: PCWSTR) -> i32 {
    if PathFileExistsW(szTo) != 0 {
        if (*(*op).req).fFlags & (FOF_RENAMEONCOLLISION) as u16 != 0 {
            let newPath = try_find_new_name(szTo);
            if !newPath.IsEmpty() {
                return (SHNotifyMoveFileW(op, szFrom, newPath.as_ptr(), FALSE) == 0) as i32;
            }
        } else if (*(*op).req).fFlags & (FOF_NOCONFIRMATION) as u16 == 0 {
            if SHELL_ConfirmDialogW((*op).hWndOwner, (ASK_OVERWRITE_FILE) as i32, PathFindFileNameW(szTo), op) == 0 {
                return FALSE;
            }
        }
    }

    (SHNotifyMoveFileW(op, szFrom, szTo, FALSE) == 0) as i32
}
unsafe extern "C" fn move_to_dir(op: *mut FILE_OPERATION, feFrom: *const FILE_ENTRY, feTo: *const FILE_ENTRY) {
    if (*feFrom).attributes == INVALID_FILE_ATTRIBUTES {
        return;
    }

    if PathFileExistsW((*feTo).szFullPath) == 0 {
        SHNotifyCreateDirectoryW((*feTo).szFullPath, core::ptr::null_mut());
    }

    let mut szDestPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    PathCombineW(szDestPath.as_mut_ptr(), (*feTo).szFullPath, (*feFrom).szFilename);

    if IsAttribFile!((*feFrom).attributes) {
        move_file_to_file(op, (*feFrom).szFullPath, szDestPath.as_ptr());
    } else if !((((*(*op).req).fFlags & (FOF_FILESONLY) as u16) != 0) && ((*feFrom).bFromWildcard != 0)) {
        move_dir_to_dir(op, feFrom, szDestPath.as_ptr());
    }
}
unsafe extern "C" fn move_files(
    op: *mut FILE_OPERATION,
    multiDest: BOOL,
    flFrom: *const FILE_LIST,
    flTo: *const FILE_LIST,
) -> u32 {
    let mut mismatched: i32 = 0;

    if (*flFrom).dwNumFiles == 0 {
        return (ERROR_SUCCESS) as u32;
    }

    if (*flTo).dwNumFiles == 0 {
        return ERROR_FILE_NOT_FOUND;
    }

    if (*flFrom).bAnyDontExist != 0 {
        return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
    }

    if multiDest == 0
        && (*flTo).dwNumFiles > 1
        && (*flFrom).dwNumFiles > 1
    {
        return ERROR_CANCELLED;
    }

    if multiDest == 0
        && (*flFrom).bAnyDirectories == 0
        && (*flFrom).dwNumFiles > (*flTo).dwNumFiles
        && !((*flTo).bAnyDirectories != 0 && (*flTo).dwNumFiles == 1)
    {
        return ERROR_CANCELLED;
    }

    if PathFileExistsW((*(*flTo).feFiles.offset(0 as isize)).szDirectory)
        == 0
    {
        return ERROR_CANCELLED;
    }

    if multiDest != 0 {
        mismatched = (*flFrom).dwNumFiles as i32 - (*flTo).dwNumFiles as i32;
    }

    let mut fileDest: *mut FILE_ENTRY = (*flTo).feFiles.offset(0);
    let mut i: u32 = 0;
    while i < (*flFrom).dwNumFiles {
        let entryToMove: *const FILE_ENTRY = (*flFrom).feFiles.offset(i as isize);

        if PathFileExistsW((*fileDest).szDirectory) == 0 {
            return ERROR_CANCELLED;
        }

        if multiDest != 0 {
            if i >= (*flTo).dwNumFiles {
                break;
            }
            fileDest = (*flTo).feFiles.offset(i as isize);
            if mismatched != 0 && (*fileDest).bExists == 0 {
                create_dest_dirs((*(*flTo).feFiles.offset(i as isize)).szFullPath);
                (*fileDest).bExists = TRUE;
                (*fileDest).attributes = FILE_ATTRIBUTE_DIRECTORY;
            }
        }

        if ((*flFrom).dwNumFiles > 1 && (*flTo).dwNumFiles == 1)
            || IsAttribDir!((*fileDest).attributes)
        {
            move_to_dir(op, entryToMove, fileDest);
        } else if IsAttribDir!((*entryToMove).attributes) {
            move_dir_to_dir(op, entryToMove, (*fileDest).szFullPath);
        } else {
            if move_file_to_file(
                op,
                (*entryToMove).szFullPath,
                (*fileDest).szFullPath,
            ) == 0
            {
                (*(*op).req).fAnyOperationsAborted = TRUE;
                return ERROR_CANCELLED;
            }
        }

        if !(*op).progress.is_null() {
            let progress = (*op).progress;
            (*op).bCancelled |= ((*(*progress).lpVtbl).HasUserCancelled)(progress);
        }
        if (*op).bCancelled != 0 {
            return ERROR_CANCELLED;
        }

        i += 1;
    }

    if mismatched > 0 {
        return if (*flFrom).bAnyDirectories != 0 {
            DE_DESTSAMETREE
        } else {
            DE_SAMEFILE
        };
    }

    (ERROR_SUCCESS) as u32
}
unsafe extern "C" fn rename_files(
    op: *mut FILE_OPERATION,
    flFrom: *const FILE_LIST,
    flTo: *const FILE_LIST,
) -> HRESULT {
    if (*flFrom).dwNumFiles != 1 {
        return (ERROR_GEN_FAILURE) as i32;
    }

    if (*flTo).dwNumFiles != 1 {
        return (ERROR_CANCELLED) as i32;
    }

    let feFrom = (*flFrom).feFiles.add(0);
    let feTo = (*flTo).feFiles.add(0);

    if (*feFrom).bExists == 0 {
        return (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32;
    }

    if (*feTo).bExists != 0 {
        return (ERROR_ALREADY_EXISTS) as i32;
    }

    SHNotifyMoveFileW(
        op,
        (*feFrom).szFullPath,
        (*feTo).szFullPath,
        IsAttribDir!((*feFrom).attributes) as BOOL,
    ) as HRESULT
}
unsafe extern "C" fn check_flags(fFlags: FILEOP_FLAGS) {
    let wUnsupportedFlags: u16 = (FOF_NO_CONNECTED_ELEMENTS
        | FOF_NOCOPYSECURITYATTRIBS
        | FOF_NORECURSEREPARSE
        | FOF_WANTMAPPINGHANDLE) as u16;
    if (fFlags as u16 & wUnsupportedFlags) != 0 {
        FIXME!(
            "Unsupported flags: %04x\n",
            fFlags as core::ffi::c_uint,
        );
    }
}
unsafe extern "C" fn validate_operation(op: &mut FILE_OPERATION, flFrom: *const FILE_LIST, flTo: *const FILE_LIST) -> DWORD {
    let mut i: DWORD;
    let mut k: DWORD;
    let dwNumDest: DWORD;
    let mut szFrom: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szTo: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    // C++ RAII locals — see CheckForError: an uninitialised binding runs no ctor and no dtor.
    let mut strTitle = CStringW::new();
    let mut strText = CStringW::new();
    let mut feFrom: *const FILE_ENTRY;
    let mut feTo: *const FILE_ENTRY;
    let lpFileOp = (&*op.req);
    let wFunc = (*lpFileOp).wFunc;

    dwNumDest = (*flTo).dwNumFiles;

    if wFunc != FO_COPY && wFunc != FO_MOVE {
        return (ERROR_SUCCESS) as u32;
    }

    k = 0;
    while k < dwNumDest {
        feTo = (*flTo).feFiles.add(k as usize);
        i = 0;
        while i < (*flFrom).dwNumFiles {
            feFrom = (*flFrom).feFiles.add(i as usize);
            StringCbCopyW(szFrom.as_mut_ptr(), core::mem::size_of_val(&szFrom), (*feFrom).szFullPath);
            StringCbCopyW(szTo.as_mut_ptr(), core::mem::size_of_val(&szTo), (*feTo).szFullPath);
            if IsAttribDir!((*feTo).attributes) {
                PathAppendW(szTo.as_mut_ptr(), (*feFrom).szFilename);
            }

            if lstrcmpiW(szFrom.as_ptr(), szTo.as_ptr()) == 0 &&
                (wFunc == FO_MOVE || ((*lpFileOp).fFlags & (FOF_RENAMEONCOLLISION) as u16) == 0) {
                if CanShowFileOpErrorUI(lpFileOp) != 0 {
                    if wFunc == FO_MOVE {
                        strTitle.LoadStringW(IDS_MOVEERRORTITLE);
                        if IsAttribDir!((*feFrom).attributes) {
                            strText.FormatResStr(IDS_MOVEERRORSAMEFOLDER, GET_FILENAME!(feFrom));
                        } else {
                            strText.FormatResStr(IDS_MOVEERRORSAME, GET_FILENAME!(feFrom));
                        }
                    } else {
                        strTitle.LoadStringW(IDS_COPYERRORTITLE);
                        strText.FormatResStr(IDS_COPYERRORSAME, GET_FILENAME!(feFrom));
                        return (ERROR_SUCCESS) as u32;
                    }
                    MessageBoxW((*op).hWndOwner, strText.as_ptr(), strTitle.as_ptr(), MB_ICONERROR);
                    (*op).bHasDisplayedError += 1;
                    return DE_SAMEFILE;
                }
                return DE_OPCANCELLED;
            }

            if IsAttribDir!((*feFrom).attributes) {
                let cchFrom = PathAddBackslashW(szFrom.as_mut_ptr()).offset_from(szFrom.as_ptr()) as usize;
                let cchTo = PathAddBackslashW(szTo.as_mut_ptr()).offset_from(szTo.as_ptr()) as usize;
                if cchFrom < cchTo {
                    let ch = szTo[cchFrom];
                    szTo[cchFrom] = 0;
                    let compare = lstrcmpiW(szFrom.as_ptr(), szTo.as_ptr());
                    szTo[cchFrom] = ch;

                    if compare == 0 {
                        if CanShowFileOpErrorUI(lpFileOp) != 0 {
                            if wFunc == FO_MOVE {
                                strTitle.LoadStringW(IDS_MOVEERRORTITLE);
                                strText.FormatResStr(IDS_MOVEERRORSUBFOLDER, GET_FILENAME!(feFrom));
                            } else {
                                strTitle.LoadStringW(IDS_COPYERRORTITLE);
                                strText.FormatResStr(IDS_COPYERRORSUBFOLDER, GET_FILENAME!(feFrom));
                            }
                            MessageBoxW((*op).hWndOwner, strText.as_ptr(), strTitle.as_ptr(), MB_ICONERROR);
                            (*op).bHasDisplayedError += 1;
                            return DE_DESTSUBTREE;
                        }
                        return DE_OPCANCELLED;
                    }
                }
            }
            i += 1;
        }
        k += 1;
    }

    return (ERROR_SUCCESS) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_FileOperation(
    lpFileOp: *mut SHFILEOPSTRUCTW,
    Callback: FILEOPCALLBACK,
    CallerData: *mut core::ffi::c_void,
) -> i32 {
    let mut op: FILE_OPERATION = core::mem::zeroed();
    let mut flFrom: FILE_LIST = core::mem::zeroed();
    let mut flTo: FILE_LIST = core::mem::zeroed();
    let mut ret: i32 = 0;

    if lpFileOp.is_null() {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    ret = CoInitialize(core::ptr::null_mut());
    if ret < 0 {
        return ret;
    }

    (*lpFileOp).fAnyOperationsAborted = FALSE;
    check_flags((*lpFileOp).fFlags);

    ZeroMemory(
        &mut flFrom as *mut FILE_LIST as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_LIST>(),
    );
    ZeroMemory(
        &mut flTo as *mut FILE_LIST as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_LIST>(),
    );

    ret = parse_file_list(&mut flFrom, (*lpFileOp).pFrom);
    if ret != 0 {
        return ret;
    }

    if (*lpFileOp).wFunc != FO_DELETE {
        parse_file_list(&mut flTo, (*lpFileOp).pTo);
    }

    ZeroMemory(
        &mut op as *mut FILE_OPERATION as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_OPERATION>(),
    );
    op.req = lpFileOp;
    op.totalSize.QuadPart = 0u64;
    op.completedSize.QuadPart = 0u64;
    op.bManyItems = if flFrom.dwNumFiles > 1 { 1 } else { 0 };
    op.Callback = Callback;
    op.CallerCallbackData = CallerData;
    op.hWndOwner = GetAncestor((*lpFileOp).hwnd, GA_ROOT);

    ret = (validate_operation(&mut op, &flFrom, &flTo)) as i32;
    if ret != 0 {
        destroy_file_list(&mut flFrom);
        if (*lpFileOp).wFunc != FO_DELETE {
            destroy_file_list(&mut flTo);
        } else if (*lpFileOp).fFlags & (FOF_ALLOWUNDO) as u16 != 0 {
            SHUpdateRecycleBinIcon();
        }
        if ret == (ERROR_CANCELLED) as i32 {
            (*lpFileOp).fAnyOperationsAborted = TRUE;
        }
        if ret == (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32 && LOBYTE(GetVersion()) >= 6 {
            ret = (ERROR_FILE_NOT_FOUND) as i32;
        }
        let hr = SHELL32_FileOpErrorToHResult(ret, ((*lpFileOp).fAnyOperationsAborted) as u8);
        FileOpCallback(&mut op, FOCE_FINISHOPERATIONS, core::ptr::null(), core::ptr::null(), 0, hr);
        if hr < 0 && CanShowFileOpErrorUI(op.req as *const SHFILEOPSTRUCTW) != 0 && op.bHasDisplayedError == 0 {
            SHELL_ErrorBox((op.hWndOwner as *mut c_void).cast(), (hr) as u32);
        }
        CoUninitialize();
        return ret;
    }

    if (*lpFileOp).wFunc != FO_RENAME && ((*lpFileOp).fFlags & (FOF_SILENT) as u16) == 0 {
        ret = CoCreateInstance(
            &CLSID_ProgressDialog,
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            core::ptr::addr_of!(IID_IProgressDialog), core::ptr::addr_of_mut!(op.progress).cast(),
        );
        if ret < 0 {
            destroy_file_list(&mut flFrom);
            if (*lpFileOp).wFunc != FO_DELETE {
                destroy_file_list(&mut flTo);
            } else if (*lpFileOp).fFlags & (FOF_ALLOWUNDO) as u16 != 0 {
                SHUpdateRecycleBinIcon();
            }
            if ret == (ERROR_CANCELLED) as i32 {
                (*lpFileOp).fAnyOperationsAborted = TRUE;
            }
            if ret == (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32 && LOBYTE(GetVersion()) >= 6 {
                ret = (ERROR_FILE_NOT_FOUND) as i32;
            }
            let hr = SHELL32_FileOpErrorToHResult(ret, ((*lpFileOp).fAnyOperationsAborted) as u8);
            FileOpCallback(&mut op, FOCE_FINISHOPERATIONS, core::ptr::null(), core::ptr::null(), 0, hr);
            if hr < 0 && CanShowFileOpErrorUI(op.req as *const SHFILEOPSTRUCTW) != 0 && op.bHasDisplayedError == 0 {
                SHELL_ErrorBox((op.hWndOwner as *mut c_void).cast(), (hr) as u32);
            }
            CoUninitialize();
            return ret;
        }

        ((*(*op.progress).lpVtbl).StartProgressDialog)(op.progress, op.hWndOwner, core::ptr::null_mut(), PROGDLG_NORMAL & PROGDLG_AUTOTIME, core::ptr::null());
        _SetOperationTitle(&mut op);
        _FileOpCountManager(&mut op, &flFrom);
    }

    FileOpCallback(&mut op, FOCE_STARTOPERATIONS, core::ptr::null(), core::ptr::null(), 0, 0);

    match (*lpFileOp).wFunc {
        FO_COPY => {
            ret = copy_files(&mut op, if (*op.req).fFlags & (FOF_MULTIDESTFILES) as u16 != 0 { 1 } else { 0 }, &flFrom, &mut flTo);
        }
        FO_DELETE => {
            ret = delete_files(&mut op, &flFrom);
        }
        FO_MOVE => {
            ret = (move_files(&mut op, if (*op.req).fFlags & (FOF_MULTIDESTFILES) as u16 != 0 { 1 } else { 0 }, &flFrom, &flTo)) as i32;
        }
        FO_RENAME => {
            ret = rename_files(&mut op, &flFrom, &flTo);
        }
        _ => {
            ret = (ERROR_INVALID_PARAMETER) as i32;
        }
    }

    if !op.progress.is_null() {
        ((*(*op.progress).lpVtbl).StopProgressDialog)(op.progress);
        ((*(*op.progress).lpVtbl).Release)(op.progress);
    }

    destroy_file_list(&mut flFrom);

    if (*lpFileOp).wFunc != FO_DELETE {
        destroy_file_list(&mut flTo);
    } else if (*lpFileOp).fFlags & (FOF_ALLOWUNDO) as u16 != 0 {
        SHUpdateRecycleBinIcon();
    }

    if ret == (ERROR_CANCELLED) as i32 {
        (*lpFileOp).fAnyOperationsAborted = TRUE;
    }
    if ret == (ERROR_SHELL_INTERNAL_FILE_NOT_FOUND) as i32 && LOBYTE(GetVersion()) >= 6 {
        ret = (ERROR_FILE_NOT_FOUND) as i32;
    }

    let hr = SHELL32_FileOpErrorToHResult(ret, ((*lpFileOp).fAnyOperationsAborted) as u8);

    FileOpCallback(&mut op, FOCE_FINISHOPERATIONS, core::ptr::null(), core::ptr::null(), 0, hr);

    if hr < 0 && CanShowFileOpErrorUI(op.req as *const SHFILEOPSTRUCTW) != 0 && op.bHasDisplayedError == 0 {
        SHELL_ErrorBox((op.hWndOwner as *mut c_void).cast(), (hr) as u32);
    }

    CoUninitialize();

    ret
}

#[no_mangle]
pub unsafe extern "system" fn SHFileOperationW(lpFileOp: *mut SHFILEOPSTRUCTW) -> i32 {
    SHELL32_FileOperation(lpFileOp, None, core::ptr::null_mut())
}
// One definition, spelled `extern "system"`: that IS stdcall on i386 and the C
// convention on 64-bit, and the island now spells the slot it goes into the same way
// (normalize_island_abi.py). The `#[cfg]`-duplicated stdcall/C twins this replaces were
// two bodies that had to be edited in lockstep forever.
unsafe extern "system" fn _DestroyCallback(p: *mut core::ffi::c_void, pData: *mut core::ffi::c_void) -> i32 {
    let lp = p as *mut _SHNAMEMAPPINGW;
    SHFree((*lp).pszOldPath as *mut c_void);
    SHFree((*lp).pszNewPath as *mut c_void);
    TRUE
}


#[no_mangle]
pub unsafe extern "system" fn SHFreeNameMappings(hNameMapping: HANDLE) {
    if !hNameMapping.is_null() {
        DSA_DestroyCallback(
            hNameMapping as *mut _DSA,
            Some(_DestroyCallback),
            core::ptr::null_mut(),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn SheGetDirA(drive: u32, buffer: *mut i8) -> u32 {
    let mut org_path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut drv_path: [i8; 3] = [0; 3];

    if drive != 0 {
        strcpy(
            drv_path.as_mut_ptr(),
            b"A:\0".as_ptr() as *const i8,
        );
        drv_path[0] = (b'A' + (drive as u8) - 1 as u8) as i8;
        GetCurrentDirectoryW(
            (MAX_PATH) as u32,
            org_path.as_mut_ptr(),
        );
        SetCurrentDirectoryA(drv_path.as_ptr());
    }

    let ret = GetCurrentDirectoryA(
        MAX_PATH,
        buffer,
    );

    if drive != 0 {
        SetCurrentDirectoryW(org_path.as_ptr());
    }

    if ret == 0 {
        return GetLastError();
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SheGetDirW(drive: u32, buffer: *mut u16) -> u32 {
    let mut org_path: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut drv_path: [u8; 3] = [0u8; 3];
    let mut ret: u32;

    if drive != 0 {
        strcpy(
            drv_path.as_mut_ptr() as *mut c_char,
            b"A:\0".as_ptr() as *const c_char,
        );
        let idx: usize = 0;
        drv_path[idx] += (drive as u8) - 1;

        GetCurrentDirectoryW((MAX_PATH) as u32, org_path.as_mut_ptr());

        SetCurrentDirectoryA(drv_path.as_mut_ptr() as *mut c_char);
    }

    ret = GetCurrentDirectoryW((MAX_PATH) as u32, buffer);

    if drive != 0 {
        SetCurrentDirectoryW(org_path.as_mut_ptr());
    }

    if ret == 0 {
        return GetLastError();
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SheChangeDirA(path: *mut i8) -> u32 {
    if SetCurrentDirectoryA(path) != 0 {
        return 0;
    } else {
        return GetLastError();
    }
}

#[no_mangle]
pub unsafe extern "system" fn SheChangeDirW(path: PWSTR) -> DWORD {
    if SetCurrentDirectoryW(path) != 0 {
        0
    } else {
        GetLastError()
    }
}

#[no_mangle]
pub unsafe extern "system" fn IsNetDrive(drive: core::ffi::c_int) -> core::ffi::c_int {
    let mut root: [core::ffi::c_char; 4] = [0; 4];
    strcpy(root.as_mut_ptr(), b"A:\\\0".as_ptr() as *const core::ffi::c_char);
    let index = 0;
    root[index] = (root[index] as core::ffi::c_int + drive) as core::ffi::c_char;
    (GetDriveTypeA(root.as_mut_ptr()) == DRIVE_REMOTE) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "system" fn RealDriveType(drive: i32, bQueryNet: i32) -> i32 {
    let mut root: [u8; 4] = [b'A', b':', b'\\', 0];
    root[0] += drive as u8;
    GetDriveTypeA(root.as_ptr() as *const i8) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SHPathPrepareForWriteW(
    hwnd: HWND,
    modless: *mut IUnknown,
    path: PCWSTR,
    flags: u32,
) -> i32 {
    let mut res: u32;
    let mut err: u32;
    let realpath: PCWSTR;
    let mut len: i32;
    let last_slash: *mut u16;
    let mut temppath: *mut u16 = core::ptr::null_mut();

    TRACE!("%p %p %s 0x%08x\n", hwnd, modless, debugstr_w(path), flags);

    if flags & !(SHPPFW_DIRCREATE | SHPPFW_ASKDIRCREATE | SHPPFW_IGNOREFILENAME) != 0 {
        FIXME!("unimplemented flags 0x%08x\n", flags);
    }

    if flags & SHPPFW_IGNOREFILENAME != 0 {
        last_slash = StrRChrW(path, core::ptr::null_mut(), '\\' as u16);
        if last_slash.is_null() {
            len = 1;
        } else {
            len = ((last_slash as usize).wrapping_sub(path as usize) / core::mem::size_of::<u16>()) as i32
                + 1;
        }
        temppath = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize * core::mem::size_of::<u16>()) as SIZE_T,
        ) as *mut u16;
        if temppath.is_null() {
            return E_OUTOFMEMORY;
        }
        StrCpyNW(temppath, path, len);
        realpath = temppath;
    } else {
        realpath = path;
    }

    if flags & (SHPPFW_DIRCREATE | SHPPFW_ASKDIRCREATE) != 0 {
        if flags & SHPPFW_ASKDIRCREATE != 0 {
            FIXME!("treating SHPPFW_ASKDIRCREATE as SHPPFW_DIRCREATE\n");
        }
        SHCreateDirectoryExW(
            0 as HWND,
            realpath,
            core::ptr::null_mut(),
        );
    }

    res = GetFileAttributesW(realpath);
    HeapFree(
        GetProcessHeap(),
        0,
        temppath as *mut core::ffi::c_void,
    );

    if res == INVALID_FILE_ATTRIBUTES {
        err = GetLastError();
        if err == ERROR_FILE_NOT_FOUND {
            return HRESULT_FROM_WIN32((ERROR_PATH_NOT_FOUND) as i32);
        }
        return HRESULT_FROM_WIN32((err) as i32);
    } else if res & FILE_ATTRIBUTE_DIRECTORY != 0 {
        return S_OK;
    } else {
        return HRESULT_FROM_WIN32((ERROR_DIRECTORY) as i32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHPathPrepareForWriteA(
    hwnd: HWND,
    modless: *mut IUnknown,
    path: *const i8,
    flags: u32,
) -> i32 {
    let mut wpath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    MultiByteToWideChar(
        CP_ACP,
        0,
        path,
        -1,
        wpath.as_mut_ptr(),
        (MAX_PATH) as i32,
    );
    SHPathPrepareForWriteW(hwnd, modless, wpath.as_ptr(), flags)
}
unsafe extern "C" fn SHELL_DupSZZ(Input: PCWSTR) -> PWSTR {
    let len = wcslen(Input);
    let cb = (len + 2) * core::mem::size_of::<u16>() as usize;
    let Output = SHAlloc(cb as SIZE_T) as PWSTR;
    if !Output.is_null() {
        CopyMemory(
            Output as *mut core::ffi::c_void,
            Input as *const core::ffi::c_void,
            cb - core::mem::size_of::<u16>() as usize,
        );
        *Output.add(len + 1) = UNICODE_NULL;
    }
    Output
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_SingleFileOperation(
    hWnd: HWND,
    Op: UINT,
    mut Src: PCWSTR,
    mut Dest: PCWSTR,
    Flags: UINT,
    ppNewName: *mut PWSTR,
) -> HRESULT {
    let mut hr = S_OK;
    Src = SHELL_DupSZZ(Src) as PCWSTR;
    if Src.is_null() {
        hr = E_OUTOFMEMORY;
    }
    if !Dest.is_null() {
        Dest = SHELL_DupSZZ(Dest) as PCWSTR;
        if Dest.is_null() {
            hr = E_OUTOFMEMORY;
        }
    }

    let mut fos: SHFILEOPSTRUCTW = core::mem::zeroed();
    fos.hwnd = hWnd;
    fos.wFunc = Op;
    fos.pFrom = Src;
    fos.pTo = Dest;
    fos.fFlags = Flags as FILEOP_FLAGS;

    if !ppNewName.is_null() {
        *ppNewName = core::ptr::null_mut();
        fos.fFlags |= (FOF_WANTMAPPINGHANDLE) as u16;
    }

    if hr >= 0 {
        let err = SHFileOperationW(&mut fos);
        hr = SHELL32_FileOpErrorToHResult(err, (fos.fAnyOperationsAborted) as u8);
    } else if CanShowFileOpErrorUI(&fos) != 0 {
        SHELL_ErrorBox((hWnd as *mut c_void).cast(), (hr) as u32);
    }

    SHFree((Src as PWSTR).cast());
    SHFree((Dest as PWSTR).cast());

    if fos.hNameMappings != core::ptr::null_mut() {
        if hr >= 0 && !ppNewName.is_null() {
            assert!(
                DSA_GetItemCount(fos.hNameMappings as HDSA) == 1 as i32,
            );
            let pMap = DSA_GetItemPtr(fos.hNameMappings as HDSA, 0)
                as *mut SHNAMEMAPPINGW;
            *ppNewName = SHELL_DupSZZ((*pMap).pszNewPath);
            if (*ppNewName).is_null() {
                hr = S_FALSE;
            }
        }
        SHFreeNameMappings(fos.hNameMappings);
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_SingleFileOperation_RustBridge(
    hWnd: HWND,
    Op: UINT,
    Src: PCWSTR,
    Dest: PCWSTR,
    Flags: UINT,
    ppNewName: *mut PWSTR,
) -> HRESULT {
    SHELL_SingleFileOperation(hWnd, Op, Src, Dest, Flags, ppNewName)
}
#[no_mangle]
pub unsafe extern "system" fn _FileOpCountManager(op: *mut FILE_OPERATION, from: *const FILE_LIST) -> u32 {
    let mut ticks: u32 = GetTickCount();
    let mut i: u32 = 0;

    while i < (*from).dwNumFiles {
        let entry_to_count: *mut FILE_ENTRY = (*from).feFiles.add(i as usize);

        let mut the_file_name: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        StringCchCopyW(
            the_file_name.as_mut_ptr(),
            MAX_PATH as usize,
            (*entry_to_count).szFullPath,
        );

        _FileOpCount(
            op,
            the_file_name.as_mut_ptr(),
            IsAttribDir!((*entry_to_count).attributes) as BOOL,
            core::ptr::addr_of_mut!(ticks),
        );

        i += 1;
    }

    0
}
unsafe extern "C" fn _FileOpCount(op: *mut FILE_OPERATION, pwszBuf: *mut u16, bFolder: i32, ticks: *mut u32) -> i32 {
    let cchBuf = wcslen(pwszBuf) as u32;
    let mut pwszFilename = pwszBuf.offset(cchBuf as isize);
    let mut cchFilenameMax = (MAX_PATH as u32).wrapping_sub(cchBuf);
    if cchFilenameMax == 0 {
        return FALSE;
    }

    if bFolder != 0 {
        *pwszFilename = '\\' as u16;
        pwszFilename = pwszFilename.offset(1);
        cchFilenameMax = cchFilenameMax.wrapping_sub(1);
        StringCchCopyW(pwszFilename, cchFilenameMax as usize, CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());
    }

    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind = FindFirstFileW(pwszBuf, &mut wfd);
    if hFind == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    loop {
        if (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            if wcscmp(wfd.cFileName.as_ptr(), CPP_COM_WSTR_C245AFF91259.as_ptr()) == 0 ||
               wcscmp(wfd.cFileName.as_ptr(), CPP_COM_WSTR_77ECA80E282A.as_ptr()) == 0 {
            } else {
                StringCchCopyW(pwszFilename, cchFilenameMax as usize, wfd.cFileName.as_ptr());
                _FileOpCount(op, pwszBuf, TRUE, ticks);
            }
        } else {
            let file_size = ((wfd.nFileSizeHigh as u64) << 32) | (wfd.nFileSizeLow as u64);
            (*op).totalSize.QuadPart = (*op).totalSize.QuadPart.wrapping_add(file_size);
        }

        if GetTickCount().wrapping_sub(*ticks) > (500 as u32) {
            if !(*op).progress.is_null() {
                (*op).bCancelled = ((*(*(*op).progress).lpVtbl).HasUserCancelled)((*op).progress);
            }
            if (*op).bCancelled != 0 {
                break;
            }
            *ticks = GetTickCount();
        }
        if FindNextFileW(hFind, &mut wfd) == 0 {
            break;
        }
    }

    FindClose(hFind);
    TRUE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * SHFileOperation
//  *
//  * Copyright 2000 Juergen Schmied
//  * Copyright 2002 Andriy Palamarchuk
//  * Copyright 2004 Dietrich Teickner (from Odin)
//  * Copyright 2004 Rolf Kalbermatter
//  * Copyright 2019-2023 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
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
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// #define IsAttrib(x, y)  ((INVALID_FILE_ATTRIBUTES != (x)) && ((x) & (y)))
// #define IsAttribFile(x) (!((x) & FILE_ATTRIBUTE_DIRECTORY))
// #define IsAttribDir(x)  IsAttrib(x, FILE_ATTRIBUTE_DIRECTORY)
// #define IsDotDir(x)     ((x[0] == '.') && ((x[1] == 0) || ((x[1] == '.') && (x[2] == 0))))
//
// #define FO_MASK         0xF
//
// #define NEW_FILENAME_ON_COPY_TRIES 100
//
// typedef struct
// {
//     SHFILEOPSTRUCTW *req;
//     DWORD dwYesToAllMask;
//     BOOL bManyItems;
//     BOOL bCancelled;
//     IProgressDialog *progress;
//     ULARGE_INTEGER completedSize;
//     ULARGE_INTEGER totalSize;
//     WCHAR szBuilderString[50];
//     FILEOPCALLBACK Callback;
//     void *CallerCallbackData;
//     HWND hWndOwner;
//     BOOL bHasDisplayedError;
// } FILE_OPERATION;
//
// #define ERROR_SHELL_INTERNAL_FILE_NOT_FOUND 1026
//
// typedef struct
// {
//     DWORD attributes;
//     LPWSTR szDirectory;
//     LPWSTR szFilename;
//     LPWSTR szFullPath;
//     BOOL bFromWildcard;
//     BOOL bFromRelative;
//     BOOL bExists;
// } FILE_ENTRY;
//
// typedef struct
// {
//     FILE_ENTRY *feFiles;
//     DWORD num_alloc;
//     DWORD dwNumFiles;
//     BOOL bAnyFromWildcard;
//     BOOL bAnyDirectories;
//     BOOL bAnyDontExist;
// } FILE_LIST;
//
// static DWORD SHNotifyCreateDirectoryW(LPCWSTR path, LPSECURITY_ATTRIBUTES sec);
// static DWORD SHNotifyRemoveDirectoryW(LPCWSTR path);
// static DWORD SHNotifyDeleteFileW(FILE_OPERATION *op, LPCWSTR path);
// static DWORD SHNotifyMoveFileW(FILE_OPERATION *op, LPCWSTR src, LPCWSTR dest, BOOL isdir);
// static DWORD SHNotifyCopyFileW(FILE_OPERATION *op, LPCWSTR src, LPCWSTR dest, BOOL bFailIfExists);
// static DWORD SHFindAttrW(LPCWSTR pName, BOOL fileOnly);
// static HRESULT copy_files(FILE_OPERATION *op, BOOL multiDest, const FILE_LIST *flFrom, FILE_LIST *flTo);
// static DWORD move_files(FILE_OPERATION *op, BOOL multiDest, const FILE_LIST *flFrom, const FILE_LIST *flTo);
//
// DWORD WINAPI _FileOpCountManager(FILE_OPERATION *op, const FILE_LIST *flFrom);
// static BOOL _FileOpCount(FILE_OPERATION *op, LPWSTR pwszBuf, BOOL bFolder, DWORD *ticks);
//
// static HRESULT SHELL32_FileOpErrorToHResult(int err, BOOL AnyOperationsAborted = FALSE)
// {
//     enum { de_min = DE_SAMEFILE, de_max = 0xB7 /*DE_ERROR_MAX*/ };
//     const bool IsDeErr = (err & ~ERRORONDEST) >= de_min && (err & ~ERRORONDEST) <= de_max;
//     if (err == DE_OPCANCELLED || AnyOperationsAborted)
//         return HRESULT_FROM_WIN32(ERROR_CANCELLED);
//     if (err == DE_ACCESSDENIEDSRC)
//         return HRESULT_FROM_WIN32(ERROR_ACCESS_DENIED);
//     if (err == DE_FILENAMETOOLONG)
//         return HRESULT_FROM_WIN32(ERROR_FILENAME_EXCED_RANGE);
//     if (err == ERROR_SHELL_INTERNAL_FILE_NOT_FOUND)
//         return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
//     if (err == ERROR_ALREADY_EXISTS)
//         return HRESULT_FROM_WIN32(err);
//     return IsDeErr ? E_FAIL : HRESULT_FROM_WIN32(err);
// }
//
// static BOOL CanShowFileOpErrorUI(const SHFILEOPSTRUCTW &fos)
// {
//     if ((fos.fFlags & FOF_SILENT) && !fos.hwnd)
//         return FALSE;
//     return !(fos.fFlags & FOF_NOERRORUI);
// }
//
// static void HandleDesktopIniOp(PCWSTR Path)
// {
//     // Refresh the parent folder if somebody changes the desktop.ini inside
//     PCWSTR Name = PathFindFileNameW(Path);
//     if ((Name[0] | 32) != 'd' || _wcsicmp(Name, L"desktop.ini"))
//         return;
//     WCHAR Dir[MAX_PATH];
//     if (FAILED_UNEXPECTEDLY(StringCchCopyW(Dir, _countof(Dir), Path)))
//         return;
//     PathRemoveFileSpecW(Dir);
//     if (!(GetFileAttributesW(Dir) & (FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_READONLY)))
//         return;
//     SHChangeNotify(SHCNE_UPDATEITEM, SHCNF_PATHW, Dir, NULL);
// }
//
// /* Confirm dialogs with an optional "Yes To All" as used in file operations confirmations
//  */
// struct confirm_msg_info
// {
//     LPWSTR lpszText;
//     LPWSTR lpszCaption;
//     HICON hIcon;
//     BOOL bYesToAll;
// };
//
// /* as some buttons may be hidden and the dialog height may change we may need
//  * to move the controls */
// static void confirm_msg_move_button(HWND hDlg, INT iId, INT *xPos, INT yOffset, BOOL bShow)
// {
//     HWND hButton = GetDlgItem(hDlg, iId);
//     RECT r;
//
//     if (bShow)
//     {
//         POINT pt;
//         int width;
//
//         GetWindowRect(hButton, &r);
//         width = r.right - r.left;
//         pt.x = r.left;
//         pt.y = r.top;
//         ScreenToClient(hDlg, &pt);
//         MoveWindow(hButton, *xPos - width, pt.y - yOffset, width, r.bottom - r.top, FALSE);
//         *xPos -= width + 5;
//     }
//     else
//         ShowWindow(hButton, SW_HIDE);
// }
//
// /* Note: we paint the text manually and don't use the static control to make
//  * sure the text has the same height as the one computed in WM_INITDIALOG
//  */
// static INT_PTR ConfirmMsgBox_Paint(HWND hDlg)
// {
//     PAINTSTRUCT ps;
//     HFONT hOldFont;
//     RECT r;
//     HDC hdc;
//
//     BeginPaint(hDlg, &ps);
//     hdc = ps.hdc;
//     SetBkMode(hdc, TRANSPARENT);
//     SetTextColor(hdc, GetSysColor(COLOR_BTNTEXT));
//
//     GetClientRect(GetDlgItem(hDlg, IDC_YESTOALL_MESSAGE), &r);
//     /* this will remap the rect to dialog coords */
//     MapWindowPoints(GetDlgItem(hDlg, IDC_YESTOALL_MESSAGE), hDlg, (LPPOINT)&r, 2);
//     hOldFont = (HFONT)SelectObject(hdc, (HFONT)SendDlgItemMessageW(hDlg, IDC_YESTOALL_MESSAGE, WM_GETFONT, 0, 0));
//     DrawTextW(hdc, (LPWSTR)GetWindowLongPtrW(hDlg, DWLP_USER), -1, &r, DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_WORDBREAK);
//     SelectObject(hdc, hOldFont);
//     EndPaint(hDlg, &ps);
//
//     return TRUE;
// }
//
// static INT_PTR ConfirmMsgBox_Init(HWND hDlg, LPARAM lParam)
// {
//     struct confirm_msg_info *info = (struct confirm_msg_info *)lParam;
//     INT xPos, yOffset;
//     int width, height;
//     HFONT hOldFont;
//     HDC hdc;
//     RECT r;
//
//     SetWindowTextW(hDlg, info->lpszCaption);
//     ShowWindow(GetDlgItem(hDlg, IDC_YESTOALL_MESSAGE), SW_HIDE);
//     SetWindowLongPtrW(hDlg, DWLP_USER, (LONG_PTR)info->lpszText);
//     SendDlgItemMessageW(hDlg, IDC_YESTOALL_ICON, STM_SETICON, (WPARAM)info->hIcon, 0);
//
//     /* compute the text height and resize the dialog */
//     GetClientRect(GetDlgItem(hDlg, IDC_YESTOALL_MESSAGE), &r);
//     hdc = GetDC(hDlg);
//     yOffset = r.bottom;
//     hOldFont = (HFONT)SelectObject(hdc, (HFONT)SendDlgItemMessageW(hDlg, IDC_YESTOALL_MESSAGE, WM_GETFONT, 0, 0));
//     DrawTextW(hdc, info->lpszText, -1, &r, DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_WORDBREAK | DT_CALCRECT);
//     SelectObject(hdc, hOldFont);
//     yOffset -= r.bottom;
//     yOffset = min(yOffset, 35);  /* don't make the dialog too small */
//     ReleaseDC(hDlg, hdc);
//
//     GetClientRect(hDlg, &r);
//     xPos = r.right - 7;
//     GetWindowRect(hDlg, &r);
//     width = r.right - r.left;
//     height = r.bottom - r.top - yOffset;
//     MoveWindow(hDlg, (GetSystemMetrics(SM_CXSCREEN) - width)/2,
//         (GetSystemMetrics(SM_CYSCREEN) - height)/2, width, height, FALSE);
//
//     confirm_msg_move_button(hDlg, IDCANCEL,     &xPos, yOffset, info->bYesToAll);
//     confirm_msg_move_button(hDlg, IDNO,         &xPos, yOffset, TRUE);
//     confirm_msg_move_button(hDlg, IDC_YESTOALL, &xPos, yOffset, info->bYesToAll);
//     confirm_msg_move_button(hDlg, IDYES,        &xPos, yOffset, TRUE);
//
//     return TRUE;
// }
//
// static INT_PTR __stdcall ConfirmMsgBoxProc(HWND hDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return ConfirmMsgBox_Init(hDlg, lParam);
//         case WM_PAINT:
//             return ConfirmMsgBox_Paint(hDlg);
//         case WM_COMMAND:
//             EndDialog(hDlg, wParam);
//             break;
//         case WM_CLOSE:
//             EndDialog(hDlg, IDCANCEL);
//             break;
//     }
//     return FALSE;
// }
//
// int SHELL_ConfirmMsgBox(HWND hWnd, LPWSTR lpszText, LPWSTR lpszCaption, HICON hIcon, BOOL bYesToAll)
// {
//     struct confirm_msg_info info;
//
//     info.lpszText = lpszText;
//     info.lpszCaption = lpszCaption;
//     info.hIcon = hIcon;
//     info.bYesToAll = bYesToAll;
//     return DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_YESTOALL_MSGBOX), hWnd, ConfirmMsgBoxProc, (LPARAM)&info);
// }
//
// /* confirmation dialogs content */
// typedef struct
// {
//     HINSTANCE hIconInstance;
//     UINT icon_resource_id;
//     UINT caption_resource_id, text_resource_id;
// } SHELL_ConfirmIDstruc;
//
// static BOOL SHELL_ConfirmIDs(int nKindOfDialog, SHELL_ConfirmIDstruc *ids)
// {
//     ids->hIconInstance = shell32_hInstance;
//     switch (nKindOfDialog)
//     {
//         case ASK_DELETE_FILE:
//             ids->icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
//             ids->caption_resource_id  = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id  = IDS_DELETEITEM_TEXT;
//             return TRUE;
//
//         case ASK_DELETE_FOLDER:
//             ids->icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
//             ids->caption_resource_id  = IDS_DELETEFOLDER_CAPTION;
//             ids->text_resource_id  = IDS_DELETEITEM_TEXT;
//             return TRUE;
//
//         case ASK_DELETE_MULTIPLE_ITEM:
//             ids->icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
//             ids->caption_resource_id  = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id  = IDS_DELETEMULTIPLE_TEXT;
//             return TRUE;
//
//         case ASK_TRASH_FILE:
//             ids->icon_resource_id = IDI_SHELL_TRASH_FILE;
//             ids->caption_resource_id = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id = IDS_TRASHITEM_TEXT;
//             return TRUE;
//
//         case ASK_TRASH_FOLDER:
//             ids->icon_resource_id = IDI_SHELL_TRASH_FILE;
//             ids->caption_resource_id = IDS_DELETEFOLDER_CAPTION;
//             ids->text_resource_id = IDS_TRASHFOLDER_TEXT;
//             return TRUE;
//
//         case ASK_TRASH_MULTIPLE_ITEM:
//             ids->icon_resource_id = IDI_SHELL_TRASH_FILE;
//             ids->caption_resource_id = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id = IDS_TRASHMULTIPLE_TEXT;
//             return TRUE;
//
//         case ASK_CANT_TRASH_ITEM:
//             ids->icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
//             ids->caption_resource_id  = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id  = IDS_CANTTRASH_TEXT;
//             return TRUE;
//
//         case ASK_DELETE_SELECTED:
//             ids->icon_resource_id = IDI_SHELL_CONFIRM_DELETE;
//             ids->caption_resource_id  = IDS_DELETEITEM_CAPTION;
//             ids->text_resource_id  = IDS_DELETESELECTED_TEXT;
//             return TRUE;
//
//       case ASK_OVERWRITE_FILE:
//             ids->icon_resource_id = IDI_SHELL_FOLDER_MOVE2;
//             ids->caption_resource_id  = IDS_OVERWRITEFILE_CAPTION;
//             ids->text_resource_id  = IDS_OVERWRITEFILE_TEXT;
//             return TRUE;
//
//       case ASK_OVERWRITE_FOLDER:
//             ids->icon_resource_id = IDI_SHELL_FOLDER_MOVE2;
//             ids->caption_resource_id  = IDS_OVERWRITEFILE_CAPTION;
//             ids->text_resource_id  = IDS_OVERWRITEFOLDER_TEXT;
//             return TRUE;
//
//         default:
//             FIXME(" Unhandled nKindOfDialog %d stub\n", nKindOfDialog);
//     }
//     return FALSE;
// }
//
// static BOOL SHELL_ConfirmDialogW(HWND hWnd, int nKindOfDialog, LPCWSTR szDir, FILE_OPERATION *op)
// {
//     WCHAR szCaption[255], szText[255], szBuffer[MAX_PATH + 256];
//     SHELL_ConfirmIDstruc ids;
//     DWORD_PTR args[1];
//     HICON hIcon;
//     int ret;
//
//     assert(nKindOfDialog >= 0 && nKindOfDialog < 32);
//     if (op && (op->dwYesToAllMask & (1 << nKindOfDialog)))
//         return TRUE;
//
//     if (!SHELL_ConfirmIDs(nKindOfDialog, &ids)) return FALSE;
//
//     LoadStringW(shell32_hInstance, ids.caption_resource_id, szCaption, sizeof(szCaption)/sizeof(WCHAR));
//     LoadStringW(shell32_hInstance, ids.text_resource_id, szText, sizeof(szText)/sizeof(WCHAR));
//
//     args[0] = (DWORD_PTR)szDir;
//     FormatMessageW(FORMAT_MESSAGE_FROM_STRING|FORMAT_MESSAGE_ARGUMENT_ARRAY,
//                    szText, 0, 0, szBuffer, sizeof(szBuffer), (va_list*)args);
//     hIcon = LoadIconW(ids.hIconInstance, (LPWSTR)MAKEINTRESOURCE(ids.icon_resource_id));
//
//     ret = SHELL_ConfirmMsgBox(hWnd, szBuffer, szCaption, hIcon, op && op->bManyItems);
//     if (op)
//     {
//         if (ret == IDC_YESTOALL)
//         {
//             op->dwYesToAllMask |= (1 << nKindOfDialog);
//             ret = IDYES;
//         }
//         if (ret == IDCANCEL)
//             op->bCancelled = TRUE;
//         if (ret != IDYES)
//             op->req->fAnyOperationsAborted = TRUE;
//     }
//     return ret == IDYES;
// }
//
// BOOL SHELL_ConfirmYesNoW(HWND hWnd, int nKindOfDialog, LPCWSTR szDir)
// {
//     return SHELL_ConfirmDialogW(hWnd, nKindOfDialog, szDir, NULL);
// }
//
// static DWORD SHELL32_AnsiToUnicodeBuf(LPCSTR aPath, LPWSTR *wPath, DWORD minChars)
// {
//     DWORD len = MultiByteToWideChar(CP_ACP, 0, aPath, -1, NULL, 0);
//
//     if (len < minChars)
//       len = minChars;
//
//     *wPath = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     if (*wPath)
//     {
//       MultiByteToWideChar(CP_ACP, 0, aPath, -1, *wPath, len);
//       return NO_ERROR;
//     }
//     return E_OUTOFMEMORY;
// }
//
// static void SHELL32_FreeUnicodeBuf(LPWSTR wPath)
// {
//     HeapFree(GetProcessHeap(), 0, wPath);
// }
//
// extern "C" HRESULT WINAPI SHIsFileAvailableOffline(LPCWSTR path, LPDWORD status)
// {
//     FIXME("(%s, %p) stub\n", debugstr_w(path), status);
//     return E_FAIL;
// }
//
// static HRESULT FileOpCallback(FILE_OPERATION *op, FILEOPCALLBACKEVENT Event, LPCWSTR Source,
//                               LPCWSTR Destination, UINT Attributes, HRESULT hrOp = S_OK)
// {
//     if ((Attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_SYSTEM)) == FILE_ATTRIBUTE_SYSTEM)
//     {
//         if (Event == FOCE_POSTDELETEITEM)
//             HandleDesktopIniOp(Source);
//         if (Event == FOCE_POSTCOPYITEM)
//             HandleDesktopIniOp(Destination);
//         if (Event == FOCE_POSTMOVEITEM || Event == FOCE_POSTRENAMEITEM)
//         {
//             HandleDesktopIniOp(Source);
//             HandleDesktopIniOp(Destination);
//         }
//     }
//
//     HRESULT hr = S_OK;
//     if (op->Callback)
//     {
//         hr = op->Callback(Event, Source, Destination, Attributes, hrOp, op->CallerCallbackData);
//         if (hr == HRESULT_FROM_WIN32(ERROR_CANCELLED))
//             op->bCancelled = TRUE;
//     }
//     return hr;
// }
//
// /**************************************************************************
//  * SHELL_DeleteDirectory()  [internal]
//  *
//  * Asks for confirmation when bShowUI is true and deletes the directory and
//  * all its subdirectories and files if necessary.
//  */
// BOOL SHELL_DeleteDirectoryW(FILE_OPERATION *op, LPCWSTR pszDir, BOOL bShowUI)
// {
//     BOOL    ret = TRUE;
//     HANDLE  hFind;
//     WIN32_FIND_DATAW wfd;
//     WCHAR   szTemp[MAX_PATH];
//
//     /* Make sure the directory exists before eventually prompting the user */
//     PathCombineW(szTemp, pszDir, L"*");
//     hFind = FindFirstFileW(szTemp, &wfd);
//     if (hFind == INVALID_HANDLE_VALUE)
//       return FALSE;
//
//     if (FAILED(FileOpCallback(op, FOCE_PREDELETEITEM, pszDir, NULL, wfd.dwFileAttributes)))
//         return FALSE;
//
//     if (!bShowUI || (ret = SHELL_ConfirmDialogW(op->hWndOwner, ASK_DELETE_FOLDER, pszDir, NULL)))
//     {
//         do
//         {
//             if (IsDotDir(wfd.cFileName))
//                 continue;
//             PathCombineW(szTemp, pszDir, wfd.cFileName);
//             if (FILE_ATTRIBUTE_DIRECTORY & wfd.dwFileAttributes)
//                 ret = SHELL_DeleteDirectoryW(op, szTemp, FALSE);
//             else
//                 ret = (SHNotifyDeleteFileW(op, szTemp) == ERROR_SUCCESS);
//
//             if (op->progress != NULL)
//                 op->bCancelled |= op->progress->HasUserCancelled();
//         } while (ret && FindNextFileW(hFind, &wfd) && !op->bCancelled);
//     }
//     FindClose(hFind);
//     if (ret)
//         ret = (SHNotifyRemoveDirectoryW(pszDir) == ERROR_SUCCESS);
//     FileOpCallback(op, FOCE_POSTDELETEITEM, pszDir, NULL, wfd.dwFileAttributes, ret ? S_OK : E_FAIL);
//     return ret;
// }
//
// /**************************************************************************
//  * Win32CreateDirectory      [SHELL32.93]
//  *
//  * Creates a directory. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  path       [I]   path to directory to create
//  *
//  * RETURNS
//  *  TRUE if successful, FALSE otherwise
//  */
//
// static DWORD SHNotifyCreateDirectoryW(LPCWSTR path, LPSECURITY_ATTRIBUTES sec)
// {
//     TRACE("(%s, %p)\n", debugstr_w(path), sec);
//
//     if (CreateDirectoryW(path, sec))
//     {
//       SHChangeNotify(SHCNE_MKDIR, SHCNF_PATHW, path, NULL);
//       return ERROR_SUCCESS;
//     }
//     return GetLastError();
// }
//
// /**********************************************************************/
//
// extern "C" BOOL WINAPI Win32CreateDirectoryW(LPCWSTR path, LPSECURITY_ATTRIBUTES sec)
// {
//     return (SHNotifyCreateDirectoryW(path, sec) == ERROR_SUCCESS);
// }
//
// /************************************************************************
//  * Win32RemoveDirectory      [SHELL32.94]
//  *
//  * Deletes a directory. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  path       [I]   path to directory to delete
//  *
//  * RETURNS
//  *  TRUE if successful, FALSE otherwise
//  */
// static DWORD SHNotifyRemoveDirectoryW(LPCWSTR path)
// {
//     BOOL ret;
//     TRACE("(%s)\n", debugstr_w(path));
//
//     ret = RemoveDirectoryW(path);
//     if (!ret)
//     {
//       /* Directory may be write protected */
//       DWORD dwAttr = GetFileAttributesW(path);
//       if (IsAttrib(dwAttr, FILE_ATTRIBUTE_READONLY))
//         if (SetFileAttributesW(path, dwAttr & ~FILE_ATTRIBUTE_READONLY))
//           ret = RemoveDirectoryW(path);
//     }
//     if (ret)
//     {
//       SHChangeNotify(SHCNE_RMDIR, SHCNF_PATHW, path, NULL);
//       return ERROR_SUCCESS;
//     }
//     return GetLastError();
// }
//
// /***********************************************************************/
//
// extern "C" BOOL WINAPI Win32RemoveDirectoryW(LPCWSTR path)
// {
//     return (SHNotifyRemoveDirectoryW(path) == ERROR_SUCCESS);
// }
//
// static void _SetOperationTitle(FILE_OPERATION *op) {
//     if (op->progress == NULL)
//         return;
//     WCHAR szTitle[50], szPreflight[50];
//     UINT animation_id = NULL;
//
//     switch (op->req->wFunc)
//     {
//         case FO_COPY:
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_COPYING, szTitle, sizeof(szTitle)/sizeof(WCHAR));
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM_TO, op->szBuilderString, sizeof( op->szBuilderString)/sizeof(WCHAR));
//             animation_id = IDA_SHELL_COPY;
//             break;
//         case FO_DELETE:
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_DELETING, szTitle, sizeof(szTitle)/sizeof(WCHAR));
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM, op->szBuilderString, sizeof( op->szBuilderString)/sizeof(WCHAR));
//             animation_id = IDA_SHELL_DELETE;
//             break;
//         case FO_MOVE:
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_MOVING, szTitle, sizeof(szTitle)/sizeof(WCHAR));
//             LoadStringW(shell32_hInstance, IDS_FILEOOP_FROM_TO, op->szBuilderString, sizeof( op->szBuilderString)/sizeof(WCHAR));
//             animation_id = IDA_SHELL_COPY;
//             break;
//         default:
//             return;
//     }
//     LoadStringW(shell32_hInstance, IDS_FILEOOP_PREFLIGHT, szPreflight, sizeof(szPreflight)/sizeof(WCHAR));
//
//     op->progress->SetTitle(szTitle);
//     op->progress->SetLine(1, szPreflight, false, NULL);
//     op->progress->SetAnimation(shell32_hInstance, animation_id);
// }
//
// static void _SetOperationTexts(FILE_OPERATION *op, LPCWSTR src, LPCWSTR dest) {
//     if (op->progress == NULL || src == NULL)
//         return;
//     LPWSTR fileSpecS, pathSpecS, fileSpecD, pathSpecD;
//     WCHAR szFolderS[50], szFolderD[50], szFinalString[260];
//
//     DWORD_PTR args[2];
//
//     fileSpecS = (pathSpecS = (LPWSTR) src);
//     fileSpecD = (pathSpecD = (LPWSTR) dest);
//
//     // March across the string to get the file path and it's parent dir.
//     for (LPWSTR ptr = (LPWSTR) src; *ptr; ptr++) {
//         if (*ptr == '\\') {
//             pathSpecS = fileSpecS;
//             fileSpecS = ptr+1;
//         }
//     }
//     lstrcpynW(szFolderS, pathSpecS, min(50, fileSpecS - pathSpecS));
//     args[0] = (DWORD_PTR) szFolderS;
//
//     switch (op->req->wFunc)
//     {
//         case FO_COPY:
//         case FO_MOVE:
//             if (dest == NULL)
//                 return;
//             for (LPWSTR ptr = (LPWSTR) dest; *ptr; ptr++) {
//                 if (*ptr == '\\') {
//                     pathSpecD = fileSpecD;
//                     fileSpecD = ptr + 1;
//                 }
//             }
//             lstrcpynW(szFolderD, pathSpecD, min(50, fileSpecD - pathSpecD));
//             args[1] = (DWORD_PTR) szFolderD;
//             break;
//         case FO_DELETE:
//             break;
//         default:
//             return;
//     }
//
//     FormatMessageW(FORMAT_MESSAGE_FROM_STRING|FORMAT_MESSAGE_ARGUMENT_ARRAY,
//                    op->szBuilderString, 0, 0, szFinalString, sizeof(szFinalString), (va_list*)args);
//
//     op->progress->SetLine(1, fileSpecS, false, NULL);
//     op->progress->SetLine(2, szFinalString, false, NULL);
// }
//
//
// DWORD __stdcall SHCopyProgressRoutine(
//     LARGE_INTEGER TotalFileSize,
//     LARGE_INTEGER TotalBytesTransferred,
//     LARGE_INTEGER StreamSize,
//     LARGE_INTEGER StreamBytesTransferred,
//     DWORD dwStreamNumber,
//     DWORD dwCallbackReason,
//     HANDLE hSourceFile,
//     HANDLE hDestinationFile,
//     LPVOID lpData
// ) {
//     FILE_OPERATION *op = (FILE_OPERATION *) lpData;
//
//     if (op->progress) {
//         /*
//          * This is called at the start of each file. To keop less state,
//          * I'm adding the file to the completed size here, and the re-subtracting
//          * it when drawing the progress bar.
//          */
//         if (dwCallbackReason & CALLBACK_STREAM_SWITCH)
//             op->completedSize.QuadPart += TotalFileSize.QuadPart;
//
//         op->progress->SetProgress64(op->completedSize.QuadPart -
//                                     TotalFileSize.QuadPart +
//                                     TotalBytesTransferred.QuadPart
//                                   , op->totalSize.QuadPart);
//
//
//         op->bCancelled = op->progress->HasUserCancelled();
//     }
//
//     return 0;
// }
//
//
// /************************************************************************
//  * SHNotifyDeleteFileW          [internal]
//  *
//  * Deletes a file. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  op         [I]   File Operation context
//  *  path       [I]   path to source file to move
//  *
//  * RETURNS
//  *  ERORR_SUCCESS if successful
//  */
// static DWORD SHNotifyDeleteFileW(FILE_OPERATION *op, LPCWSTR path)
// {
//     BOOL ret;
//
//     TRACE("(%s)\n", debugstr_w(path));
//
//     _SetOperationTexts(op, path, NULL);
//
//     LARGE_INTEGER FileSize;
//     FileSize.QuadPart = 0;
//
//     WIN32_FIND_DATAW wfd;
//     HANDLE hFile = FindFirstFileW(path, &wfd);
//     if (hFile != INVALID_HANDLE_VALUE && IsAttribFile(wfd.dwFileAttributes)) {
//         ULARGE_INTEGER tmp;
//         tmp.u.LowPart  = wfd.nFileSizeLow;
//         tmp.u.HighPart = wfd.nFileSizeHigh;
//         FileSize.QuadPart = tmp.QuadPart;
//     }
//     UINT attrib = hFile != INVALID_HANDLE_VALUE ? wfd.dwFileAttributes : 0;
//     BOOL aborted = FAILED(FileOpCallback(op, FOCE_PREDELETEITEM, path, NULL, attrib));
//     FindClose(hFile);
//     if (aborted)
//         return ERROR_CANCELLED;
//
//     ret = DeleteFileW(path);
//     if (!ret)
//     {
//       /* File may be write protected or a system file */
//       DWORD dwAttr = GetFileAttributesW(path);
//       if (IsAttrib(dwAttr, FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM))
//         if (SetFileAttributesW(path, dwAttr & ~(FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM)))
//             ret = DeleteFileW(path);
//     }
//     FileOpCallback(op, FOCE_POSTDELETEITEM, path, NULL, attrib, ret ? S_OK : E_FAIL);
//     if (ret)
//     {
//         // Bit of a hack to make the progress bar move. We don't have progress inside the file, so inform when done.
//         SHCopyProgressRoutine(FileSize, FileSize, FileSize, FileSize, 0, CALLBACK_STREAM_SWITCH, NULL, NULL, op);
//         SHChangeNotify(SHCNE_DELETE, SHCNF_PATHW, path, NULL);
//         return ERROR_SUCCESS;
//     }
//     return GetLastError();
// }
//
// /************************************************************************
//  * Win32DeleteFile           [SHELL32.164]
//  *
//  * Deletes a file. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  path       [I]   path to file to delete
//  *
//  * RETURNS
//  *  TRUE if successful, FALSE otherwise
//  */
// extern "C" DWORD WINAPI Win32DeleteFileW(LPCWSTR path)
// {
//     return (SHNotifyDeleteFileW(NULL, path) == ERROR_SUCCESS);
// }
//
// /************************************************************************
//  * CheckForError          [internal]
//  *
//  * Show message box if operation failed
//  *
//  * PARAMS
//  *  op         [I]   File Operation context
//  *  error      [I]   Error code
//  *  src        [I]   Source file full name
//  *
//  */
// static DWORD CheckForError(FILE_OPERATION *op, DWORD error, LPCWSTR src)
// {
//     CStringW strTitle, strMask, strText;
//     LPWSTR lpMsgBuffer;
//
//     if (error == ERROR_SUCCESS || !CanShowFileOpErrorUI(*op->req))
//         return error;
//
//     strTitle.LoadStringW(op->req->wFunc == FO_COPY ? IDS_COPYERRORTITLE : IDS_MOVEERRORTITLE);
//
//     FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
//                    NULL,
//                    error,
//                    MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
//                    (LPWSTR)&lpMsgBuffer,
//                    0,
//                    NULL);
//
//     strText.Format(op->req->wFunc == FO_COPY ? IDS_COPYERROR : IDS_MOVEERROR,
//                    PathFindFileNameW(src),
//                    lpMsgBuffer);
//
//     MessageBoxW(op->hWndOwner, strText, strTitle, MB_ICONERROR);
//     LocalFree(lpMsgBuffer);
//     op->bHasDisplayedError++;
//     return error;
// }
//
// /************************************************************************
//  * SHNotifyMoveFile          [internal]
//  *
//  * Moves a file. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  op         [I]   File Operation context
//  *  src        [I]   path to source file to move
//  *  dest       [I]   path to target file to move to
//  *
//  * RETURNS
//  *  ERROR_SUCCESS if successful
//  */
// static DWORD SHNotifyMoveFileW(FILE_OPERATION *op, LPCWSTR src, LPCWSTR dest, BOOL isdir)
// {
//     BOOL ret;
//
//     TRACE("(%s %s)\n", debugstr_w(src), debugstr_w(dest));
//
//     _SetOperationTexts(op, src, dest);
//
//     const BOOL IsRen = op->req->wFunc == FO_RENAME;
//     UINT attrib = GetFileAttributesW(src);
//     if (attrib == INVALID_FILE_ATTRIBUTES)
//         attrib = isdir ? FILE_ATTRIBUTE_DIRECTORY : FILE_ATTRIBUTE_NORMAL;
//     if (FAILED(FileOpCallback(op, IsRen ? FOCE_PRERENAMEITEM : FOCE_PREMOVEITEM, src, dest, attrib)))
//         return ERROR_CANCELLED;
//
//     ret = MoveFileWithProgressW(src, dest, SHCopyProgressRoutine, op, MOVEFILE_REPLACE_EXISTING);
//
//     /* MOVEFILE_REPLACE_EXISTING fails with dirs, so try MoveFile */
//     if (!ret)
//         ret = MoveFileW(src, dest);
//     DWORD LastError = GetLastError();
//
//     if (!ret)
//     {
//         DWORD dwAttr = SHFindAttrW(dest, FALSE);
//         if (INVALID_FILE_ATTRIBUTES == dwAttr)
//         {
//             /* Source file may be write protected or a system file */
//             dwAttr = GetFileAttributesW(src);
//             if (IsAttrib(dwAttr, FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM))
//             {
//                 if (SetFileAttributesW(src, dwAttr & ~(FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_SYSTEM)))
//                 {
//                     ret = MoveFileW(src, dest);
//                     LastError = GetLastError();
//                     SetFileAttributesW(ret ? dest : src, dwAttr);
//                 }
//             }
//         }
//     }
//     FileOpCallback(op, IsRen ? FOCE_POSTRENAMEITEM : FOCE_POSTMOVEITEM, src, dest, attrib, ret ? S_OK : E_FAIL);
//     if (ret)
//     {
//         if (IsRen)
//         {
//             SHChangeNotify(isdir ? SHCNE_RENAMEFOLDER : SHCNE_RENAMEITEM, SHCNF_PATHW, src, dest);
//         }
//         else
//         {
//             SHChangeNotify(isdir ? SHCNE_MKDIR : SHCNE_CREATE, SHCNF_PATHW, dest, NULL);
//             SHChangeNotify(isdir ? SHCNE_RMDIR : SHCNE_DELETE, SHCNF_PATHW, src, NULL);
//         }
//         return ERROR_SUCCESS;
//     }
//     return CheckForError(op, LastError, src);
// }
//
// static BOOL SHIsCdRom(LPCWSTR path)
// {
//     WCHAR tmp[] = { L"A:\\" };
//
//     if (!path || !path[0])
//         return FALSE;
//
//     if (path[1] != UNICODE_NULL && path[1] != ':')
//         return FALSE;
//
//     tmp[0] = path[0];
//
//     return GetDriveTypeW(tmp) == DRIVE_CDROM;
// }
//
// /************************************************************************
//  * SHNotifyCopyFile          [internal]
//  *
//  * Copies a file. Also triggers a change notify if one exists.
//  *
//  * PARAMS
//  *  src           [I]   path to source file to move
//  *  dest          [I]   path to target file to move to
//  *  bFailIfExists [I]   if TRUE, the target file will not be overwritten if
//  *                      a file with this name already exists
//  *
//  * RETURNS
//  *  ERROR_SUCCESS if successful
//  */
// static DWORD SHNotifyCopyFileW(FILE_OPERATION *op, LPCWSTR src, LPCWSTR dest, BOOL bFailIfExists)
// {
//     BOOL ret;
//     DWORD attribs;
//
//     TRACE("(%s %s %s)\n", debugstr_w(src), debugstr_w(dest), bFailIfExists ? "failIfExists" : "");
//
//     _SetOperationTexts(op, src, dest);
//
//     /* Destination file may already exist with read only attribute */
//     attribs = GetFileAttributesW(dest);
//     if (FAILED(FileOpCallback(op, FOCE_PRECOPYITEM, src, dest, attribs)))
//         return ERROR_CANCELLED;
//
//     if (IsAttrib(attribs, FILE_ATTRIBUTE_READONLY))
//         SetFileAttributesW(dest, attribs & ~FILE_ATTRIBUTE_READONLY);
//
//     if (GetFileAttributesW(dest) & FILE_ATTRIBUTE_READONLY)
//     {
//         SetFileAttributesW(dest, attribs & ~FILE_ATTRIBUTE_READONLY);
//         if (GetFileAttributesW(dest) & FILE_ATTRIBUTE_READONLY)
//         {
//             TRACE("[shell32, SHNotifyCopyFileW] STILL SHIT\n");
//         }
//     }
//
//     ret = CopyFileExW(src, dest, SHCopyProgressRoutine, op, &op->bCancelled, bFailIfExists);
//     const DWORD error = ret ? 0 : GetLastError();
//     FileOpCallback(op, FOCE_POSTCOPYITEM, src, dest, attribs, HRESULT_FROM_WIN32(error));
//     if (!error)
//     {
//         // We are copying from a CD-ROM volume, which is readonly
//         if (SHIsCdRom(src))
//         {
//             attribs = GetFileAttributesW(dest);
//             attribs &= ~FILE_ATTRIBUTE_READONLY;
//             SetFileAttributesW(dest, attribs);
//         }
//
//         SHChangeNotify(SHCNE_CREATE, SHCNF_PATHW, dest, NULL);
//         return ERROR_SUCCESS;
//     }
//     return CheckForError(op, error, src);
// }
//
// /*************************************************************************
//  * SHCreateDirectory         [SHELL32.165]
//  *
//  * This function creates a file system folder whose fully qualified path is
//  * given by path. If one or more of the intermediate folders do not exist,
//  * they will be created as well.
//  *
//  * PARAMS
//  *  hWnd       [I]
//  *  path       [I]   path of directory to create
//  *
//  * RETURNS
//  *  ERROR_SUCCESS or one of the following values:
//  *  ERROR_BAD_PATHNAME if the path is relative
//  *  ERROR_FILE_EXISTS when a file with that name exists
//  *  ERROR_PATH_NOT_FOUND can't find the path, probably invalid
//  *  ERROR_INVALID_NAME if the path contains invalid chars
//  *  ERROR_ALREADY_EXISTS when the directory already exists
//  *  ERROR_FILENAME_EXCED_RANGE if the filename was to long to process
//  *
//  * NOTES
//  *  exported by ordinal
//  *  Win9x exports ANSI
//  *  WinNT/2000 exports Unicode
//  */
// int WINAPI SHCreateDirectory(HWND hWnd, LPCWSTR path)
// {
//      return SHCreateDirectoryExW(hWnd, path, NULL);
// }
//
// /*************************************************************************
//  * SHCreateDirectoryExA      [SHELL32.@]
//  *
//  * This function creates a file system folder whose fully qualified path is
//  * given by path. If one or more of the intermediate folders do not exist,
//  * they will be created as well.
//  *
//  * PARAMS
//  *  hWnd       [I]
//  *  path       [I]   path of directory to create
//  *  sec        [I]   security attributes to use or NULL
//  *
//  * RETURNS
//  *  ERROR_SUCCESS or one of the following values:
//  *  ERROR_BAD_PATHNAME or ERROR_PATH_NOT_FOUND if the path is relative
//  *  ERROR_INVALID_NAME if the path contains invalid chars
//  *  ERROR_FILE_EXISTS when a file with that name exists
//  *  ERROR_ALREADY_EXISTS when the directory already exists
//  *  ERROR_FILENAME_EXCED_RANGE if the filename was too long to process
//  *
//  *  FIXME: Not implemented yet;
//  *  SHCreateDirectoryEx also verifies that the files in the directory will be visible
//  *  if the path is a network path to deal with network drivers which might have a limited
//  *  but unknown maximum path length. If not:
//  *
//  *  If hWnd is set to a valid window handle, a message box is displayed warning
//  *  the user that the files may not be accessible. If the user chooses not to
//  *  proceed, the function returns ERROR_CANCELLED.
//  *
//  *  If hWnd is set to NULL, no user interface is displayed and the function
//  *  returns ERROR_CANCELLED.
//  */
// int WINAPI SHCreateDirectoryExA(HWND hWnd, LPCSTR path, LPSECURITY_ATTRIBUTES sec)
// {
//     LPWSTR wPath;
//     DWORD retCode;
//
//     TRACE("(%s, %p)\n", debugstr_a(path), sec);
//
//     retCode = SHELL32_AnsiToUnicodeBuf(path, &wPath, 0);
//     if (!retCode)
//     {
//         retCode = SHCreateDirectoryExW(hWnd, wPath, sec);
//         SHELL32_FreeUnicodeBuf(wPath);
//     }
//     return retCode;
// }
//
// /*************************************************************************
//  * SHCreateDirectoryExW      [SHELL32.@]
//  *
//  * See SHCreateDirectoryExA.
//  */
// int WINAPI SHCreateDirectoryExW(HWND hWnd, LPCWSTR path, LPSECURITY_ATTRIBUTES sec)
// {
//     int ret = ERROR_BAD_PATHNAME;
//     TRACE("(%p, %s, %p)\n", hWnd, debugstr_w(path), sec);
//
//     if (PathIsRelativeW(path))
//     {
//       SetLastError(ret);
//     }
//     else
//     {
//         ret = SHNotifyCreateDirectoryW(path, sec);
//         /* Refuse to work on certain error codes before trying to create directories recursively */
//         if (ret != ERROR_SUCCESS &&
//           ret != ERROR_FILE_EXISTS &&
//           ret != ERROR_ALREADY_EXISTS &&
//           ret != ERROR_FILENAME_EXCED_RANGE)
//         {
//             WCHAR *pEnd, *pSlash, szTemp[MAX_PATH + 1];  /* extra for PathAddBackslash() */
//
//             lstrcpynW(szTemp, path, MAX_PATH);
//             pEnd = PathAddBackslashW(szTemp);
//             pSlash = szTemp + 3;
//
//             while (*pSlash)
//             {
//                 while (*pSlash && *pSlash != '\\') pSlash++;
//                 if (*pSlash)
//                 {
//                     *pSlash = 0;    /* terminate path at separator */
//
//                     ret = SHNotifyCreateDirectoryW(szTemp, pSlash + 1 == pEnd ? sec : NULL);
//                 }
//                 *pSlash++ = '\\'; /* put the separator back */
//             }
//         }
//
//         if (ret && hWnd && (ERROR_CANCELLED != ret && ERROR_ALREADY_EXISTS != ret))
//         {
//             ShellMessageBoxW(shell32_hInstance, hWnd, MAKEINTRESOURCEW(IDS_CREATEFOLDER_DENIED), MAKEINTRESOURCEW(IDS_CREATEFOLDER_CAPTION),
//                                     MB_ICONEXCLAMATION | MB_OK, path);
//             ret = ERROR_CANCELLED;
//         }
//     }
//
//     return ret;
// }
//
// /*************************************************************************
//  * SHFindAttrW      [internal]
//  *
//  * Get the Attributes for a file or directory. The difference to GetAttributes()
//  * is that this function will also work for paths containing wildcard characters
//  * in its filename.
//
//  * PARAMS
//  *  path       [I]   path of directory or file to check
//  *  fileOnly   [I]   TRUE if only files should be found
//  *
//  * RETURNS
//  *  INVALID_FILE_ATTRIBUTES if the path does not exist, the actual attributes of
//  *  the first file or directory found otherwise
//  */
// static DWORD SHFindAttrW(LPCWSTR pName, BOOL fileOnly)
// {
//     WIN32_FIND_DATAW wfd;
//     BOOL b_FileMask = fileOnly && (NULL != StrPBrkW(pName, L"*?"));
//     DWORD dwAttr = INVALID_FILE_ATTRIBUTES;
//     HANDLE hFind = FindFirstFileW(pName, &wfd);
//
//     TRACE("%s %d\n", debugstr_w(pName), fileOnly);
//     if (INVALID_HANDLE_VALUE != hFind)
//     {
//         do
//         {
//             if (b_FileMask && IsAttribDir(wfd.dwFileAttributes))
//                continue;
//             dwAttr = wfd.dwFileAttributes;
//             break;
//         } while (FindNextFileW(hFind, &wfd));
//
//       FindClose(hFind);
//     }
//     return dwAttr;
// }
//
// /*************************************************************************
//  *
//  * _ConvertAtoW  helper function for SHFileOperationA
//  *
//  * Converts a string or string-list to unicode.
//  */
// static DWORD _ConvertAtoW(PCSTR strSrc, PCWSTR* pStrDest, BOOL isList)
// {
//     *pStrDest = NULL;
//
//     // If the input is null, nothing to convert.
//     if (!strSrc)
//         return 0;
//
//     // Measure the total size, depending on if it's a zero-terminated list.
//     int sizeA = 0;
//     if (isList)
//     {
//         PCSTR tmpSrc = strSrc;
//         int size;
//         do
//         {
//             size = lstrlenA(tmpSrc) + 1;
//             sizeA += size;
//             tmpSrc += size;
//         } while (size != 1);
//     }
//     else
//     {
//         sizeA = lstrlenA(strSrc) + 1;
//     }
//
//     // Measure the needed allocation size.
//     int sizeW = MultiByteToWideChar(CP_ACP, 0, strSrc, sizeA, NULL, 0);
//     if (!sizeW)
//         return GetLastError();
//
//     PWSTR strDest = (PWSTR) HeapAlloc(GetProcessHeap(), 0, sizeW * sizeof(WCHAR));
//     if (!strDest)
//         return ERROR_OUTOFMEMORY;
//
//     int err = MultiByteToWideChar(CP_ACP, 0, strSrc, sizeA, strDest, sizeW);
//     if (!err)
//     {
//         HeapFree(GetProcessHeap(), 0, strDest);
//         return GetLastError();
//     }
//
//     *pStrDest = strDest;
//     return 0;
// }
//
// /*************************************************************************
//  * SHFileOperationA          [SHELL32.@]
//  *
//  * Function to copy, move, delete and create one or more files with optional
//  * user prompts.
//  *
//  * PARAMS
//  *  lpFileOp   [I/O] pointer to a structure containing all the necessary information
//  *
//  * RETURNS
//  *  Success: ERROR_SUCCESS.
//  *  Failure: ERROR_CANCELLED.
//  *
//  * NOTES
//  *  exported by name
//  */
// int WINAPI SHFileOperationA(LPSHFILEOPSTRUCTA lpFileOp)
// {
//     int errCode, retCode;
//     SHFILEOPSTRUCTW nFileOp = { 0 };
//
//     // Convert A information to W
//     nFileOp.hwnd = lpFileOp->hwnd;
//     nFileOp.wFunc = lpFileOp->wFunc;
//     nFileOp.fFlags = lpFileOp->fFlags;
//
//     errCode = _ConvertAtoW(lpFileOp->pFrom, &nFileOp.pFrom, TRUE);
//     if (errCode != 0)
//         goto cleanup;
//
//     if (FO_DELETE != (nFileOp.wFunc & FO_MASK))
//     {
//         errCode = _ConvertAtoW(lpFileOp->pTo, &nFileOp.pTo, TRUE);
//         if (errCode != 0)
//             goto cleanup;
//     }
//
//     if (nFileOp.fFlags & FOF_SIMPLEPROGRESS)
//     {
//         errCode = _ConvertAtoW(lpFileOp->lpszProgressTitle, &nFileOp.lpszProgressTitle, FALSE);
//         if (errCode != 0)
//             goto cleanup;
//     }
//
//     // Call the actual function
//     retCode = SHFileOperationW(&nFileOp);
//     if (retCode)
//     {
//         ERR("SHFileOperationW failed with 0x%x\n", retCode);
//     }
//
//     // Cleanup
// cleanup:
//     if (nFileOp.pFrom)
//         HeapFree(GetProcessHeap(), 0, (PVOID) nFileOp.pFrom);
//     if (nFileOp.pTo)
//         HeapFree(GetProcessHeap(), 0, (PVOID) nFileOp.pTo);
//     if (nFileOp.lpszProgressTitle)
//         HeapFree(GetProcessHeap(), 0, (PVOID) nFileOp.lpszProgressTitle);
//
//     if (errCode != 0)
//     {
//         lpFileOp->fAnyOperationsAborted = TRUE;
//         SetLastError(errCode);
//
//         return errCode;
//     }
//
//     // Thankfully, starting with NT4 the name mappings are always unicode, so no need to convert.
//     lpFileOp->hNameMappings = nFileOp.hNameMappings;
//     lpFileOp->fAnyOperationsAborted = nFileOp.fAnyOperationsAborted;
//     return retCode;
// }
//
// static void __inline grow_list(FILE_LIST *list)
// {
//     FILE_ENTRY *newx = (FILE_ENTRY *)HeapReAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, list->feFiles,
//                                   list->num_alloc * 2 * sizeof(*newx) );
//     list->feFiles = newx;
//     list->num_alloc *= 2;
// }
//
// /* adds a file to the FILE_ENTRY struct
//  */
// static void add_file_to_entry(FILE_ENTRY *feFile, LPCWSTR szFile)
// {
//     DWORD dwLen = lstrlenW(szFile) + 1;
//     LPCWSTR ptr;
//     LPCWSTR ptr2;
//
//     feFile->szFullPath = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, dwLen * sizeof(WCHAR));
//     lstrcpyW(feFile->szFullPath, szFile);
//
//     ptr = StrRChrW(szFile, NULL, '\\');
//     ptr2 = StrRChrW(szFile, NULL, '/');
//     if (!ptr || ptr < ptr2)
//         ptr = ptr2;
//     if (ptr)
//     {
//         dwLen = ptr - szFile + 1;
//         feFile->szDirectory = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, dwLen * sizeof(WCHAR));
//         lstrcpynW(feFile->szDirectory, szFile, dwLen);
//
//         dwLen = lstrlenW(feFile->szFullPath) - dwLen + 1;
//         feFile->szFilename = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, dwLen * sizeof(WCHAR));
//         lstrcpyW(feFile->szFilename, ptr + 1); /* skip over backslash */
//     }
//     feFile->bFromWildcard = FALSE;
// }
//
// static LPWSTR wildcard_to_file(LPCWSTR szWildCard, LPCWSTR szFileName)
// {
//     LPCWSTR ptr;
//     LPCWSTR ptr2;
//     LPWSTR szFullPath;
//     DWORD dwDirLen, dwFullLen;
//
//     ptr = StrRChrW(szWildCard, NULL, '\\');
//     ptr2 = StrRChrW(szWildCard, NULL, '/');
//     if (!ptr || ptr < ptr2)
//         ptr = ptr2;
//     dwDirLen = ptr - szWildCard + 1;
//
//     dwFullLen = dwDirLen + lstrlenW(szFileName) + 1;
//     szFullPath = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, dwFullLen * sizeof(WCHAR));
//
//     lstrcpynW(szFullPath, szWildCard, dwDirLen + 1);
//     lstrcatW(szFullPath, szFileName);
//
//     return szFullPath;
// }
//
// static void parse_wildcard_files(FILE_LIST *flList, LPCWSTR szFile, LPDWORD pdwListIndex)
// {
//     WIN32_FIND_DATAW wfd;
//     HANDLE hFile = FindFirstFileW(szFile, &wfd);
//     FILE_ENTRY *file;
//     LPWSTR szFullPath;
//     BOOL res;
//
//     if (hFile == INVALID_HANDLE_VALUE) return;
//
//     for (res = TRUE; res; res = FindNextFileW(hFile, &wfd))
//     {
//         if (IsDotDir(wfd.cFileName))
//             continue;
//
//         if (*pdwListIndex >= flList->num_alloc)
//             grow_list( flList );
//
//         szFullPath = wildcard_to_file(szFile, wfd.cFileName);
//         file = &flList->feFiles[(*pdwListIndex)++];
//         add_file_to_entry(file, szFullPath);
//         file->bFromWildcard = TRUE;
//         file->attributes = wfd.dwFileAttributes;
//
//         if (IsAttribDir(file->attributes))
//             flList->bAnyDirectories = TRUE;
//
//         HeapFree(GetProcessHeap(), 0, szFullPath);
//     }
//
//     FindClose(hFile);
// }
//
// /* takes the null-separated file list and fills out the FILE_LIST */
// static HRESULT parse_file_list(FILE_LIST *flList, LPCWSTR szFiles)
// {
//     LPCWSTR ptr = szFiles;
//     WCHAR szCurFile[MAX_PATH];
//     DWORD i = 0;
//
//     if (!szFiles)
//         return ERROR_INVALID_PARAMETER;
//
//     flList->bAnyFromWildcard = FALSE;
//     flList->bAnyDirectories = FALSE;
//     flList->bAnyDontExist = FALSE;
//     flList->num_alloc = 32;
//     flList->dwNumFiles = 0;
//
//     /* empty list */
//     if (!szFiles[0])
//         return ERROR_ACCESS_DENIED;
//
//     flList->feFiles = (FILE_ENTRY *)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//                                 flList->num_alloc * sizeof(FILE_ENTRY));
//
//     while (*ptr)
//     {
//         if (i >= flList->num_alloc) grow_list( flList );
//
//         /* change relative to absolute path */
//         if (PathIsRelativeW(ptr))
//         {
//             GetCurrentDirectoryW(MAX_PATH, szCurFile);
//             PathCombineW(szCurFile, szCurFile, ptr);
//             flList->feFiles[i].bFromRelative = TRUE;
//         }
//         else
//         {
//             lstrcpyW(szCurFile, ptr);
//             flList->feFiles[i].bFromRelative = FALSE;
//         }
//
//         /* parse wildcard files if they are in the filename */
//         if (StrPBrkW(szCurFile, L"*?"))
//         {
//             parse_wildcard_files(flList, szCurFile, &i);
//             flList->bAnyFromWildcard = TRUE;
//             i--;
//         }
//         else
//         {
//             FILE_ENTRY *file = &flList->feFiles[i];
//             add_file_to_entry(file, szCurFile);
//             file->attributes = GetFileAttributesW( file->szFullPath );
//             file->bExists = (file->attributes != INVALID_FILE_ATTRIBUTES);
//
//             if (!file->bExists)
//                 flList->bAnyDontExist = TRUE;
//
//             if (IsAttribDir(file->attributes))
//                 flList->bAnyDirectories = TRUE;
//         }
//
//         /* advance to the next string */
//         ptr += lstrlenW(ptr) + 1;
//         i++;
//     }
//     flList->dwNumFiles = i;
//
//     return S_OK;
// }
//
// /* free the FILE_LIST */
// static void destroy_file_list(FILE_LIST *flList)
// {
//     DWORD i;
//
//     if (!flList || !flList->feFiles)
//         return;
//
//     for (i = 0; i < flList->dwNumFiles; i++)
//     {
//         HeapFree(GetProcessHeap(), 0, flList->feFiles[i].szDirectory);
//         HeapFree(GetProcessHeap(), 0, flList->feFiles[i].szFilename);
//         HeapFree(GetProcessHeap(), 0, flList->feFiles[i].szFullPath);
//     }
//
//     HeapFree(GetProcessHeap(), 0, flList->feFiles);
// }
//
// static CStringW try_find_new_name(LPCWSTR szDestPath)
// {
//     CStringW mask(szDestPath);
//     CStringW ext(PathFindExtensionW(szDestPath));
//
//     // cut off extension before inserting a "new file" mask
//     if (!ext.IsEmpty())
//     {
//         mask = mask.Left(mask.GetLength() - ext.GetLength());
//     }
//     mask += L" (%d)" + ext;
//
//     CStringW newName;
//
//     // trying to find new file name
//     for (int i = 1; i < NEW_FILENAME_ON_COPY_TRIES; i++)
//     {
//         newName.Format(mask, i);
//
//         if (!PathFileExistsW(newName))
//         {
//             return newName;
//         }
//     }
//
//     return CStringW();
// }
//
// static void copy_dir_to_dir(FILE_OPERATION *op, const FILE_ENTRY *feFrom, LPCWSTR szDestPath)
// {
//     WCHAR szFrom[MAX_PATH], szTo[MAX_PATH];
//     FILE_LIST flFromNew, flToNew;
//
//     if (feFrom->szFilename && IsDotDir(feFrom->szFilename))
//         return;
//
//     if (PathFileExistsW(szDestPath))
//         PathCombineW(szTo, szDestPath, feFrom->szFilename);
//     else
//         lstrcpyW(szTo, szDestPath);
//
//     if (PathFileExistsW(szTo))
//     {
//         if (op->req->fFlags & FOF_RENAMEONCOLLISION)
//         {
//             CStringW newPath = try_find_new_name(szTo);
//             if (!newPath.IsEmpty())
//             {
//                 StringCchCopyW(szTo, _countof(szTo), newPath);
//             }
//         }
//         else if (!(op->req->fFlags & FOF_NOCONFIRMATION))
//         {
//             if (!SHELL_ConfirmDialogW(op->hWndOwner, ASK_OVERWRITE_FOLDER, feFrom->szFilename, op))
//             {
//                 /* Vista returns an ERROR_CANCELLED even if user pressed "No" */
//                 if (!op->bManyItems)
//                     op->bCancelled = TRUE;
//                 return;
//             }
//         }
//     }
//
//     szTo[lstrlenW(szTo) + 1] = '\0';
//     SHNotifyCreateDirectoryW(szTo, NULL);
//
//     PathCombineW(szFrom, feFrom->szFullPath, L"*.*");
//     szFrom[lstrlenW(szFrom) + 1] = '\0';
//
//     ZeroMemory(&flFromNew, sizeof(FILE_LIST));
//     ZeroMemory(&flToNew, sizeof(FILE_LIST));
//     parse_file_list(&flFromNew, szFrom);
//     parse_file_list(&flToNew, szTo);
//
//     copy_files(op, FALSE, &flFromNew, &flToNew);
//
//     destroy_file_list(&flFromNew);
//     destroy_file_list(&flToNew);
// }
//
// static BOOL copy_file_to_file(FILE_OPERATION *op, const WCHAR *szFrom, const WCHAR *szTo)
// {
//     if (PathFileExistsW(szTo))
//     {
//         if (op->req->fFlags & FOF_RENAMEONCOLLISION)
//         {
//             CStringW newPath = try_find_new_name(szTo);
//             if (!newPath.IsEmpty())
//             {
//                 return SHNotifyCopyFileW(op, szFrom, newPath, FALSE) == 0;
//             }
//         }
//         else if (!(op->req->fFlags & FOF_NOCONFIRMATION))
//         {
//             if (!SHELL_ConfirmDialogW(op->hWndOwner, ASK_OVERWRITE_FILE, PathFindFileNameW(szTo), op))
//                 return FALSE;
//         }
//     }
//
//     return SHNotifyCopyFileW(op, szFrom, szTo, FALSE) == 0;
// }
//
// /* copy a file or directory to another directory */
// static void copy_to_dir(FILE_OPERATION *op, const FILE_ENTRY *feFrom, const FILE_ENTRY *feTo)
// {
//     if (!PathFileExistsW(feTo->szFullPath))
//         SHNotifyCreateDirectoryW(feTo->szFullPath, NULL);
//
//     if (IsAttribFile(feFrom->attributes))
//     {
//         WCHAR szDestPath[MAX_PATH];
//
//         PathCombineW(szDestPath, feTo->szFullPath, feFrom->szFilename);
//         copy_file_to_file(op, feFrom->szFullPath, szDestPath);
//     }
//     else if (!(op->req->fFlags & FOF_FILESONLY && feFrom->bFromWildcard))
//         copy_dir_to_dir(op, feFrom, feTo->szFullPath);
// }
//
// static void create_dest_dirs(LPCWSTR szDestDir)
// {
//     WCHAR dir[MAX_PATH];
//     LPCWSTR ptr = StrChrW(szDestDir, '\\');
//
//     /* make sure all directories up to last one are created */
//     while (ptr && (ptr = StrChrW(ptr + 1, '\\')))
//     {
//         lstrcpynW(dir, szDestDir, ptr - szDestDir + 1);
//
//         if (!PathFileExistsW(dir))
//             SHNotifyCreateDirectoryW(dir, NULL);
//     }
//
//     /* create last directory */
//     if (!PathFileExistsW(szDestDir))
//         SHNotifyCreateDirectoryW(szDestDir, NULL);
// }
//
// /* the FO_COPY operation */
// static HRESULT copy_files(FILE_OPERATION *op, BOOL multiDest, const FILE_LIST *flFrom, FILE_LIST *flTo)
// {
//     DWORD i;
//     const FILE_ENTRY *entryToCopy;
//     const FILE_ENTRY *fileDest = &flTo->feFiles[0];
//
//     if (flFrom->bAnyDontExist)
//         return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
//
//     if (flTo->dwNumFiles == 0)
//     {
//         /* If the destination is empty, SHFileOperation should use the current directory */
//         WCHAR curdir[MAX_PATH+1];
//
//         GetCurrentDirectoryW(MAX_PATH, curdir);
//         curdir[lstrlenW(curdir)+1] = 0;
//
//         destroy_file_list(flTo);
//         ZeroMemory(flTo, sizeof(FILE_LIST));
//         parse_file_list(flTo, curdir);
//         fileDest = &flTo->feFiles[0];
//     }
//
//     if (multiDest)
//     {
//         if (flFrom->bAnyFromWildcard)
//             return ERROR_CANCELLED;
//
//         if (flFrom->dwNumFiles != flTo->dwNumFiles)
//         {
//             if (flFrom->dwNumFiles != 1 && !IsAttribDir(fileDest->attributes))
//                 return ERROR_CANCELLED;
//
//             /* Free all but the first entry. */
//             for (i = 1; i < flTo->dwNumFiles; i++)
//             {
//                 HeapFree(GetProcessHeap(), 0, flTo->feFiles[i].szDirectory);
//                 HeapFree(GetProcessHeap(), 0, flTo->feFiles[i].szFilename);
//                 HeapFree(GetProcessHeap(), 0, flTo->feFiles[i].szFullPath);
//             }
//
//             flTo->dwNumFiles = 1;
//         }
//         else if (IsAttribDir(fileDest->attributes))
//         {
//             for (i = 1; i < flTo->dwNumFiles; i++)
//                 if (!IsAttribDir(flTo->feFiles[i].attributes) ||
//                     !IsAttribDir(flFrom->feFiles[i].attributes))
//                 {
//                     return ERROR_CANCELLED;
//                 }
//         }
//     }
//     else if (flFrom->dwNumFiles != 1)
//     {
//         if (flTo->dwNumFiles != 1 && !IsAttribDir(fileDest->attributes))
//             return ERROR_CANCELLED;
//
//         if (PathFileExistsW(fileDest->szFullPath) &&
//             IsAttribFile(fileDest->attributes))
//         {
//             return ERROR_CANCELLED;
//         }
//
//         if (flTo->dwNumFiles == 1 && fileDest->bFromRelative &&
//             !PathFileExistsW(fileDest->szFullPath))
//         {
//             return ERROR_CANCELLED;
//         }
//     }
//
//     for (i = 0; i < flFrom->dwNumFiles; i++)
//     {
//         entryToCopy = &flFrom->feFiles[i];
//
//         if ((multiDest) &&
//             flTo->dwNumFiles > 1)
//         {
//             fileDest = &flTo->feFiles[i];
//         }
//
//         if (IsAttribDir(entryToCopy->attributes) &&
//             !lstrcmpiW(entryToCopy->szFullPath, fileDest->szDirectory))
//         {
//             return ERROR_SUCCESS;
//         }
//
//         create_dest_dirs(fileDest->szDirectory);
//
//         if (!lstrcmpiW(entryToCopy->szFullPath, fileDest->szFullPath))
//         {
//             if (IsAttribFile(entryToCopy->attributes))
//                 return ERROR_NO_MORE_SEARCH_HANDLES;
//             else
//                 return ERROR_SUCCESS;
//         }
//
//         if ((flFrom->dwNumFiles > 1 && flTo->dwNumFiles == 1) ||
//             IsAttribDir(fileDest->attributes))
//         {
//             copy_to_dir(op, entryToCopy, fileDest);
//         }
//         else if (IsAttribDir(entryToCopy->attributes))
//         {
//             copy_dir_to_dir(op, entryToCopy, fileDest->szFullPath);
//         }
//         else
//         {
//             if (!copy_file_to_file(op, entryToCopy->szFullPath, fileDest->szFullPath))
//             {
//                 op->req->fAnyOperationsAborted = TRUE;
//                 return ERROR_CANCELLED;
//             }
//         }
//
//         if (op->progress != NULL)
//             op->bCancelled |= op->progress->HasUserCancelled();
//         /* Vista return code. XP would return e.g. ERROR_FILE_NOT_FOUND, ERROR_ALREADY_EXISTS */
//         if (op->bCancelled)
//             return ERROR_CANCELLED;
//     }
//
//     /* Vista return code. On XP if the used pressed "No" for the last item,
//      * ERROR_ARENA_TRASHED would be returned */
//     return ERROR_SUCCESS;
// }
//
// static BOOL confirm_delete_list(HWND hWnd, DWORD fFlags, BOOL fTrash, const FILE_LIST *flFrom)
// {
//     if (flFrom->dwNumFiles > 1)
//     {
//         WCHAR tmp[8];
//
//         wnsprintfW(tmp, sizeof(tmp)/sizeof(tmp[0]), L"%d", flFrom->dwNumFiles);
//         return SHELL_ConfirmDialogW(hWnd, (fTrash?ASK_TRASH_MULTIPLE_ITEM:ASK_DELETE_MULTIPLE_ITEM), tmp, NULL);
//     }
//     else
//     {
//         const FILE_ENTRY *fileEntry = &flFrom->feFiles[0];
//
//         if (IsAttribFile(fileEntry->attributes))
//             return SHELL_ConfirmDialogW(hWnd, (fTrash?ASK_TRASH_FILE:ASK_DELETE_FILE), fileEntry->szFullPath, NULL);
//         else if (!(fFlags & FOF_FILESONLY && fileEntry->bFromWildcard))
//             return SHELL_ConfirmDialogW(hWnd, (fTrash?ASK_TRASH_FOLDER:ASK_DELETE_FOLDER), fileEntry->szFullPath, NULL);
//     }
//     return TRUE;
// }
//
// /* the FO_DELETE operation */
// static HRESULT delete_files(FILE_OPERATION *op, const FILE_LIST *flFrom)
// {
//     const FILE_ENTRY *fileEntry;
//     DWORD i;
//     BOOL bPathExists;
//     BOOL bTrash;
//
//     if (!flFrom->dwNumFiles)
//         return ERROR_SUCCESS;
//
//     /* Windows also checks only the first item */
//     bTrash = (op->req->fFlags & FOF_ALLOWUNDO)
//         && TRASH_CanTrashFile(flFrom->feFiles[0].szFullPath);
//
//     BOOL confirm = !(op->req->fFlags & FOF_NOCONFIRMATION);
//     if (bTrash && SHELL_GetSetting(SSF_NOCONFIRMRECYCLE, fNoConfirmRecycle))
//         confirm = FALSE;
//     if (confirm || (!bTrash && op->req->fFlags & FOF_WANTNUKEWARNING))
//         if (!confirm_delete_list(op->hWndOwner, op->req->fFlags, bTrash, flFrom))
//         {
//             op->req->fAnyOperationsAborted = TRUE;
//             return 0;
//         }
//
//     /* Check files. Do not delete one if one file does not exists */
//     for (i = 0; i < flFrom->dwNumFiles; i++)
//     {
//         fileEntry = &flFrom->feFiles[i];
//
//         if (fileEntry->attributes == (ULONG)-1)
//         {
//             // This is a windows 2003 server specific value which has been removed.
//             // Later versions of windows return ERROR_FILE_NOT_FOUND.
//             return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
//         }
//     }
//
//     for (i = 0; i < flFrom->dwNumFiles; i++)
//     {
//         bPathExists = TRUE;
//         fileEntry = &flFrom->feFiles[i];
//
//         if (!IsAttribFile(fileEntry->attributes) &&
//             (op->req->fFlags & FOF_FILESONLY && fileEntry->bFromWildcard))
//             continue;
//
//         if (bTrash)
//         {
//             BOOL bDelete;
//             if (TRASH_TrashFile(fileEntry->szFullPath))
//             {
//                 UINT event = IsAttribFile(fileEntry->attributes) ? SHCNE_DELETE : SHCNE_RMDIR;
//                 SHChangeNotify(event, SHCNF_PATHW, fileEntry->szFullPath, NULL);
//                 continue;
//             }
//
//             /* Note: Windows silently deletes the file in such a situation, we show a dialog */
//             if (!(op->req->fFlags & FOF_NOCONFIRMATION) || (op->req->fFlags & FOF_WANTNUKEWARNING))
//                 bDelete = SHELL_ConfirmDialogW(op->hWndOwner, ASK_CANT_TRASH_ITEM, fileEntry->szFullPath, NULL);
//             else
//                 bDelete = TRUE;
//
//             if (!bDelete)
//             {
//                 op->req->fAnyOperationsAborted = TRUE;
//                 break;
//             }
//         }
//
//         /* delete the file or directory */
//         if (IsAttribFile(fileEntry->attributes))
//             bPathExists = (ERROR_SUCCESS == SHNotifyDeleteFileW(op, fileEntry->szFullPath));
//         else
//             bPathExists = SHELL_DeleteDirectoryW(op, fileEntry->szFullPath, FALSE);
//
//         if (!bPathExists)
//         {
//             DWORD err = GetLastError();
//
//             if (ERROR_FILE_NOT_FOUND == err)
//             {
//                 // This is a windows 2003 server specific value which ahs been removed.
//                 // Later versions of windows return ERROR_FILE_NOT_FOUND.
//                 return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
//             }
//             else
//             {
//                 return err;
//             }
//         }
//
//         if (op->progress != NULL)
//             op->bCancelled |= op->progress->HasUserCancelled();
//         /* Should fire on progress dialog only */
//         if (op->bCancelled)
//             return ERROR_CANCELLED;
//     }
//
//     return ERROR_SUCCESS;
// }
//
// static void move_dir_to_dir(FILE_OPERATION *op, const FILE_ENTRY *feFrom, LPCWSTR szDestPath)
// {
//     WCHAR szFrom[MAX_PATH], szTo[MAX_PATH];
//     FILE_LIST flFromNew, flToNew;
//
//     if (feFrom->szFilename && IsDotDir(feFrom->szFilename))
//         return;
//
//     UINT attrib = FILE_ATTRIBUTE_DIRECTORY;
//     if (FAILED(FileOpCallback(op, FOCE_PREMOVEITEM, feFrom->szFullPath, szDestPath, attrib)))
//         return;
//
//     SHNotifyCreateDirectoryW(szDestPath, NULL);
//
//     PathCombineW(szFrom, feFrom->szFullPath, L"*.*");
//     szFrom[lstrlenW(szFrom) + 1] = '\0';
//
//     lstrcpyW(szTo, szDestPath);
//     szTo[lstrlenW(szDestPath) + 1] = '\0';
//
//     ZeroMemory(&flFromNew, sizeof(FILE_LIST));
//     ZeroMemory(&flToNew, sizeof(FILE_LIST));
//     parse_file_list(&flFromNew, szFrom);
//     parse_file_list(&flToNew, szTo);
//
//     move_files(op, FALSE, &flFromNew, &flToNew);
//
//     destroy_file_list(&flFromNew);
//     destroy_file_list(&flToNew);
//
//     BOOL success = FALSE;
//     if (PathIsDirectoryEmptyW(feFrom->szFullPath))
//         success = Win32RemoveDirectoryW(feFrom->szFullPath);
//     FileOpCallback(op, FOCE_POSTMOVEITEM, feFrom->szFullPath, szDestPath, attrib, success ? S_OK : E_FAIL);
// }
//
// static BOOL move_file_to_file(FILE_OPERATION *op, const WCHAR *szFrom, const WCHAR *szTo)
// {
//     if (PathFileExistsW(szTo))
//     {
//         if (op->req->fFlags & FOF_RENAMEONCOLLISION)
//         {
//             CStringW newPath = try_find_new_name(szTo);
//             if (!newPath.IsEmpty())
//             {
//                 return SHNotifyMoveFileW(op, szFrom, newPath, FALSE) == 0;
//             }
//         }
//         else if (!(op->req->fFlags & FOF_NOCONFIRMATION))
//         {
//             if (!SHELL_ConfirmDialogW(op->hWndOwner, ASK_OVERWRITE_FILE, PathFindFileNameW(szTo), op))
//                 return FALSE;
//         }
//     }
//
//     return SHNotifyMoveFileW(op, szFrom, szTo, FALSE) == 0;
// }
//
// /* moves a file or directory to another directory */
// static void move_to_dir(FILE_OPERATION *op, const FILE_ENTRY *feFrom, const FILE_ENTRY *feTo)
// {
//     if (feFrom->attributes == INVALID_FILE_ATTRIBUTES)
//         return;
//
//     if (!PathFileExistsW(feTo->szFullPath))
//         SHNotifyCreateDirectoryW(feTo->szFullPath, NULL);
//
//     WCHAR szDestPath[MAX_PATH];
//     PathCombineW(szDestPath, feTo->szFullPath, feFrom->szFilename);
//
//     if (IsAttribFile(feFrom->attributes))
//         move_file_to_file(op, feFrom->szFullPath, szDestPath);
//     else if (!(op->req->fFlags & FOF_FILESONLY && feFrom->bFromWildcard))
//         move_dir_to_dir(op, feFrom, szDestPath);
// }
//
// /* the FO_MOVE operation */
// static DWORD move_files(FILE_OPERATION *op, BOOL multiDest, const FILE_LIST *flFrom, const FILE_LIST *flTo)
// {
//     DWORD i;
//     INT mismatched = 0;
//
//     const FILE_ENTRY *entryToMove;
//     const FILE_ENTRY *fileDest;
//
//     if (!flFrom->dwNumFiles)
//         return ERROR_SUCCESS;
//
//     if (!flTo->dwNumFiles)
//         return ERROR_FILE_NOT_FOUND;
//
//     if (flFrom->bAnyDontExist)
//         return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
//
//     if (!(multiDest) &&
//         flTo->dwNumFiles > 1 && flFrom->dwNumFiles > 1)
//     {
//         return ERROR_CANCELLED;
//     }
//
//     if (!(multiDest) &&
//         !flFrom->bAnyDirectories &&
//         flFrom->dwNumFiles > flTo->dwNumFiles &&
//         !(flTo->bAnyDirectories && flTo->dwNumFiles == 1))
//     {
//         return ERROR_CANCELLED;
//     }
//
//     if (!PathFileExistsW(flTo->feFiles[0].szDirectory))
//         return ERROR_CANCELLED;
//
//     if (multiDest)
//         mismatched = flFrom->dwNumFiles - flTo->dwNumFiles;
//
//     fileDest = &flTo->feFiles[0];
//     for (i = 0; i < flFrom->dwNumFiles; i++)
//     {
//         entryToMove = &flFrom->feFiles[i];
//
//         if (!PathFileExistsW(fileDest->szDirectory))
//             return ERROR_CANCELLED;
//
//         if (multiDest)
//         {
//             if (i >= flTo->dwNumFiles)
//                 break;
//             fileDest = &flTo->feFiles[i];
//             if (mismatched && !fileDest->bExists)
//             {
//                 create_dest_dirs(flTo->feFiles[i].szFullPath);
//                 flTo->feFiles[i].bExists = TRUE;
//                 flTo->feFiles[i].attributes = FILE_ATTRIBUTE_DIRECTORY;
//             }
//         }
//
//         if ((flFrom->dwNumFiles > 1 && flTo->dwNumFiles == 1) ||
//             IsAttribDir(fileDest->attributes))
//         {
//             move_to_dir(op, entryToMove, fileDest);
//         }
//         else if (IsAttribDir(entryToMove->attributes))
//         {
//             move_dir_to_dir(op, entryToMove, fileDest->szFullPath);
//         }
//         else
//         {
//             if (!move_file_to_file(op, entryToMove->szFullPath, fileDest->szFullPath))
//             {
//                 op->req->fAnyOperationsAborted = TRUE;
//                 return ERROR_CANCELLED;
//             }
//         }
//
//         if (op->progress != NULL)
//             op->bCancelled |= op->progress->HasUserCancelled();
//         /* Should fire on progress dialog only */
//         if (op->bCancelled)
//             return ERROR_CANCELLED;
//
//     }
//
//     if (mismatched > 0)
//     {
//         if (flFrom->bAnyDirectories)
//             return DE_DESTSAMETREE;
//         else
//             return DE_SAMEFILE;
//     }
//
//     return ERROR_SUCCESS;
// }
//
// /* the FO_RENAME files */
// static HRESULT rename_files(FILE_OPERATION *op,  const FILE_LIST *flFrom, const FILE_LIST *flTo)
// {
//     const FILE_ENTRY *feFrom;
//     const FILE_ENTRY *feTo;
//
//     if (flFrom->dwNumFiles != 1)
//         return ERROR_GEN_FAILURE;
//
//     if (flTo->dwNumFiles != 1)
//         return ERROR_CANCELLED;
//
//     feFrom = &flFrom->feFiles[0];
//     feTo= &flTo->feFiles[0];
//
//     /* fail if destination doesn't exist */
//     if (!feFrom->bExists)
//         return ERROR_SHELL_INTERNAL_FILE_NOT_FOUND;
//
//     /* fail if destination already exists */
//     if (feTo->bExists)
//         return ERROR_ALREADY_EXISTS;
//
//     return SHNotifyMoveFileW(op, feFrom->szFullPath, feTo->szFullPath, IsAttribDir(feFrom->attributes));
// }
//
// /* alert the user if an unsupported flag is used */
// static void check_flags(FILEOP_FLAGS fFlags)
// {
//     WORD wUnsupportedFlags = FOF_NO_CONNECTED_ELEMENTS |
//         FOF_NOCOPYSECURITYATTRIBS | FOF_NORECURSEREPARSE |
//         FOF_WANTMAPPINGHANDLE;
//
//     if (fFlags & wUnsupportedFlags)
//         FIXME("Unsupported flags: %04x\n", fFlags);
// }
//
// #define GET_FILENAME(fe) ((fe)->szFilename[0] ? (fe)->szFilename : (fe)->szFullPath)
//
// static DWORD
// validate_operation(FILE_OPERATION &op, FILE_LIST *flFrom, FILE_LIST *flTo)
// {
//     DWORD i, k, dwNumDest;
//     WCHAR szFrom[MAX_PATH], szTo[MAX_PATH];
//     CStringW strTitle, strText;
//     const FILE_ENTRY *feFrom;
//     const FILE_ENTRY *feTo;
//     const SHFILEOPSTRUCTW *lpFileOp = op.req;
//     UINT wFunc = lpFileOp->wFunc;
//
//     dwNumDest = flTo->dwNumFiles;
//
//     if (wFunc != FO_COPY && wFunc != FO_MOVE)
//         return ERROR_SUCCESS;
//
//     for (k = 0; k < dwNumDest; ++k)
//     {
//         feTo = &flTo->feFiles[k];
//         for (i = 0; i < flFrom->dwNumFiles; ++i)
//         {
//             feFrom = &flFrom->feFiles[i];
//             StringCbCopyW(szFrom, sizeof(szFrom), feFrom->szFullPath);
//             StringCbCopyW(szTo, sizeof(szTo), feTo->szFullPath);
//             if (IsAttribDir(feTo->attributes))
//             {
//                 PathAppendW(szTo, feFrom->szFilename);
//             }
//
//             // same path?
//             if (lstrcmpiW(szFrom, szTo) == 0 &&
//                 (wFunc == FO_MOVE || !(lpFileOp->fFlags & FOF_RENAMEONCOLLISION)))
//             {
//                 if (CanShowFileOpErrorUI(*lpFileOp))
//                 {
//                     if (wFunc == FO_MOVE)
//                     {
//                         strTitle.LoadStringW(IDS_MOVEERRORTITLE);
//                         if (IsAttribDir(feFrom->attributes))
//                             strText.Format(IDS_MOVEERRORSAMEFOLDER, GET_FILENAME(feFrom));
//                         else
//                             strText.Format(IDS_MOVEERRORSAME, GET_FILENAME(feFrom));
//                     }
//                     else
//                     {
//                         strTitle.LoadStringW(IDS_COPYERRORTITLE);
//                         strText.Format(IDS_COPYERRORSAME, GET_FILENAME(feFrom));
//                         return ERROR_SUCCESS;
//                     }
//                     MessageBoxW(op.hWndOwner, strText, strTitle, MB_ICONERROR);
//                     op.bHasDisplayedError++;
//                     return DE_SAMEFILE;
//                 }
//                 return DE_OPCANCELLED;
//             }
//
//             // subfolder?
//             if (IsAttribDir(feFrom->attributes))
//             {
//                 size_t cchFrom = PathAddBackslashW(szFrom) - szFrom;
//                 size_t cchTo = PathAddBackslashW(szTo) - szTo;
//                 if (cchFrom < cchTo)
//                 {
//                     WCHAR ch = szTo[cchFrom];
//                     szTo[cchFrom] = 0;
//                     int compare = lstrcmpiW(szFrom, szTo);
//                     szTo[cchFrom] = ch;
//
//                     if (compare == 0)
//                     {
//                         if (CanShowFileOpErrorUI(*lpFileOp))
//                         {
//                             if (wFunc == FO_MOVE)
//                             {
//                                 strTitle.LoadStringW(IDS_MOVEERRORTITLE);
//                                 strText.Format(IDS_MOVEERRORSUBFOLDER, GET_FILENAME(feFrom));
//                             }
//                             else
//                             {
//                                 strTitle.LoadStringW(IDS_COPYERRORTITLE);
//                                 strText.Format(IDS_COPYERRORSUBFOLDER, GET_FILENAME(feFrom));
//                             }
//                             MessageBoxW(op.hWndOwner, strText, strTitle, MB_ICONERROR);
//                             op.bHasDisplayedError++;
//                             return DE_DESTSUBTREE;
//                         }
//                         return DE_OPCANCELLED;
//                     }
//                 }
//             }
//         }
//     }
//
//     return ERROR_SUCCESS;
// }
//
// int SHELL32_FileOperation(LPSHFILEOPSTRUCTW lpFileOp, FILEOPCALLBACK Callback, void *CallerData)
// {
//     FILE_OPERATION op;
//     FILE_LIST flFrom, flTo;
//     int ret = 0;
//
//     if (!lpFileOp)
//         return ERROR_INVALID_PARAMETER;
//
//     ret = CoInitialize(NULL);
//     if (FAILED(ret))
//         return ret;
//
//     lpFileOp->fAnyOperationsAborted = FALSE;
//     check_flags(lpFileOp->fFlags);
//
//     ZeroMemory(&flFrom, sizeof(FILE_LIST));
//     ZeroMemory(&flTo, sizeof(FILE_LIST));
//
//     if ((ret = parse_file_list(&flFrom, lpFileOp->pFrom)))
//         return ret;
//
//     if (lpFileOp->wFunc != FO_DELETE)
//         parse_file_list(&flTo, lpFileOp->pTo);
//
//     ZeroMemory(&op, sizeof(op));
//     op.req = lpFileOp;
//     op.totalSize.QuadPart = 0ull;
//     op.completedSize.QuadPart = 0ull;
//     op.bManyItems = (flFrom.dwNumFiles > 1);
//     op.Callback = Callback;
//     op.CallerCallbackData = CallerData;
//     op.hWndOwner = GetAncestor(lpFileOp->hwnd, GA_ROOT);
//
//     ret = validate_operation(op, &flFrom, &flTo);
//     if (ret)
//         goto cleanup;
//
//     if (lpFileOp->wFunc != FO_RENAME && !(lpFileOp->fFlags & FOF_SILENT)) {
//         ret = CoCreateInstance(CLSID_ProgressDialog,
//                                NULL,
//                                CLSCTX_INPROC_SERVER,
//                                IID_PPV_ARG(IProgressDialog, &op.progress));
//         if (FAILED(ret))
//             goto cleanup;
//
//         op.progress->StartProgressDialog(op.hWndOwner, NULL, PROGDLG_NORMAL & PROGDLG_AUTOTIME, NULL);
//         _SetOperationTitle(&op);
//         _FileOpCountManager(&op, &flFrom);
//     }
//
//     FileOpCallback(&op, FOCE_STARTOPERATIONS, NULL, NULL, 0);
//
//     switch (lpFileOp->wFunc)
//     {
//         case FO_COPY:
//             ret = copy_files(&op, op.req->fFlags & FOF_MULTIDESTFILES, &flFrom, &flTo);
//             break;
//         case FO_DELETE:
//             ret = delete_files(&op, &flFrom);
//             break;
//         case FO_MOVE:
//             ret = move_files(&op, op.req->fFlags & FOF_MULTIDESTFILES, &flFrom, &flTo);
//             break;
//         case FO_RENAME:
//             ret = rename_files(&op, &flFrom, &flTo);
//             break;
//         default:
//             ret = ERROR_INVALID_PARAMETER;
//             break;
//     }
//
//     if (op.progress) {
//         op.progress->StopProgressDialog();
//         op.progress->Release();
//     }
//
// cleanup:
//     destroy_file_list(&flFrom);
//
//     if (lpFileOp->wFunc != FO_DELETE)
//         destroy_file_list(&flTo);
//     else if (lpFileOp->fFlags & FOF_ALLOWUNDO)
//         SHUpdateRecycleBinIcon();
//
//     if (ret == ERROR_CANCELLED)
//         lpFileOp->fAnyOperationsAborted = TRUE;
//     if (ret == ERROR_SHELL_INTERNAL_FILE_NOT_FOUND && LOBYTE(GetVersion()) >= 6)
//         ret = ERROR_FILE_NOT_FOUND;
//
//     HRESULT hr = SHELL32_FileOpErrorToHResult(ret);
//     TRACE("SHFO FINISHOPERATIONS %#x (%d)\n", hr, ret);
//     FileOpCallback(&op, FOCE_FINISHOPERATIONS, NULL, NULL, 0, hr);
//
//     if (FAILED(hr) && CanShowFileOpErrorUI(*op.req) && !op.bHasDisplayedError)
//         SHELL_ErrorBox(op.hWndOwner, hr);
//
//     CoUninitialize();
//
//     return ret;
// }
//
// /*************************************************************************
//  * SHFileOperationW          [SHELL32.@]
//  *
//  * See SHFileOperationA
//  */
// int WINAPI SHFileOperationW(LPSHFILEOPSTRUCTW lpFileOp)
// {
//     return SHELL32_FileOperation(lpFileOp, NULL, NULL);
// }
//
// // Used by SHFreeNameMappings
// static int __stdcall _DestroyCallback(void *p, void *pData)
// {
//     LPSHNAMEMAPPINGW lp = (SHNAMEMAPPINGW *)p;
//
//     SHFree(lp->pszOldPath);
//     SHFree(lp->pszNewPath);
//
//     return TRUE;
// }
//
// /*************************************************************************
//  * SHFreeNameMappings      [shell32.246]
//  *
//  * Free the mapping handle returned by SHFileOperation if FOF_WANTSMAPPINGHANDLE
//  * was specified.
//  *
//  * PARAMS
//  *  hNameMapping [I] handle to the name mappings used during renaming of files
//  *
//  * RETURNS
//  *  Nothing
//  */
// void WINAPI SHFreeNameMappings(HANDLE hNameMapping)
// {
//     if (hNameMapping)
//     {
//         DSA_DestroyCallback((HDSA) hNameMapping, _DestroyCallback, NULL);
//     }
// }
//
// /*************************************************************************
//  * SheGetDirA [SHELL32.@]
//  *
//  * drive = 0: returns the current directory path
//  * drive > 0: returns the current directory path of the specified drive
//  *            drive=1 -> A:  drive=2 -> B:  ...
//  * returns 0 if successful
// */
// extern "C" DWORD WINAPI SheGetDirA(DWORD drive, LPSTR buffer)
// {
//     WCHAR org_path[MAX_PATH];
//     DWORD ret;
//     char drv_path[3];
//
//     /* change current directory to the specified drive */
//     if (drive) {
//         strcpy(drv_path, "A:");
//         drv_path[0] += (char)drive-1;
//
//         GetCurrentDirectoryW(MAX_PATH, org_path);
//
//         SetCurrentDirectoryA(drv_path);
//     }
//
//     /* query current directory path of the specified drive */
//     ret = GetCurrentDirectoryA(MAX_PATH, buffer);
//
//     /* back to the original drive */
//     if (drive)
//         SetCurrentDirectoryW(org_path);
//
//     if (!ret)
//         return GetLastError();
//
//     return 0;
// }
//
// /*************************************************************************
//  * SheGetDirW [SHELL32.@]
//  *
//  * drive = 0: returns the current directory path
//  * drive > 0: returns the current directory path of the specified drive
//  *            drive=1 -> A:  drive=2 -> B:  ...
//  * returns 0 if successful
//  */
// extern "C" DWORD WINAPI SheGetDirW(DWORD drive, LPWSTR buffer)
// {
//     WCHAR org_path[MAX_PATH];
//     DWORD ret;
//     char drv_path[3];
//
//     /* change current directory to the specified drive */
//     if (drive)
//     {
//         strcpy(drv_path, "A:");
//         drv_path[0] += (char)drive-1;
//
//         GetCurrentDirectoryW(MAX_PATH, org_path);
//
//         SetCurrentDirectoryA(drv_path);
//     }
//
//     /* query current directory path of the specified drive */
//     ret = GetCurrentDirectoryW(MAX_PATH, buffer);
//
//     /* back to the original drive */
//     if (drive)
//         SetCurrentDirectoryW(org_path);
//
//     if (!ret)
//         return GetLastError();
//
//     return 0;
// }
//
// /*************************************************************************
//  * SheChangeDirA [SHELL32.@]
//  *
//  * changes the current directory to the specified path
//  * and returns 0 if successful
//  */
// extern "C" DWORD WINAPI SheChangeDirA(LPSTR path)
// {
//     if (SetCurrentDirectoryA(path))
//         return 0;
//     else
//         return GetLastError();
// }
//
// /*************************************************************************
//  * SheChangeDirW [SHELL32.@]
//  *
//  * changes the current directory to the specified path
//  * and returns 0 if successful
//  */
// extern "C" DWORD WINAPI SheChangeDirW(LPWSTR path)
// {
//     if (SetCurrentDirectoryW(path))
//         return 0;
//     else
//         return GetLastError();
// }
//
// /*************************************************************************
//  * IsNetDrive            [SHELL32.66]
//  */
// extern "C" int WINAPI IsNetDrive(int drive)
// {
//     char root[4];
//     strcpy(root, "A:\\");
//     root[0] += (char)drive;
//     return (GetDriveTypeA(root) == DRIVE_REMOTE);
// }
//
//
// /*************************************************************************
//  * RealDriveType                [SHELL32.524]
//  */
// extern "C" INT WINAPI RealDriveType(INT drive, BOOL bQueryNet)
// {
//     char root[] = "A:\\";
//     root[0] += (char)drive;
//     return GetDriveTypeA(root);
// }
//
// /***********************************************************************
//  *              SHPathPrepareForWriteW (SHELL32.@)
//  */
// extern "C" HRESULT WINAPI SHPathPrepareForWriteW(HWND hwnd, IUnknown *modless, LPCWSTR path, DWORD flags)
// {
//     DWORD res;
//     DWORD err;
//     LPCWSTR realpath;
//     int len;
//     WCHAR* last_slash;
//     WCHAR* temppath=NULL;
//
//     TRACE("%p %p %s 0x%08x\n", hwnd, modless, debugstr_w(path), flags);
//
//     if (flags & ~(SHPPFW_DIRCREATE|SHPPFW_ASKDIRCREATE|SHPPFW_IGNOREFILENAME))
//         FIXME("unimplemented flags 0x%08x\n", flags);
//
//     /* cut off filename if necessary */
//     if (flags & SHPPFW_IGNOREFILENAME)
//     {
//         last_slash = StrRChrW(path, NULL, '\\');
//         if (last_slash == NULL)
//             len = 1;
//         else
//             len = last_slash - path + 1;
//         temppath = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!temppath)
//             return E_OUTOFMEMORY;
//         StrCpyNW(temppath, path, len);
//         realpath = temppath;
//     }
//     else
//     {
//         realpath = path;
//     }
//
//     /* try to create the directory if asked to */
//     if (flags & (SHPPFW_DIRCREATE|SHPPFW_ASKDIRCREATE))
//     {
//         if (flags & SHPPFW_ASKDIRCREATE)
//             FIXME("treating SHPPFW_ASKDIRCREATE as SHPPFW_DIRCREATE\n");
//
//         SHCreateDirectoryExW(0, realpath, NULL);
//     }
//
//     /* check if we can access the directory */
//     res = GetFileAttributesW(realpath);
//
//     HeapFree(GetProcessHeap(), 0, temppath);
//
//     if (res == INVALID_FILE_ATTRIBUTES)
//     {
//         err = GetLastError();
//         if (err == ERROR_FILE_NOT_FOUND)
//             return HRESULT_FROM_WIN32(ERROR_PATH_NOT_FOUND);
//         return HRESULT_FROM_WIN32(err);
//     }
//     else if (res & FILE_ATTRIBUTE_DIRECTORY)
//         return S_OK;
//     else
//         return HRESULT_FROM_WIN32(ERROR_DIRECTORY);
// }
//
// /***********************************************************************
//  *              SHPathPrepareForWriteA (SHELL32.@)
//  */
// extern "C" HRESULT WINAPI SHPathPrepareForWriteA(HWND hwnd, IUnknown *modless, LPCSTR path, DWORD flags)
// {
//     WCHAR wpath[MAX_PATH];
//     MultiByteToWideChar( CP_ACP, 0, path, -1, wpath, MAX_PATH);
//     return SHPathPrepareForWriteW(hwnd, modless, wpath, flags);
// }
//
// static PWSTR SHELL_DupSZZ(PCWSTR Input)
// {
//     SIZE_T len = wcslen(Input), cb = (len + 2) * sizeof(*Input);
//     PWSTR Output = (PWSTR)SHAlloc(cb);
//     if (Output)
//     {
//         CopyMemory(Output, Input, cb - sizeof(*Input));
//         Output[len + 1] = UNICODE_NULL;
//     }
//     return Output;
// }
//
// HRESULT SHELL_SingleFileOperation(HWND hWnd, UINT Op, PCWSTR Src, PCWSTR Dest, UINT Flags, PWSTR *ppNewName)
// {
//     HRESULT hr = S_OK;
//     if ((Src = SHELL_DupSZZ(Src)) == NULL)
//         hr = E_OUTOFMEMORY;
//     if (Dest && (Dest = SHELL_DupSZZ(Dest)) == NULL)
//         hr = E_OUTOFMEMORY;
//
//     SHFILEOPSTRUCTW fos = { hWnd, Op, Src, Dest, (FILEOP_FLAGS)Flags };
//     if (ppNewName)
//     {
//         *ppNewName = NULL;
//         fos.fFlags |= FOF_WANTMAPPINGHANDLE;
//     }
//
//     if (SUCCEEDED(hr))
//     {
//         int err = SHFileOperationW(&fos);
//         hr = SHELL32_FileOpErrorToHResult(err, fos.fAnyOperationsAborted);
//     }
//     else if (CanShowFileOpErrorUI(fos))
//     {
//         SHELL_ErrorBox(hWnd, hr);
//     }
//     SHFree(const_cast<PWSTR>(Src));
//     SHFree(const_cast<PWSTR>(Dest));
//
//     if (fos.hNameMappings)
//     {
//         if (SUCCEEDED(hr) && ppNewName)
//         {
//             assert(DSA_GetItemCount((HDSA)fos.hNameMappings) == 1);
//             SHNAMEMAPPINGW *pMap = (SHNAMEMAPPINGW*)DSA_GetItemPtr((HDSA)fos.hNameMappings, 0);
//             if ((*ppNewName = SHELL_DupSZZ(pMap->pszNewPath)) == NULL)
//                 hr = S_FALSE;
//         }
//         SHFreeNameMappings(fos.hNameMappings);
//     }
//     return hr;
// }
//
// /* Rust bridge — C ABI shim for the Rust-owned CFSFolder (shell32-rust). */
// extern "C" HRESULT WINAPI
// SHELL_SingleFileOperation_RustBridge(HWND hWnd, UINT Op, PCWSTR Src, PCWSTR Dest, UINT Flags, PWSTR *ppNewName)
// {
//     return SHELL_SingleFileOperation(hWnd, Op, Src, Dest, Flags, ppNewName);
// }
//
//
// /*
//  * The two following background operations were modified from filedefext.cpp
//  * They use an inordinate amount of mutable state across the string functions,
//  * so are not easy to follow and care is required when modifying.
//  */
//
// DWORD WINAPI
// _FileOpCountManager(FILE_OPERATION *op, const FILE_LIST *from)
// {
//     DWORD ticks = GetTickCount();
//     FILE_ENTRY *entryToCount;
//
//     for (UINT i = 0; i < from->dwNumFiles; i++)
//     {
//         entryToCount = &from->feFiles[i];
//
//         WCHAR theFileName[MAX_PATH];
//         StringCchCopyW(theFileName, MAX_PATH, entryToCount->szFullPath);
//         _FileOpCount(op, theFileName, IsAttribDir(entryToCount->attributes), &ticks);
//     }
//     return 0;
// }
//
// // All path manipulations, even when this function is nested, occur on the one buffer.
// static BOOL
// _FileOpCount(FILE_OPERATION *op, LPWSTR pwszBuf, BOOL bFolder, DWORD *ticks)
// {
//     /* Find filename position */
//     UINT cchBuf = wcslen(pwszBuf);
//     WCHAR *pwszFilename = pwszBuf + cchBuf;
//     size_t cchFilenameMax = MAX_PATH - cchBuf;
//     if (!cchFilenameMax)
//         return FALSE;
//
//     if (bFolder) {
//         *(pwszFilename++) = '\\';
//         --cchFilenameMax;
//         /* Find all files, FIXME: shouldn't be "*"? */
//         StringCchCopyW(pwszFilename, cchFilenameMax, L"*");
//     }
//
//     WIN32_FIND_DATAW wfd;
//     HANDLE hFind = FindFirstFileW(pwszBuf, &wfd);
//     if (hFind == INVALID_HANDLE_VALUE)
//     {
//         ERR("FindFirstFileW %ls failed\n", pwszBuf);
//         return FALSE;
//     }
//
//     do
//     {
//         if (wfd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//         {
//             /* Don't process "." and ".." items */
//             if (!wcscmp(wfd.cFileName, L".") || !wcscmp(wfd.cFileName, L".."))
//                 continue;
//
//             StringCchCopyW(pwszFilename, cchFilenameMax, wfd.cFileName);
//             _FileOpCount(op, pwszBuf, TRUE, ticks);
//         }
//         else
//         {
//             ULARGE_INTEGER FileSize;
//             FileSize.u.LowPart  = wfd.nFileSizeLow;
//             FileSize.u.HighPart = wfd.nFileSizeHigh;
//             op->totalSize.QuadPart += FileSize.QuadPart;
//         }
//         if (GetTickCount() - *ticks > (DWORD) 500)
//         {
//             // Check if the dialog has ended. If it has, we'll spin down.
//             if (op->progress != NULL)
//                 op->bCancelled = op->progress->HasUserCancelled();
//
//             if (op->bCancelled)
//                 break;
//             *ticks = GetTickCount();
//         }
//     } while(FindNextFileW(hFind, &wfd));
//
//     FindClose(hFind);
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
