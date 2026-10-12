use crate::winmine::dialog::CongratsDlgProc;
use crate::winmine::dialog::CustomDlgProc;
use crate::winmine::dialog::TimesDlgProc;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_E84B679556B9: [u16; 6] = [0x0053u16, 0x006F, 0x0075, 0x006E, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut wnd_style: DWORD = (WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX) as DWORD;

static mut registry_key: [WCHAR; 27] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut xposW: [WCHAR; 5] = [
    'X' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut yposW: [WCHAR; 5] = [
    'Y' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut heightW: [WCHAR; 7] = [
    'H' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut widthW: [WCHAR; 6] = [
    'W' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut minesW: [WCHAR; 6] = [
    'M' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut difficultyW: [WCHAR; 11] = [
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut markW: [WCHAR; 5] = [
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut nameW: [WCHAR; 7] = [
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut timeW: [WCHAR; 7] = [
    'T' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    '%' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// WINE_DEFAULT_DEBUG_CHANNEL(winemine): debug-channel macro, no Rust equivalent emitted here.

#[no_mangle]
pub unsafe extern "C" fn CheckLevel(p_board: *mut BOARD) {
    if (*p_board).rows < BEGINNER_ROWS as ::core::ffi::c_uint {
        (*p_board).rows = BEGINNER_ROWS as ::core::ffi::c_uint;
    }

    if (*p_board).rows > MAX_ROWS as ::core::ffi::c_uint {
        (*p_board).rows = MAX_ROWS as ::core::ffi::c_uint;
    }

    if (*p_board).cols < BEGINNER_COLS as ::core::ffi::c_uint {
        (*p_board).cols = BEGINNER_COLS as ::core::ffi::c_uint;
    }

    if (*p_board).cols > MAX_COLS as ::core::ffi::c_uint {
        (*p_board).cols = MAX_COLS as ::core::ffi::c_uint;
    }

    if (*p_board).mines < BEGINNER_MINES as ::core::ffi::c_uint {
        (*p_board).mines = BEGINNER_MINES as ::core::ffi::c_uint;
    }

    if (*p_board).mines
        > (*p_board)
            .cols
            .wrapping_sub(1 as ::core::ffi::c_uint)
            .wrapping_mul((*p_board).rows.wrapping_sub(1 as ::core::ffi::c_uint))
    {
        (*p_board).mines = (*p_board)
            .cols
            .wrapping_sub(1 as ::core::ffi::c_uint)
            .wrapping_mul((*p_board).rows.wrapping_sub(1 as ::core::ffi::c_uint));
    }
}

unsafe extern "C" fn LoadBoard(p_board: *mut BOARD) {
    let mut size: DWORD;
    let mut type_: DWORD = core::mem::zeroed();
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut data: [WCHAR; (MAX_PLAYER_NAME_SIZE + 1) as _] = [0; (MAX_PLAYER_NAME_SIZE + 1) as _];
    let mut key_name: [WCHAR; 8] = [0; 8];
    let mut i: core::ffi::c_uint;

    RegOpenKeyExW(HKEY_CURRENT_USER, &raw const registry_key as LPCWSTR, 0, KEY_QUERY_VALUE, &raw mut hkey);

    size = core::mem::size_of::<LONG>() as DWORD;
    if RegQueryValueExW(hkey, &raw const xposW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).pos.x as LPBYTE, &raw mut size) != 0 {
        (*p_board).pos.x = 0;
    }

    size = core::mem::size_of::<LONG>() as DWORD;
    if RegQueryValueExW(hkey, &raw const yposW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).pos.y as LPBYTE, &raw mut size) != 0 {
        (*p_board).pos.y = 0;
    }

    size = core::mem::size_of::<core::ffi::c_uint>() as DWORD;
    if RegQueryValueExW(hkey, &raw const heightW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).rows as LPBYTE, &raw mut size) != 0 {
        (*p_board).rows = BEGINNER_ROWS;
    }

    size = core::mem::size_of::<core::ffi::c_uint>() as DWORD;
    if RegQueryValueExW(hkey, &raw const widthW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).cols as LPBYTE, &raw mut size) != 0 {
        (*p_board).cols = BEGINNER_COLS;
    }

    size = core::mem::size_of::<core::ffi::c_uint>() as DWORD;
    if RegQueryValueExW(hkey, &raw const minesW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).mines as LPBYTE, &raw mut size) != 0 {
        (*p_board).mines = BEGINNER_MINES;
    }

    size = core::mem::size_of::<DIFFICULTY>() as DWORD;
    if RegQueryValueExW(hkey, &raw const difficultyW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).difficulty as LPBYTE, &raw mut size) != 0 {
        (*p_board).difficulty = BEGINNER;
    }

    size = core::mem::size_of::<BOOL>() as DWORD;
    if RegQueryValueExW(hkey, &raw const markW as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).IsMarkQ as LPBYTE, &raw mut size) != 0 {
        (*p_board).IsMarkQ = (TRUE as i32);
    }

    /* WinXP SP3: Enabled when value >= 3, disabled when value < 3 */
    size = core::mem::size_of::<BOOL>() as DWORD;
    if RegQueryValueExW(hkey, RUSTCONV_WSTR_E84B679556B9.as_ptr(), core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).IsSoundEnabled as LPBYTE, &raw mut size) == (ERROR_SUCCESS as i32) {
        (*p_board).IsSoundEnabled = (if (*p_board).IsSoundEnabled >= 3 {
            TRUE
        } else {
            FALSE
        }) as BOOL;
    } else {
        (*p_board).IsSoundEnabled = (FALSE as i32);
    }

    i = 0;
    while i < 3 {
        wsprintfW(&raw mut key_name as LPWSTR, &raw const nameW as LPCWSTR, i + 1);
        size = core::mem::size_of::<[WCHAR; 32]>() as DWORD;
        if RegQueryValueExW(hkey, &raw mut key_name as *mut WCHAR as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut data as LPBYTE, &raw mut size) == (ERROR_SUCCESS as i32) {
            lstrcpynW(&raw mut (*p_board).best_name[i as usize] as LPWSTR, &raw const data as *const WCHAR as LPCWSTR, (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int);
        } else {
            LoadStringW((*p_board).hInst, IDS_NOBODY, &raw mut (*p_board).best_name[i as usize] as LPWSTR, (MAX_PLAYER_NAME_SIZE + 1) as i32);
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while i < 3 {
        wsprintfW(&raw mut key_name as LPWSTR, &raw const timeW as LPCWSTR, i + 1);
        size = core::mem::size_of::<DWORD>() as DWORD;
        if RegQueryValueExW(hkey, &raw mut key_name as *mut WCHAR as LPCWSTR, core::ptr::null_mut(), &raw mut type_, &raw mut (*p_board).best_time[i as usize] as LPBYTE, &raw mut size) != 0 {
            (*p_board).best_time[i as usize] = 999;
        }
        i = i.wrapping_add(1);
    }

    RegCloseKey(hkey);
}

unsafe extern "C" fn InitBoard(p_board: *mut BOARD) {
    let mut hMenu: HMENU = core::ptr::null_mut();

    (*p_board).hMinesBMP = LoadBitmapW(
        (*p_board).hInst,
        MAKEINTRESOURCEW!(IDI_MINES) as LPCWSTR,
    );
    (*p_board).hFacesBMP = LoadBitmapW(
        (*p_board).hInst,
        MAKEINTRESOURCEW!(IDI_FACES) as LPCWSTR,
    );
    (*p_board).hLedsBMP = LoadBitmapW(
        (*p_board).hInst,
        MAKEINTRESOURCEW!(IDI_LEDS) as LPCWSTR,
    );

    LoadBoard(p_board);

    hMenu = GetMenu((*p_board).hWnd);
    CheckMenuItem(
        hMenu,
        (IDM_BEGINNER as UINT).wrapping_add((*p_board).difficulty as UINT),
        MF_CHECKED as UINT,
    );
    if (*p_board).IsMarkQ != 0 {
        CheckMenuItem(
            hMenu,
            IDM_MARKQ as UINT,
            MF_CHECKED as UINT,
        );
    } else {
        CheckMenuItem(
            hMenu,
            IDM_MARKQ as UINT,
            MF_UNCHECKED as UINT,
        );
    }
    CheckMenuItem(
        hMenu,
        IDM_SOUND as UINT,
        (if (*p_board).IsSoundEnabled != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    CheckLevel(p_board);
}

unsafe extern "C" fn SaveBoard(p_board: *mut BOARD) {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint;
    let mut data: [WCHAR; (MAX_PLAYER_NAME_SIZE + 1) as _] = core::mem::zeroed();
    let mut key_name: [WCHAR; 8] = core::mem::zeroed();

    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(registry_key) as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut::<WCHAR>(),
        REG_OPTION_NON_VOLATILE as DWORD,
        KEY_WRITE as REGSAM,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        core::ptr::addr_of_mut!(hkey),
        core::ptr::null_mut::<DWORD>(),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(xposW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).pos.x) as *const BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(yposW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).pos.y) as *const BYTE,
        core::mem::size_of::<LONG>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(difficultyW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).difficulty) as *const BYTE,
        core::mem::size_of::<DIFFICULTY>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(heightW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).rows) as *const BYTE,
        core::mem::size_of::<::core::ffi::c_uint>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(widthW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).cols) as *const BYTE,
        core::mem::size_of::<::core::ffi::c_uint>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(minesW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).mines) as *const BYTE,
        core::mem::size_of::<::core::ffi::c_uint>() as DWORD,
    );
    RegSetValueExW(
        hkey,
        core::ptr::addr_of!(markW) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).IsMarkQ) as *const BYTE,
        core::mem::size_of::<BOOL>() as DWORD,
    );
    /* WinXP SP3: Set to 3 when enabled, set to 0 when disabled */
    (*p_board).IsSoundEnabled = (if (*p_board).IsSoundEnabled != 0 {
        3
    } else {
        0
    }) as BOOL;
    RegSetValueExW(
        hkey,
        RUSTCONV_WSTR_E84B679556B9.as_ptr(),
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!((*p_board).IsSoundEnabled) as *const BYTE,
        core::mem::size_of::<BOOL>() as DWORD,
    );

    i = 0 as ::core::ffi::c_uint;
    while i < 3 as ::core::ffi::c_uint {
        wsprintfW(
            core::ptr::addr_of_mut!(key_name) as LPWSTR,
            core::ptr::addr_of!(nameW) as LPCWSTR,
            i.wrapping_add(1 as ::core::ffi::c_uint),
        );
        lstrcpynW(
            core::ptr::addr_of_mut!(data) as LPWSTR,
            (core::ptr::addr_of_mut!((*p_board).best_name) as *mut [WCHAR; 32])
                .offset(i as isize) as *mut WCHAR as LPCWSTR,
            (core::mem::size_of::<[WCHAR; (MAX_PLAYER_NAME_SIZE + 1) as _]>()
                / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        );
        RegSetValueExW(
            hkey,
            core::ptr::addr_of_mut!(key_name) as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            REG_SZ as DWORD,
            core::ptr::addr_of_mut!(data) as *mut WCHAR as *const BYTE,
            ((lstrlenW(core::ptr::addr_of_mut!(data) as *mut WCHAR as LPCWSTR)
                + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        );
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < 3 as ::core::ffi::c_uint {
        wsprintfW(
            core::ptr::addr_of_mut!(key_name) as LPWSTR,
            core::ptr::addr_of!(timeW) as LPCWSTR,
            i.wrapping_add(1 as ::core::ffi::c_uint),
        );
        RegSetValueExW(
            hkey,
            core::ptr::addr_of_mut!(key_name) as *mut WCHAR as LPCWSTR,
            0 as DWORD,
            REG_DWORD as DWORD,
            (core::ptr::addr_of_mut!((*p_board).best_time) as *mut DWORD).offset(i as isize)
                as *const BYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        i = i.wrapping_add(1);
    }
    RegCloseKey(hkey);
}

unsafe extern "C" fn DestroyBoard(p_board: *mut BOARD) {
    DeleteObject((*p_board).hFacesBMP as HGDIOBJ);
    DeleteObject((*p_board).hLedsBMP as HGDIOBJ);
    DeleteObject((*p_board).hMinesBMP as HGDIOBJ);
}

unsafe extern "C" fn SetDifficulty(p_board: *mut BOARD, difficulty: DIFFICULTY) {
    let mut hMenu: HMENU = core::ptr::null_mut();

    if difficulty as core::ffi::c_uint == CUSTOM as core::ffi::c_uint {
        if DialogBoxParamW(
            (*p_board).hInst,
            MAKEINTRESOURCEW!(DLG_CUSTOM as WORD),
            (*p_board).hWnd,
            Some(CustomDlgProc),
            p_board as LPARAM,
        ) != 0 as INT_PTR
        {
            return;
        }
    }

    hMenu = GetMenu((*p_board).hWnd);
    CheckMenuItem(
        hMenu,
        (IDM_BEGINNER as UINT).wrapping_add((*p_board).difficulty as UINT),
        MF_UNCHECKED as UINT,
    );
    (*p_board).difficulty = difficulty;
    CheckMenuItem(
        hMenu,
        (IDM_BEGINNER as UINT).wrapping_add(difficulty as UINT),
        MF_CHECKED as UINT,
    );

    let d: core::ffi::c_uint = difficulty as core::ffi::c_uint;
    if d == BEGINNER as core::ffi::c_uint {
        (*p_board).cols = BEGINNER_COLS as core::ffi::c_uint;
        (*p_board).rows = BEGINNER_ROWS as core::ffi::c_uint;
        (*p_board).mines = BEGINNER_MINES as core::ffi::c_uint;
    } else if d == ADVANCED as core::ffi::c_uint {
        (*p_board).cols = ADVANCED_COLS as core::ffi::c_uint;
        (*p_board).rows = ADVANCED_ROWS as core::ffi::c_uint;
        (*p_board).mines = ADVANCED_MINES as core::ffi::c_uint;
    } else if d == EXPERT as core::ffi::c_uint {
        (*p_board).cols = EXPERT_COLS as core::ffi::c_uint;
        (*p_board).rows = EXPERT_ROWS as core::ffi::c_uint;

        (*p_board).mines = EXPERT_MINES as core::ffi::c_uint;
    } else if d == CUSTOM as core::ffi::c_uint {
        // no-op case
    }
}

unsafe extern "C" fn ShiftBetween(x: *mut LONG, y: *mut LONG, a: LONG, b: LONG) {
    if *x < a {
        *y += a - *x;
        *x = a;
    } else if *y > b {
        *x -= *y - b;
        *y = b;
    }
}

unsafe extern "C" fn MoveOnScreen(rect: *mut RECT) {
    let mut hMonitor: HMONITOR;
    let mut mi: MONITORINFO = core::mem::zeroed();

    /* find the nearest monitor ... */
    hMonitor = MonitorFromRect(rect as LPCRECT, MONITOR_DEFAULTTONEAREST as DWORD);

    /* ... and move it into the work area (ie excluding task bar)*/
    mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(hMonitor, core::ptr::addr_of_mut!(mi));

    ShiftBetween(
        core::ptr::addr_of_mut!((*rect).left),
        core::ptr::addr_of_mut!((*rect).right),
        mi.rcWork.left,
        mi.rcWork.right,
    );
    ShiftBetween(
        core::ptr::addr_of_mut!((*rect).top),
        core::ptr::addr_of_mut!((*rect).bottom),
        mi.rcWork.top,
        mi.rcWork.bottom,
    );
}

unsafe extern "C" fn CreateBoard(p_board: *mut BOARD) {
    let mut left: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_uint = 0;
    let mut row: ::core::ffi::c_uint = 0;
    let mut wnd_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    (*p_board).mb = (MB_NONE) as ::core::ffi::c_uint;
    (*p_board).boxes_left = (*p_board)
        .cols
        .wrapping_mul((*p_board).rows)
        .wrapping_sub((*p_board).mines);
    (*p_board).num_flags = (0) as ::core::ffi::c_uint;

    col = (0) as ::core::ffi::c_uint;
    while col <= (*p_board).cols.wrapping_add((1) as ::core::ffi::c_uint) {
        row = (0) as ::core::ffi::c_uint;
        while row <= (*p_board).rows.wrapping_add((1) as ::core::ffi::c_uint) {
            (*p_board).box_[col as usize][row as usize].set_IsPressed(
                (FALSE) as ::core::ffi::c_uint,
            );
            (*p_board).box_[col as usize][row as usize].set_IsMine(
                (FALSE) as ::core::ffi::c_uint,
            );
            (*p_board).box_[col as usize][row as usize].set_FlagType(
                (NORMAL) as ::core::ffi::c_uint,
            );
            (*p_board).box_[col as usize][row as usize].set_NumMines(
                (0) as ::core::ffi::c_uint,
            );
            row = row.wrapping_add(1);
        }
        col = col.wrapping_add(1);
    }

    (*p_board).width = (*p_board)
        .cols
        .wrapping_mul((MINE_WIDTH) as ::core::ffi::c_uint)
        .wrapping_add(
            ((BOARD_WMARGIN) * (2)) as ::core::ffi::c_uint,
        );
    (*p_board).height = (*p_board)
        .rows
        .wrapping_mul((MINE_HEIGHT) as ::core::ffi::c_uint)
        .wrapping_add((LED_HEIGHT) as ::core::ffi::c_uint)
        .wrapping_add(
            ((BOARD_HMARGIN) * (3)) as ::core::ffi::c_uint,
        );

    /* setting the mines rectangle boundary */
    left = (BOARD_WMARGIN) as ::core::ffi::c_int;
    top = ((BOARD_HMARGIN) * (2) + (LED_HEIGHT)) as i32;
    right = left + ((*p_board).cols.wrapping_mul((MINE_WIDTH) as ::core::ffi::c_uint)) as ::core::ffi::c_int;
    bottom = top + ((*p_board).rows.wrapping_mul((MINE_HEIGHT) as ::core::ffi::c_uint)) as ::core::ffi::c_int;
    SetRect(
        core::ptr::addr_of_mut!((*p_board).mines_rect),
        left,
        top,
        right,
        bottom,
    );

    /* setting the face rectangle boundary */
    left = (*p_board)
        .width
        .wrapping_div((2) as ::core::ffi::c_uint)
        .wrapping_sub(((FACE_WIDTH) / (2)) as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    top = (BOARD_HMARGIN) as ::core::ffi::c_int;
    right = left + (FACE_WIDTH) as ::core::ffi::c_int;
    bottom = top + (FACE_HEIGHT) as ::core::ffi::c_int;
    SetRect(
        core::ptr::addr_of_mut!((*p_board).face_rect),
        left,
        top,
        right,
        bottom,
    );

    /* setting the timer rectangle boundary */
    left = (BOARD_WMARGIN) as ::core::ffi::c_int;
    top = (BOARD_HMARGIN) as ::core::ffi::c_int;
    right = left + ((LED_WIDTH) * (3)) as ::core::ffi::c_int;
    bottom = top + (LED_HEIGHT) as ::core::ffi::c_int;
    SetRect(
        core::ptr::addr_of_mut!((*p_board).timer_rect),
        left,
        top,
        right,
        bottom,
    );

    /* setting the counter rectangle boundary */
    left = (*p_board)
        .width
        .wrapping_sub((BOARD_WMARGIN) as ::core::ffi::c_uint)
        .wrapping_sub(((LED_WIDTH) * (3)) as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    top = (BOARD_HMARGIN) as ::core::ffi::c_int;
    right = (*p_board)
        .width
        .wrapping_sub((BOARD_WMARGIN) as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    bottom = top + (LED_HEIGHT) as ::core::ffi::c_int;
    SetRect(
        core::ptr::addr_of_mut!((*p_board).counter_rect),
        left,
        top,
        right,
        bottom,
    );

    (*p_board).status = (WAITING) as ::core::ffi::c_uint;
    (*p_board).face_bmp = (SMILE_BMP) as ::core::ffi::c_uint;
    (*p_board).time = (0) as ::core::ffi::c_uint;

    SetRect(
        core::ptr::addr_of_mut!(wnd_rect),
        (*p_board).pos.x as ::core::ffi::c_int,
        (*p_board).pos.y as ::core::ffi::c_int,
        ((*p_board).pos.x as ::core::ffi::c_ulong)
            .wrapping_add((*p_board).width as ::core::ffi::c_ulong) as ::core::ffi::c_int,
        ((*p_board).pos.y as ::core::ffi::c_ulong)
            .wrapping_add((*p_board).height as ::core::ffi::c_ulong) as ::core::ffi::c_int,
    );
    AdjustWindowRect(
        core::ptr::addr_of_mut!(wnd_rect),
        wnd_style,
        (TRUE) as ::core::ffi::c_int,
    );

    /* Make sure the window is completely on the screen */
    MoveOnScreen(core::ptr::addr_of_mut!(wnd_rect));
    MoveWindow(
        (*p_board).hWnd,
        wnd_rect.left as ::core::ffi::c_int,
        wnd_rect.top as ::core::ffi::c_int,
        (wnd_rect.right - wnd_rect.left) as ::core::ffi::c_int,
        (wnd_rect.bottom - wnd_rect.top) as ::core::ffi::c_int,
        (TRUE) as ::core::ffi::c_int,
    );
    RedrawWindow(
        (*p_board).hWnd,
        ::core::ptr::null::<RECT>(),
        (0) as HRGN,
        ((RDW_INVALIDATE) | (RDW_UPDATENOW) | (RDW_ERASE)) as UINT,
    );
}

unsafe extern "C" fn PlaceMines(
    p_board: *mut BOARD,
    selected_col: ::core::ffi::c_int,
    selected_row: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_uint = 0;
    let mut row: ::core::ffi::c_uint = 0;
    srand(
        (time((::core::ptr::null_mut::<time_t>()).cast())) as ::core::ffi::c_uint,
    );
    (*p_board).box_[selected_col as usize][selected_row as usize]
        .set_IsMine(TRUE as ::core::ffi::c_uint);
    i = 0 as ::core::ffi::c_int;
    while (i as ::core::ffi::c_uint) < (*p_board).mines {
        col = (rand() as ::core::ffi::c_uint)
            .wrapping_rem((*p_board).cols)
            .wrapping_add(1 as ::core::ffi::c_uint);
        row = (rand() as ::core::ffi::c_uint)
            .wrapping_rem((*p_board).rows)
            .wrapping_add(1 as ::core::ffi::c_uint);
        if (*p_board).box_[col as usize][row as usize].IsMine() == 0 {
            i += 1;
            (*p_board).box_[col as usize][row as usize]
                .set_IsMine(TRUE as ::core::ffi::c_uint);
        }
    }
    (*p_board).box_[selected_col as usize][selected_row as usize]
        .set_IsMine(FALSE as ::core::ffi::c_uint);
    col = 1 as ::core::ffi::c_uint;
    while col < (*p_board).cols.wrapping_add(1 as ::core::ffi::c_uint) {
        row = 1 as ::core::ffi::c_uint;
        while row < (*p_board).rows.wrapping_add(1 as ::core::ffi::c_uint) {
            i = -(1 as ::core::ffi::c_int);
            while i <= 1 as ::core::ffi::c_int {
                j = -(1 as ::core::ffi::c_int);
                while j <= 1 as ::core::ffi::c_int {
                    if (*p_board).box_[col.wrapping_add(i as ::core::ffi::c_uint) as usize]
                        [row.wrapping_add(j as ::core::ffi::c_uint) as usize]
                        .IsMine()
                        != 0
                    {
                        (*p_board).box_[col as usize][row as usize].set_NumMines(
                            (*p_board).box_[col as usize][row as usize].NumMines()
                                + 1 as ::core::ffi::c_uint,
                        );
                    }
                    j += 1;
                }
                i += 1;
            }
            row = row.wrapping_add(1);
        }
        col = col.wrapping_add(1);
    }
}

unsafe extern "C" fn DrawMine(hdc: HDC, hMemDC: HDC, p_board: *mut BOARD, col: ::core::ffi::c_uint, row: ::core::ffi::c_uint, IsPressed: BOOL) {
    let mut offset: MINEBMP_OFFSET = BOX_BMP;

    if col == 0 as ::core::ffi::c_uint
        || col > (*p_board).cols
        || row == 0 as ::core::ffi::c_uint
        || row > (*p_board).rows
    {
        return;
    }

    if (*p_board).status as ::core::ffi::c_uint == GAMEOVER as ::core::ffi::c_uint {
        if (*p_board).box_[col as usize][row as usize].IsMine() != 0 {
            match ((*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int) as u32 {
                FLAG => {
                    offset = FLAG_BMP;
                }
                COMPLETE => {
                    offset = EXPLODE_BMP;
                }
                QUESTION | NORMAL => {
                    offset = MINE_BMP;
                }
                _ => {}
            }
        } else {
            match ((*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int) as u32 {
                QUESTION => {
                    offset = QUESTION_BMP;
                }
                FLAG => {
                    offset = WRONG_BMP;
                }
                NORMAL => {
                    offset = BOX_BMP;
                }
                COMPLETE => {
                    // Do nothing
                }
                _ => {
                    WINE_TRACE!(b"Unknown FlagType during game over in DrawMine\n\0".as_ptr() as *const ::core::ffi::c_char);
                }
            }
        }
    } else {
        // WAITING or PLAYING
        match ((*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int) as u32 {
            QUESTION => {
                if IsPressed == 0 {
                    offset = QUESTION_BMP;
                } else {
                    offset = QPRESS_BMP;
                }
            }
            FLAG => {
                offset = FLAG_BMP;
            }
            NORMAL => {
                if IsPressed == 0 {
                    offset = BOX_BMP;
                } else {
                    offset = MPRESS_BMP;
                }
            }
            COMPLETE => {
                // Do nothing
            }
            _ => {
                WINE_TRACE!(b"Unknown FlagType while playing in DrawMine\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
    }

    if (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int == COMPLETE as ::core::ffi::c_int
        && (*p_board).box_[col as usize][row as usize].IsMine() == 0
    {
        offset = (*p_board).box_[col as usize][row as usize].NumMines() as MINEBMP_OFFSET;
    }

    BitBlt(
        hdc,
        ((col.wrapping_sub(1 as ::core::ffi::c_uint))
            .wrapping_mul(MINE_WIDTH as ::core::ffi::c_uint) as ::core::ffi::c_ulong)
            .wrapping_add((*p_board).mines_rect.left as ::core::ffi::c_ulong) as INT,
        ((row.wrapping_sub(1 as ::core::ffi::c_uint))
            .wrapping_mul(MINE_HEIGHT as ::core::ffi::c_uint) as ::core::ffi::c_ulong)
            .wrapping_add((*p_board).mines_rect.top as ::core::ffi::c_ulong) as INT,
        (MINE_WIDTH) as INT,
        (MINE_HEIGHT) as INT,
        hMemDC,
        0,
        (offset as ::core::ffi::c_uint).wrapping_mul(MINE_HEIGHT as ::core::ffi::c_uint) as INT,
        SRCCOPY,
    );
}

unsafe extern "C" fn DrawMines(hdc: HDC, hMemDC: HDC, p_board: *mut BOARD) {
    let mut hOldObj: HGDIOBJ;
    let mut col: u32;
    let mut row: u32;
    hOldObj = SelectObject(hMemDC, (*p_board).hMinesBMP as HGDIOBJ);

    row = 1 as u32;
    while row <= (*p_board).rows {
        col = 1 as u32;
        while col <= (*p_board).cols {
            DrawMine(hdc, hMemDC, p_board, col, row, FALSE as i32);
            col = col.wrapping_add(1);
        }
        row = row.wrapping_add(1);
    }
    SelectObject(hMemDC, hOldObj);
}

unsafe extern "C" fn DrawLeds(
    mut hdc: HDC,
    mut hMemDC: HDC,
    mut p_board: *mut BOARD,
    mut number: ::core::ffi::c_int,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
) {
    let mut hOldObj: HGDIOBJ = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut led: [::core::ffi::c_uint; 3 as usize] = [0; 3 as usize];
    let mut i: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_int = 0;
    count = number;
    if count < 1000 as ::core::ffi::c_int {
        if count >= 0 as ::core::ffi::c_int {
            led[0 as usize] =
                (count / 100 as ::core::ffi::c_int) as ::core::ffi::c_uint;
            count = (count as ::core::ffi::c_uint).wrapping_sub(
                led[0 as usize]
                    .wrapping_mul(100 as ::core::ffi::c_uint),
            ) as ::core::ffi::c_int;
        } else {
            led[0 as usize] = 10 as ::core::ffi::c_uint;
            count = -count;
        }
        led[1 as usize] =
            (count / 10 as ::core::ffi::c_int) as ::core::ffi::c_uint;
        count = (count as ::core::ffi::c_uint).wrapping_sub(
            led[1 as usize]
                .wrapping_mul(10 as ::core::ffi::c_uint),
        ) as ::core::ffi::c_int;
        led[2 as usize] = count as ::core::ffi::c_uint;
    } else {
        i = 0 as ::core::ffi::c_uint;
        while i < 3 as ::core::ffi::c_uint {
            led[i as usize] = 10 as ::core::ffi::c_uint;
            i = i.wrapping_add(1);
        }
    }

    hOldObj = SelectObject(hMemDC, (*p_board).hLedsBMP as HGDIOBJ);

    i = 0 as ::core::ffi::c_uint;
    while i < 3 as ::core::ffi::c_uint {
        BitBlt(
            hdc,
            i.wrapping_mul(LED_WIDTH as ::core::ffi::c_uint)
                .wrapping_add(x as ::core::ffi::c_uint) as INT,
            y as INT,
            (LED_WIDTH) as INT,
            (LED_HEIGHT) as INT,
            hMemDC,
            0 as INT,
            led[i as usize].wrapping_mul(LED_HEIGHT as ::core::ffi::c_uint) as INT,
            SRCCOPY as DWORD,
        );
        i = i.wrapping_add(1);
    }

    SelectObject(hMemDC, hOldObj);
}

unsafe extern "C" fn DrawFace(hdc: HDC, hMemDC: HDC, p_board: *mut BOARD) {
    let mut hOldObj: HGDIOBJ = core::ptr::null_mut();

    hOldObj = SelectObject(hMemDC, (*p_board).hFacesBMP as HGDIOBJ);

    BitBlt(
        hdc,
        (*p_board).face_rect.left as INT,
        (*p_board).face_rect.top as INT,
        (FACE_WIDTH) as INT,
        (FACE_HEIGHT) as INT,
        hMemDC,
        0 as INT,
        ((*p_board).face_bmp as core::ffi::c_uint)
            .wrapping_mul(FACE_HEIGHT as core::ffi::c_uint) as INT,
        SRCCOPY as DWORD,
    );

    SelectObject(hMemDC, hOldObj);
}

unsafe extern "C" fn DrawBoard(hdc: HDC, hMemDC: HDC, ps: *mut PAINTSTRUCT, p_board: *mut BOARD) {
    let mut tmp_rect: RECT = core::mem::zeroed();

    if IntersectRect(
        &mut tmp_rect as *mut RECT,
        core::ptr::addr_of_mut!((*ps).rcPaint),
        core::ptr::addr_of_mut!((*p_board).counter_rect),
    ) != 0
    {
        DrawLeds(
            hdc,
            hMemDC,
            p_board,
            (*p_board).mines.wrapping_sub((*p_board).num_flags) as i32,
            (*p_board).counter_rect.left,
            (*p_board).counter_rect.top,
        );
    }

    if IntersectRect(
        &mut tmp_rect as *mut RECT,
        core::ptr::addr_of_mut!((*ps).rcPaint),
        core::ptr::addr_of_mut!((*p_board).timer_rect),
    ) != 0
    {
        DrawLeds(
            hdc,
            hMemDC,
            p_board,
            (*p_board).time as i32,
            (*p_board).timer_rect.left,
            (*p_board).timer_rect.top,
        );
    }

    if IntersectRect(
        &mut tmp_rect as *mut RECT,
        core::ptr::addr_of_mut!((*ps).rcPaint),
        core::ptr::addr_of_mut!((*p_board).face_rect),
    ) != 0
    {
        DrawFace(hdc, hMemDC, p_board);
    }

    if IntersectRect(
        &mut tmp_rect as *mut RECT,
        core::ptr::addr_of_mut!((*ps).rcPaint),
        core::ptr::addr_of_mut!((*p_board).mines_rect),
    ) != 0
    {
        DrawMines(hdc, hMemDC, p_board);
    }
}

unsafe extern "C" fn PlaySoundEffect(p_board: *mut BOARD, id: UINT) {
    if (*p_board).IsSoundEnabled != 0 {
        PlaySoundW(MAKEINTRESOURCEW!(id), (*p_board).hInst, SND_RESOURCE | SND_ASYNC);
    }
}

unsafe extern "C" fn AddFlag(
    mut p_board: *mut BOARD,
    mut col: ::core::ffi::c_uint,
    mut row: ::core::ffi::c_uint,
) {
    if (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int != (COMPLETE as i32) {
        match ((*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int) as u32 {
            FLAG => {
                if (*p_board).IsMarkQ != 0 {
                    (*p_board).box_[col as usize][row as usize]
                        .set_FlagType(QUESTION as ::core::ffi::c_uint as ::core::ffi::c_uint);
                    PlaySoundEffect(p_board, IDW_QUESTION as UINT);
                } else {
                    (*p_board).box_[col as usize][row as usize]
                        .set_FlagType(NORMAL as ::core::ffi::c_uint as ::core::ffi::c_uint);
                }
                (*p_board).num_flags = (*p_board).num_flags.wrapping_sub(1);
            }
            QUESTION => {
                (*p_board).box_[col as usize][row as usize]
                    .set_FlagType(NORMAL as ::core::ffi::c_uint as ::core::ffi::c_uint);
            }
            _ => {
                (*p_board).box_[col as usize][row as usize]
                    .set_FlagType(FLAG as ::core::ffi::c_uint as ::core::ffi::c_uint);
                (*p_board).num_flags = (*p_board).num_flags.wrapping_add(1);
                PlaySoundEffect(p_board, IDW_FLAG as UINT);
            }
        }
    }
}

unsafe extern "C" fn UnpressBox(p_board: *mut BOARD, col: ::core::ffi::c_uint, row: ::core::ffi::c_uint) {
    let mut hdc: HDC;
    let mut hOldObj: HGDIOBJ;
    let mut hMemDC: HDC;

    hdc = GetDC((*p_board).hWnd);
    hMemDC = CreateCompatibleDC(hdc);
    hOldObj = SelectObject(hMemDC, (*p_board).hMinesBMP as HGDIOBJ);

    DrawMine(hdc, hMemDC, p_board, col, row, (FALSE) as BOOL);

    SelectObject(hMemDC, hOldObj);
    DeleteDC(hMemDC);
    ReleaseDC((*p_board).hWnd, hdc);
}

unsafe extern "C" fn UnpressBoxes(p_board: *mut BOARD, col: ::core::ffi::c_uint, row: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int;
    i = -1;
    while i <= 1 {
        j = -1;
        while j <= 1 {
            UnpressBox(
                p_board,
                col.wrapping_add(i as ::core::ffi::c_uint),
                row.wrapping_add(j as ::core::ffi::c_uint),
            );
            j += 1;
        }
        i += 1;
    }
}

unsafe extern "C" fn PressBox(p_board: *mut BOARD, col: ::core::ffi::c_uint, row: ::core::ffi::c_uint) {
    let mut hdc: HDC;
    let mut hOldObj: HGDIOBJ;
    let mut hMemDC: HDC;

    hdc = GetDC((*p_board).hWnd);
    hMemDC = CreateCompatibleDC(hdc);
    hOldObj = SelectObject(hMemDC, (*p_board).hMinesBMP as HGDIOBJ);

    DrawMine(hdc, hMemDC, p_board, col, row, (TRUE) as BOOL);

    SelectObject(hMemDC, hOldObj);
    DeleteDC(hMemDC);
    ReleaseDC((*p_board).hWnd, hdc);
}

unsafe extern "C" fn PressBoxes(mut p_board: *mut BOARD, mut col: ::core::ffi::c_uint, mut row: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    i = -(1 as ::core::ffi::c_int);
    while i <= 1 as ::core::ffi::c_int {
        j = -(1 as ::core::ffi::c_int);
        while j <= 1 as ::core::ffi::c_int {
            (*p_board).box_[col.wrapping_add(i as ::core::ffi::c_uint) as usize]
                [row.wrapping_add(j as ::core::ffi::c_uint) as usize]
                .set_IsPressed(TRUE as ::core::ffi::c_uint);
            PressBox(
                p_board,
                col.wrapping_add(i as ::core::ffi::c_uint),
                row.wrapping_add(j as ::core::ffi::c_uint),
            );
            j += 1;
        }
        i += 1;
    }
    i = -(1 as ::core::ffi::c_int);
    while i <= 1 as ::core::ffi::c_int {
        j = -(1 as ::core::ffi::c_int);
        while j <= 1 as ::core::ffi::c_int {
            if (*p_board).box_[((*p_board).press.x + i as LONG) as usize]
                [((*p_board).press.y + j as LONG) as usize]
                .IsPressed()
                == 0
            {
                UnpressBox(
                    p_board,
                    ((*p_board).press.x + i as LONG) as ::core::ffi::c_uint,
                    ((*p_board).press.y + j as LONG) as ::core::ffi::c_uint,
                );
            }
            j += 1;
        }
        i += 1;
    }
    i = -(1 as ::core::ffi::c_int);
    while i <= 1 as ::core::ffi::c_int {
        j = -(1 as ::core::ffi::c_int);
        while j <= 1 as ::core::ffi::c_int {
            (*p_board).box_[col.wrapping_add(i as ::core::ffi::c_uint) as usize]
                [row.wrapping_add(j as ::core::ffi::c_uint) as usize]
                .set_IsPressed(FALSE as ::core::ffi::c_uint);
            PressBox(
                p_board,
                col.wrapping_add(i as ::core::ffi::c_uint),
                row.wrapping_add(j as ::core::ffi::c_uint),
            );
            j += 1;
        }
        i += 1;
    }
    (*p_board).press.x = col as LONG;
    (*p_board).press.y = row as LONG;
}

unsafe extern "C" fn CompleteBox(
    mut p_board: *mut BOARD,
    mut col: ::core::ffi::c_uint,
    mut row: ::core::ffi::c_uint,
    mut is_first_box: BOOL,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    if (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int != (COMPLETE as i32)
        && (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int != (FLAG as i32)
        && col > 0 as ::core::ffi::c_uint
        && col < (*p_board).cols.wrapping_add(1 as ::core::ffi::c_uint)
        && row > 0 as ::core::ffi::c_uint
        && row < (*p_board).rows.wrapping_add(1 as ::core::ffi::c_uint)
    {
        (*p_board).box_[col as usize][row as usize]
            .set_FlagType(COMPLETE as ::core::ffi::c_uint);
        if (*p_board).box_[col as usize][row as usize].IsMine() != 0 {
            (*p_board).face_bmp = DEAD_BMP;
            (*p_board).status = GAMEOVER;
            PlaySoundEffect(p_board, IDW_EXPLODE as UINT);
        } else if (*p_board).status as ::core::ffi::c_uint
            != GAMEOVER as ::core::ffi::c_uint
        {
            (*p_board).boxes_left = (*p_board).boxes_left.wrapping_sub(1);
            if is_first_box != 0 {
                PlaySoundEffect(p_board, IDW_BOX as UINT);
            }
        }
        if (*p_board).box_[col as usize][row as usize].NumMines() as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            i = -(1 as ::core::ffi::c_int);
            while i <= 1 as ::core::ffi::c_int {
                j = -(1 as ::core::ffi::c_int);
                while j <= 1 as ::core::ffi::c_int {
                    CompleteBox(
                        p_board,
                        col.wrapping_add(i as ::core::ffi::c_uint),
                        row.wrapping_add(j as ::core::ffi::c_uint),
                        FALSE as BOOL,
                    );
                    j += 1;
                }
                i += 1;
            }
        }
    }
}

unsafe extern "C" fn CompleteBoxes(
    mut p_board: *mut BOARD,
    mut col: ::core::ffi::c_uint,
    mut row: ::core::ffi::c_uint,
) {
    let mut numFlags: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    if (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int
        == (COMPLETE as i32)
    {
        i = -(1 as ::core::ffi::c_int);
        while i <= 1 as ::core::ffi::c_int {
            j = -(1 as ::core::ffi::c_int);
            while j <= 1 as ::core::ffi::c_int {
                if (*p_board).box_[col.wrapping_add(i as ::core::ffi::c_uint) as usize]
                    [row.wrapping_add(j as ::core::ffi::c_uint) as usize]
                    .FlagType() as ::core::ffi::c_int
                    == (FLAG as i32)
                {
                    numFlags = numFlags.wrapping_add(1);
                }
                j += 1;
            }
            i += 1;
        }
        if numFlags == (*p_board).box_[col as usize][row as usize].NumMines() {
            i = -(1 as ::core::ffi::c_int);
            while i <= 1 as ::core::ffi::c_int {
                j = -(1 as ::core::ffi::c_int);
                while j <= 1 as ::core::ffi::c_int {
                    if (*p_board).box_[col.wrapping_add(i as ::core::ffi::c_uint) as usize]
                        [row.wrapping_add(j as ::core::ffi::c_uint) as usize]
                        .FlagType() as ::core::ffi::c_int
                        != (FLAG as i32)
                    {
                        CompleteBox(
                            p_board,
                            col.wrapping_add(i as ::core::ffi::c_uint),
                            row.wrapping_add(j as ::core::ffi::c_uint),
                            (FALSE) as BOOL,
                        );
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }
}

unsafe extern "C" fn TestMines(
    mut p_board: *mut BOARD,
    mut pt: POINT,
    mut msg: ::core::ffi::c_int,
) {
    let mut draw: BOOL = (TRUE as i32);
    let mut col: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    col = ((pt.x - (*p_board).mines_rect.left) / MINE_WIDTH as LONG
        + 1 as LONG) as ::core::ffi::c_int;
    row = ((pt.y - (*p_board).mines_rect.top) / MINE_HEIGHT as LONG
        + 1 as LONG) as ::core::ffi::c_int;
    match (msg) as u32 {
        WM_LBUTTONDOWN => {
            if (*p_board).press.x != col as LONG || (*p_board).press.y != row as LONG {
                UnpressBox(
                    p_board,
                    (*p_board).press.x as ::core::ffi::c_uint,
                    (*p_board).press.y as ::core::ffi::c_uint,
                );
                (*p_board).press.x = col as LONG;
                (*p_board).press.y = row as LONG;
                PressBox(
                    p_board,
                    col as ::core::ffi::c_uint,
                    row as ::core::ffi::c_uint,
                );
            }
            draw = (FALSE as i32);
        }
        WM_LBUTTONUP => {
            if (*p_board).press.x != col as LONG || (*p_board).press.y != row as LONG {
                UnpressBox(
                    p_board,
                    (*p_board).press.x as ::core::ffi::c_uint,
                    (*p_board).press.y as ::core::ffi::c_uint,
                );
            }
            (*p_board).press.x = 0 as LONG;
            (*p_board).press.y = 0 as LONG;
            if (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int
                != (FLAG as i32)
                && (*p_board).status as ::core::ffi::c_uint
                    != PLAYING as ::core::ffi::c_uint
            {
                (*p_board).status = PLAYING;
                PlaceMines(p_board, col, row);
            }
            CompleteBox(
                p_board,
                col as ::core::ffi::c_uint,
                row as ::core::ffi::c_uint,
                (TRUE) as BOOL,
            );
        }
        WM_MBUTTONDOWN => {
            PressBoxes(
                p_board,
                col as ::core::ffi::c_uint,
                row as ::core::ffi::c_uint,
            );
            draw = (FALSE as i32);
        }
        WM_MBUTTONUP => {
            if (*p_board).press.x != col as LONG || (*p_board).press.y != row as LONG {
                UnpressBoxes(
                    p_board,
                    (*p_board).press.x as ::core::ffi::c_uint,
                    (*p_board).press.y as ::core::ffi::c_uint,
                );
            }
            (*p_board).press.x = 0 as LONG;
            (*p_board).press.y = 0 as LONG;
            CompleteBoxes(
                p_board,
                col as ::core::ffi::c_uint,
                row as ::core::ffi::c_uint,
            );
        }
        WM_RBUTTONDOWN => {
            AddFlag(
                p_board,
                col as ::core::ffi::c_uint,
                row as ::core::ffi::c_uint,
            );
        }
        _ => {
            WINE_TRACE!(
                b"Unknown message type received in TestMines\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    }
    if draw != 0 {
        RedrawWindow(
            (*p_board).hWnd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
        );
    }
}

unsafe extern "C" fn TestFace(p_board: *mut BOARD, pt: POINT, msg: core::ffi::c_int) {
    if (*p_board).status == PLAYING || (*p_board).status == WAITING {
        if msg == (WM_LBUTTONDOWN as i32) || msg == (WM_MBUTTONDOWN as i32) {
            (*p_board).face_bmp = OOH_BMP;
        } else {
            (*p_board).face_bmp = SMILE_BMP;
        }
    } else if (*p_board).status == GAMEOVER {
        (*p_board).face_bmp = DEAD_BMP;
    } else if (*p_board).status == WON {
        (*p_board).face_bmp = COOL_BMP;
    }

    if PtInRect(core::ptr::addr_of_mut!((*p_board).face_rect), pt) != 0 {
        if msg == (WM_LBUTTONDOWN as i32) {
            (*p_board).face_bmp = SPRESS_BMP;
        }

        if msg == (WM_LBUTTONUP as i32) {
            CreateBoard(p_board);
        }
    }

    RedrawWindow(
        (*p_board).hWnd,
        core::ptr::addr_of_mut!((*p_board).face_rect),
        core::ptr::null_mut(),
        RDW_INVALIDATE | RDW_UPDATENOW,
    );
}

unsafe extern "C" fn TestBoard(hWnd: HWND, p_board: *mut BOARD, x: ::core::ffi::c_int, y: ::core::ffi::c_int, msg: ::core::ffi::c_int) {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut col: ::core::ffi::c_uint = 0;
    let mut row: ::core::ffi::c_uint = 0;

    pt.x = x as LONG;
    pt.y = y as LONG;

    if PtInRect(&raw mut (*p_board).mines_rect as LPCRECT, pt) != 0
        && (*p_board).status as ::core::ffi::c_uint != GAMEOVER as ::core::ffi::c_uint
        && (*p_board).status as ::core::ffi::c_uint != WON as ::core::ffi::c_uint
    {
        TestMines(p_board, pt, msg);
    } else {
        UnpressBoxes(
            p_board,
            (*p_board).press.x as ::core::ffi::c_uint,
            (*p_board).press.y as ::core::ffi::c_uint,
        );
        (*p_board).press.x = 0 as LONG;
        (*p_board).press.y = 0 as LONG;
    }

    if (*p_board).boxes_left == 0 as ::core::ffi::c_uint {
        if (*p_board).status as ::core::ffi::c_uint != WON as ::core::ffi::c_uint {
            PlaySoundEffect(p_board, IDW_WIN as UINT);
        }
        (*p_board).status = WON;

        if (*p_board).num_flags < (*p_board).mines {
            row = 1 as ::core::ffi::c_uint;
            while row <= (*p_board).rows {
                col = 1 as ::core::ffi::c_uint;
                while col <= (*p_board).cols {
                    if (*p_board).box_[col as usize][row as usize].IsMine() as ::core::ffi::c_int != 0
                        && (*p_board).box_[col as usize][row as usize].FlagType() as ::core::ffi::c_int
                            != FLAG as ::core::ffi::c_int
                    {
                        (*p_board).box_[col as usize][row as usize]
                            .set_FlagType(FLAG as ::core::ffi::c_uint);
                    }
                    col = col.wrapping_add(1);
                }
                row = row.wrapping_add(1);
            }

            (*p_board).num_flags = (*p_board).mines;

            RedrawWindow(
                (*p_board).hWnd,
                ::core::ptr::null::<RECT>(),
                core::ptr::null_mut::<HRGN__>(),
                (RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
            );
        }

        if (*p_board).difficulty as ::core::ffi::c_uint != CUSTOM as ::core::ffi::c_uint
            && ((*p_board).time as DWORD) < (*p_board).best_time[(*p_board).difficulty as usize]
        {
            (*p_board).best_time[(*p_board).difficulty as usize] = (*p_board).time as DWORD;

            DialogBoxParamW(
                (*p_board).hInst,
                MAKEINTRESOURCEW!(DLG_CONGRATS),
                hWnd,
                Some(CongratsDlgProc),
                p_board as LPARAM,
            );
            DialogBoxParamW(
                (*p_board).hInst,
                MAKEINTRESOURCEW!(DLG_TIMES),
                hWnd,
                Some(TimesDlgProc),
                p_board as LPARAM,
            );
        }
    }
    TestFace(p_board, pt, msg);
}

unsafe extern "system" fn MainProc(hWnd: HWND, mut msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hMenu: HMENU = core::ptr::null_mut();
    static mut board: BOARD = unsafe { core::mem::zeroed() };

    match msg {
        WM_CREATE => {
            board.hInst = (*(lParam as LPCREATESTRUCTW)).hInstance;
            board.hWnd = hWnd;
            InitBoard(&raw mut board);
            CreateBoard(&raw mut board);
            return 0 as LRESULT;
        }

        WM_PAINT => {
            let mut hMemDC: HDC = core::ptr::null_mut();

            WINE_TRACE!("WM_PAINT\n");
            hdc = BeginPaint(hWnd, &raw mut ps);
            hMemDC = CreateCompatibleDC(hdc);

            DrawBoard(hdc, hMemDC, &raw mut ps, &raw mut board);

            DeleteDC(hMemDC);
            EndPaint(hWnd, &raw mut ps);

            return 0 as LRESULT;
        }

        WM_MOVE => {
            WINE_TRACE!("WM_MOVE\n");
            board.pos.x = LOWORD!(lParam) as WORD as i16 as LONG;
            board.pos.y = HIWORD!(lParam) as WORD as i16 as LONG;
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            SaveBoard(&raw mut board);
            DestroyBoard(&raw mut board);
            PostQuitMessage(0);
            return 0 as LRESULT;
        }

        WM_TIMER => {
            if board.status == PLAYING {
                board.time = board.time.wrapping_add(1);
                RedrawWindow(
                    hWnd,
                    &raw mut board.timer_rect,
                    0 as usize as HRGN,
                    (RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
                );
            }
            return 0 as LRESULT;
        }

        WM_LBUTTONDOWN => {
            WINE_TRACE!("WM_LBUTTONDOWN\n");
            if (wParam & ((MK_RBUTTON | MK_SHIFT)) as WPARAM) != 0 {
                msg = WM_MBUTTONDOWN as UINT;
            }
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            SetCapture(hWnd);
            return 0 as LRESULT;
        }

        WM_LBUTTONUP => {
            WINE_TRACE!("WM_LBUTTONUP\n");
            if (wParam & ((MK_RBUTTON | MK_SHIFT)) as WPARAM) != 0 {
                msg = WM_MBUTTONUP as UINT;
            }
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            ReleaseCapture();
            return 0 as LRESULT;
        }

        WM_RBUTTONDOWN => {
            WINE_TRACE!("WM_RBUTTONDOWN\n");
            if (wParam & (MK_LBUTTON as WPARAM)) != 0 {
                board.press.x = 0;
                board.press.y = 0;
                msg = WM_MBUTTONDOWN as UINT;
            }
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            return 0 as LRESULT;
        }

        WM_RBUTTONUP => {
            WINE_TRACE!("WM_RBUTTONUP\n");
            if (wParam & (MK_LBUTTON as WPARAM)) != 0 {
                msg = WM_MBUTTONUP as UINT;
            }
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            return 0 as LRESULT;
        }

        WM_MBUTTONDOWN => {
            WINE_TRACE!("WM_MBUTTONDOWN\n");
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            return 0 as LRESULT;
        }

        WM_MBUTTONUP => {
            WINE_TRACE!("WM_MBUTTONUP\n");
            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );
            return 0 as LRESULT;
        }

        WM_MOUSEMOVE => {
            if (wParam & (MK_MBUTTON as WPARAM)) != 0
                || ((wParam & (MK_LBUTTON as WPARAM)) != 0 && (wParam & (MK_RBUTTON as WPARAM)) != 0)
            {
                msg = WM_MBUTTONDOWN as UINT;
            } else if (wParam & (MK_LBUTTON as WPARAM)) != 0 {
                msg = WM_LBUTTONDOWN as UINT;
            } else {
                return 0 as LRESULT;
            }

            TestBoard(
                hWnd,
                &raw mut board,
                LOWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                HIWORD!(lParam) as WORD as i16 as core::ffi::c_int,
                msg as core::ffi::c_int,
            );

            return 0 as LRESULT;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as WORD as core::ffi::c_int) as u32 {
                IDM_NEW => {
                    CreateBoard(&raw mut board);
                    return 0 as LRESULT;
                }

                IDM_MARKQ => {
                    hMenu = GetMenu(hWnd);
                    board.IsMarkQ = (board.IsMarkQ == 0) as core::ffi::c_int as BOOL;
                    if board.IsMarkQ != 0 {
                        CheckMenuItem(
                            hMenu,
                            IDM_MARKQ as UINT,
                            MF_CHECKED as UINT,
                        );
                    } else {
                        CheckMenuItem(
                            hMenu,
                            IDM_MARKQ as UINT,
                            MF_UNCHECKED as UINT,
                        );
                    }
                    return 0 as LRESULT;
                }

                IDM_SOUND => {
                    hMenu = GetMenu(hWnd);
                    board.IsSoundEnabled = (board.IsSoundEnabled == 0) as core::ffi::c_int as BOOL;
                    CheckMenuItem(
                        hMenu,
                        IDM_SOUND as UINT,
                        (if board.IsSoundEnabled != 0 {
                            MF_CHECKED
                        } else {
                            MF_UNCHECKED
                        }) as UINT,
                    );
                    return 0 as LRESULT;
                }

                IDM_BEGINNER => {
                    SetDifficulty(&raw mut board, BEGINNER);
                    CreateBoard(&raw mut board);
                    return 0 as LRESULT;
                }

                IDM_ADVANCED => {
                    SetDifficulty(&raw mut board, ADVANCED);
                    CreateBoard(&raw mut board);
                    return 0 as LRESULT;
                }

                IDM_EXPERT => {
                    SetDifficulty(&raw mut board, EXPERT);
                    CreateBoard(&raw mut board);
                    return 0 as LRESULT;
                }

                IDM_CUSTOM => {
                    SetDifficulty(&raw mut board, CUSTOM);
                    CreateBoard(&raw mut board);
                    return 0 as LRESULT;
                }

                IDM_EXIT => {
                    SendMessageW(
                        hWnd,
                        WM_CLOSE as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    );
                    return 0 as LRESULT;
                }

                IDM_TIMES => {
                    DialogBoxParamW(
                        board.hInst,
                        MAKEINTRESOURCEW!(DLG_TIMES) as LPCWSTR,
                        hWnd,
                        Some(TimesDlgProc),
                        &raw mut board as LPARAM,
                    );
                    return 0 as LRESULT;
                }

                IDM_ABOUT => {
                    let mut appname: [WCHAR; 256] = [0; 256];
                    let mut other: [WCHAR; 256] = [0; 256];
                    LoadStringW(
                        board.hInst,
                        IDS_APPNAME as UINT,
                        &raw mut appname as LPWSTR,
                        (core::mem::size_of::<[WCHAR; 256]>()
                            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
                    );
                    LoadStringW(
                        board.hInst,
                        IDS_ABOUT as UINT,
                        &raw mut other as LPWSTR,
                        (core::mem::size_of::<[WCHAR; 256]>()
                            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
                    );
                    ShellAboutW(
                        hWnd,
                        &raw mut appname as *mut WCHAR as LPCWSTR,
                        &raw mut other as *mut WCHAR as LPCWSTR,
                        LoadImageW(
                            board.hInst,
                            MAKEINTRESOURCEW!(IDI_WINEMINE) as LPCWSTR,
                            IMAGE_ICON as UINT,
                            48 as core::ffi::c_int,
                            48 as core::ffi::c_int,
                            LR_SHARED as UINT,
                        ) as HICON,
                    );
                    return 0 as LRESULT;
                }

                _ => {
                    WINE_TRACE!("Unknown WM_COMMAND command message received\n");
                }
            }
        }

        _ => {}
    }
    return DefWindowProcW(hWnd, msg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    mut hInst: HINSTANCE,
    mut hPrevInst: HINSTANCE,
    mut cmdline: LPWSTR,
    mut cmdshow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut msg: MSG = core::mem::zeroed();
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut haccel: HACCEL = core::ptr::null_mut();
    let mut appname: [WCHAR; 20] = [0; 20];

    LoadStringW(
        hInst,
        (IDS_APPNAME) as UINT,
        core::ptr::addr_of_mut!(appname) as LPWSTR,
        (core::mem::size_of::<[WCHAR; 20]>() / core::mem::size_of::<WCHAR>())
            as ::core::ffi::c_int,
    );

    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.style = (0) as UINT;
    wc.lpfnWndProc =
        Some(MainProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) as WNDPROC;
    wc.cbClsExtra = (0) as ::core::ffi::c_int;
    wc.cbWndExtra = (0) as ::core::ffi::c_int;
    wc.hInstance = hInst;
    wc.hIcon = LoadIconW(
        hInst,
        MAKEINTRESOURCEW!(IDI_WINEMINE) as LPCWSTR,
    );
    wc.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        MAKEINTRESOURCEW!(IDC_ARROW) as LPCWSTR,
    );
    wc.hbrBackground = GetSysColorBrush((COLOR_BTNFACE as i32)); //MOD for ROS
    wc.lpszMenuName = MAKEINTRESOURCEW!(IDM_WINEMINE) as LPCWSTR;
    wc.lpszClassName = core::ptr::addr_of_mut!(appname) as *mut WCHAR as LPCWSTR;
    wc.hIconSm = LoadImageW(
        hInst,
        MAKEINTRESOURCEW!(IDI_WINEMINE) as LPCWSTR,
        (IMAGE_ICON) as UINT,
        GetSystemMetrics((SM_CXSMICON as i32)),
        GetSystemMetrics((SM_CYSMICON as i32)),
        (LR_SHARED) as UINT,
    ) as HICON;

    if RegisterClassExW(core::ptr::addr_of_mut!(wc)) == 0 {
        ExitProcess((1) as UINT);
    }
    hWnd = CreateWindowW!(
        core::ptr::addr_of_mut!(appname) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(appname) as *mut WCHAR as LPCWSTR,
        wnd_style,
        (CW_USEDEFAULT) as ::core::ffi::c_int,
        (CW_USEDEFAULT) as ::core::ffi::c_int,
        (CW_USEDEFAULT) as ::core::ffi::c_int,
        (CW_USEDEFAULT) as ::core::ffi::c_int,
        0,
        0,
        hInst,
        core::ptr::null_mut(),
    );

    if hWnd.is_null() {
        ExitProcess((1) as UINT);
    }

    ShowWindow(hWnd, cmdshow);
    UpdateWindow(hWnd);

    haccel = LoadAcceleratorsW(
        hInst,
        MAKEINTRESOURCEW!(IDA_WINEMINE) as LPCWSTR,
    );
    SetTimer(
        hWnd,
        (ID_TIMER) as UINT_PTR,
        (1000) as UINT,
        None,
    );

    while GetMessageW(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        (0) as UINT,
        (0) as UINT,
    ) != 0
    {
        if TranslateAcceleratorW(hWnd, haccel, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
        }
        DispatchMessageW(core::ptr::addr_of_mut!(msg));
    }
    return msg.wParam as ::core::ffi::c_int;
}
