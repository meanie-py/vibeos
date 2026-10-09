use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::comctl32undoc::Free;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_taskdialog);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_STATICW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0t\0a\0t\0i\0c\0\0\0")
};

pub const WC_LINK: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"S\0y\0s\0L\0i\0n\0k\0\0\0")
};

static mut DIALOG_MIN_WIDTH: UINT = 240 as UINT;

static mut DIALOG_SPACING: UINT = 5 as UINT;

static mut DIALOG_BUTTON_WIDTH: UINT = 50 as UINT;

static mut DIALOG_BUTTON_HEIGHT: UINT = 14 as UINT;

static mut DIALOG_EXPANDO_ICON_WIDTH: UINT = 10 as UINT;

static mut DIALOG_EXPANDO_ICON_HEIGHT: UINT = 10 as UINT;

static mut DIALOG_TIMER_MS: UINT = 200 as UINT;

static mut ID_TIMER: UINT = 1 as UINT;


#[repr(C)]
pub struct taskdialog_info_m {
    pub x_baseunit: LONG,
    pub y_baseunit: LONG,
    pub h_spacing: LONG,
    pub v_spacing: LONG,
}

#[repr(C)]
pub struct taskdialog_info {
    pub hwnd: HWND,
    pub taskconfig: *const TASKDIALOGCONFIG,
    pub last_timer_tick: DWORD,
    pub font: HFONT,
    pub main_instruction_font: HFONT,
    /* Control handles */
    pub main_icon: HWND,
    pub main_instruction: HWND,
    pub content: HWND,
    pub progress_bar: HWND,
    pub radio_buttons: *mut HWND,
    pub radio_button_count: INT,
    pub command_links: *mut HWND,
    pub command_link_count: INT,
    pub expanded_info: HWND,
    pub expando_button: HWND,
    pub verification_box: HWND,
    pub footer_icon: HWND,
    pub footer_text: HWND,
    pub buttons: *mut HWND,
    pub button_count: INT,
    pub default_button: HWND,
    /* Dialog metrics */
    pub m: taskdialog_info_m,
    pub selected_radio_id: INT,
    pub verification_checked: BOOL,
    pub expanded: BOOL,
    pub has_cancel: BOOL,
    pub expanded_text: *mut WCHAR,
    pub collapsed_text: *mut WCHAR,
}

#[repr(C)]
pub struct button_layout_info {
    pub width: LONG,
    pub line: LONG,
}

unsafe extern "C" fn taskdialog_du_to_px(dialog_info: *mut taskdialog_info, width: *mut LONG, height: *mut LONG) {
    if !width.is_null() {
        *width = MulDiv(*width, (*dialog_info).m.x_baseunit, 4);
    }
    if !height.is_null() {
        *height = MulDiv(*height, (*dialog_info).m.y_baseunit, 8);
    }
}

unsafe extern "C" fn template_write_data(ptr: *mut *mut i8, src: *const core::ffi::c_void, size: u32) {
    memcpy(*ptr as *mut core::ffi::c_void, src, size as usize);
    *ptr = (*ptr).add(size as usize);
}

unsafe extern "C" fn taskdialog_get_reference_rect(taskconfig: *const TASKDIALOGCONFIG, ret: *mut RECT) -> ::core::ffi::c_uint {
    let mut monitor: HMONITOR = MonitorFromWindow(
        if !(*taskconfig).hwndParent.is_null() {
            (*taskconfig).hwndParent
        } else {
            GetActiveWindow()
        },
        MONITOR_DEFAULTTOPRIMARY as DWORD,
    );
    let mut info: MONITORINFO = core::mem::zeroed();

    info.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(monitor, core::ptr::addr_of_mut!(info));

    if ((*taskconfig).dwFlags as u32 & TDF_POSITION_RELATIVE_TO_WINDOW) != 0 && !(*taskconfig).hwndParent.is_null() {
        GetWindowRect((*taskconfig).hwndParent, ret);
    } else {
        *ret = info.rcWork;
    }

    return (info.rcWork.right - info.rcWork.left) as ::core::ffi::c_uint;
}

unsafe extern "C" fn taskdialog_get_exe_name(mut name: *mut WCHAR, mut length: DWORD) -> *mut WCHAR {
    let mut len: DWORD = GetModuleFileNameW(core::ptr::null_mut(), name, length);
    if len != 0 && len < length {
        let mut p: *mut WCHAR = core::ptr::null_mut();
        p = wcsrchr(name, '/' as WCHAR) as *mut WCHAR;
        if !p.is_null() {
            name = p.offset(1 as isize);
        }
        p = wcsrchr(name, '\\' as WCHAR) as *mut WCHAR;
        if !p.is_null() {
            name = p.offset(1 as isize);
        }
        return name;
    } else {
        return core::ptr::null_mut();
    }
}

unsafe extern "C" fn create_taskdialog_template(taskconfig: *const TASKDIALOGCONFIG) -> *mut DLGTEMPLATE {
    let mut size: ::core::ffi::c_uint;
    let mut title_size: ::core::ffi::c_uint;
    static mut fontsize: WORD = 0x7fff;
    static mut emptyW: [WCHAR; 1] = [0];
    let mut titleW: *const WCHAR = core::ptr::null();
    let mut template: *mut DLGTEMPLATE = core::ptr::null_mut();
    let mut pathW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ptr: *mut ::core::ffi::c_char;

    /* Window title */
    if (*taskconfig).pszWindowTitle.is_null() {
        titleW = taskdialog_get_exe_name(
            core::ptr::addr_of_mut!(pathW) as *mut WCHAR,
            ARRAY_SIZE!(pathW) as DWORD,
        );
    } else if IS_INTRESOURCE!((*taskconfig).pszWindowTitle) != false {
        if LoadStringW(
            (*taskconfig).hInstance,
            LOWORD!((*taskconfig).pszWindowTitle) as UINT,
            core::ptr::addr_of_mut!(titleW) as LPWSTR,
            0,
        ) == 0
        {
            titleW = taskdialog_get_exe_name(
                core::ptr::addr_of_mut!(pathW) as *mut WCHAR,
                ARRAY_SIZE!(pathW) as DWORD,
            );
        }
    } else {
        titleW = (*taskconfig).pszWindowTitle as *const WCHAR;
    }
    if titleW.is_null() {
        titleW = core::ptr::addr_of!(emptyW) as *const WCHAR;
    }
    title_size = ((lstrlenW(titleW as LPCWSTR) + 1) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_uint;

    size = (core::mem::size_of::<DLGTEMPLATE>() as usize).wrapping_add(
        (2 as usize).wrapping_mul(core::mem::size_of::<WORD>()),
    ) as ::core::ffi::c_uint;
    size = size.wrapping_add(title_size);
    size = size.wrapping_add(2 as ::core::ffi::c_uint); /* font size */

    template = Alloc(size as DWORD) as *mut DLGTEMPLATE;
    if template.is_null() {
        return core::ptr::null_mut();
    }

    (*template).style = (DS_MODALFRAME | DS_SETFONT | WS_CAPTION | WS_VISIBLE | WS_SYSMENU) as DWORD;
    if ((*taskconfig).dwFlags as DWORD & TDF_CAN_BE_MINIMIZED as DWORD) != 0 {
        (*template).style |= WS_MINIMIZEBOX as DWORD;
    }
    if ((*taskconfig).dwFlags as DWORD & TDF_NO_SET_FOREGROUND as DWORD) == 0 {
        (*template).style |= DS_SETFOREGROUND as DWORD;
    }
    if ((*taskconfig).dwFlags as DWORD & TDF_RTL_LAYOUT as DWORD) != 0 {
        (*template).dwExtendedStyle = (WS_EX_LAYOUTRTL | WS_EX_RIGHT | WS_EX_RTLREADING) as DWORD;
    }

    ptr = template.offset(1 as isize) as *mut ::core::ffi::c_char;
    ptr = ptr.offset(2 as isize); /* menu */
    ptr = ptr.offset(2 as isize); /* class */
    template_write_data(
        core::ptr::addr_of_mut!(ptr),
        titleW as *const ::core::ffi::c_void,
        title_size,
    );
    template_write_data(
        core::ptr::addr_of_mut!(ptr),
        core::ptr::addr_of!(fontsize) as *const ::core::ffi::c_void,
        core::mem::size_of::<WORD>() as ::core::ffi::c_uint,
    );

    return template;
}

unsafe extern "C" fn taskdialog_find_button(buttons: *mut HWND, count: INT, id: INT) -> HWND {
    let mut button_id: INT;
    let mut i: INT;

    i = 0;
    while i < count {
        button_id = GetWindowLongW(*buttons.offset(i as isize), GWLP_ID) as INT;
        if button_id == id {
            return *buttons.offset(i as isize);
        }
        i += 1;
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn taskdialog_enable_button(dialog_info: *const taskdialog_info, id: INT, enable: BOOL) {
    let mut hwnd: HWND = taskdialog_find_button((*dialog_info).command_links, (*dialog_info).command_link_count, id);
    if hwnd.is_null() {
        hwnd = taskdialog_find_button((*dialog_info).buttons, (*dialog_info).button_count, id);
    }
    if !hwnd.is_null() {
        EnableWindow(hwnd, enable);
    }
}

unsafe extern "C" fn taskdialog_click_button(dialog_info: *mut taskdialog_info, id: INT) {
    if taskdialog_notify(dialog_info, TDN_BUTTON_CLICKED, id as WPARAM, 0) == S_OK {
        EndDialog((*dialog_info).hwnd, id as INT_PTR);
    }
}

unsafe extern "C" fn taskdialog_button_set_shield(dialog_info: *const taskdialog_info, id: INT, elevate: BOOL) {
    let mut hwnd: HWND = taskdialog_find_button((*dialog_info).command_links, (*dialog_info).command_link_count, id);
    if hwnd.is_null() {
        hwnd = taskdialog_find_button((*dialog_info).buttons, (*dialog_info).button_count, id);
    }
    if !hwnd.is_null() {
        SendMessageW(hwnd, BCM_SETSHIELD as UINT, 0 as WPARAM, elevate as LPARAM);
    }
}

unsafe extern "C" fn taskdialog_enable_radio_button(dialog_info: *const taskdialog_info, id: INT, enable: BOOL) {
    let hwnd: HWND = taskdialog_find_button((*dialog_info).radio_buttons, (*dialog_info).radio_button_count, id);
    if !hwnd.is_null() {
        EnableWindow(hwnd, enable);
    }
}

unsafe extern "C" fn taskdialog_click_radio_button(dialog_info: *const taskdialog_info, id: INT) {
    let hwnd: HWND = taskdialog_find_button(
        (*dialog_info).radio_buttons,
        (*dialog_info).radio_button_count,
        id,
    );
    if !hwnd.is_null() {
        SendMessageW(hwnd, BM_CLICK as UINT, 0 as WPARAM, 0 as LPARAM);
    }
}

unsafe extern "C" fn taskdialog_notify(dialog_info: *mut taskdialog_info, notification: UINT, wparam: WPARAM, lparam: LPARAM) -> HRESULT {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let pfcallback = (*taskconfig).pfCallback;
    return if pfcallback.is_some() {
        pfcallback.unwrap_unchecked()(
            (*dialog_info).hwnd,
            notification,
            wparam,
            lparam,
            (*taskconfig).lpCallbackData,
        )
    } else {
        S_OK
    };
}

unsafe extern "C" fn taskdialog_move_controls_vertically(parent: HWND, controls: *mut HWND, count: INT, offset: INT) {
    let mut rect: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut i: INT;

    i = 0;
    while i < count {
        if (*controls.offset(i as isize)).is_null() {
            i += 1;
            continue;
        }

        GetWindowRect(*controls.offset(i as isize), core::ptr::addr_of_mut!(rect));
        pt.x = rect.left;
        pt.y = rect.top;
        MapWindowPoints(HWND_DESKTOP, parent, core::ptr::addr_of_mut!(pt), 1);
        SetWindowPos(
            *controls.offset(i as isize),
            core::ptr::null_mut(),
            pt.x,
            pt.y + offset,
            0,
            0,
            (SWP_NOSIZE | SWP_NOZORDER) as UINT,
        );
        i += 1;
    }
}

unsafe extern "C" fn taskdialog_toggle_expando_control(mut dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut text: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut info_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut height: INT = 0;
    let mut offset: INT = 0;

    (*dialog_info).expanded = ((*dialog_info).expanded == 0) as BOOL;
    text = if (*dialog_info).expanded != 0 {
        (*dialog_info).expanded_text
    } else {
        (*dialog_info).collapsed_text
    };
    SendMessageW(
        (*dialog_info).expando_button,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        text as LPARAM,
    );
    ShowWindow(
        (*dialog_info).expanded_info,
        if (*dialog_info).expanded != 0 {
            (SW_SHOWDEFAULT) as i32
        } else {
            (SW_HIDE) as i32
        },
    );

    GetWindowRect((*dialog_info).expanded_info, &raw mut info_rect);
    /* If expanded information starts up not expanded, call taskdialog_layout()
     * to to set size for expanded information control at least once */
    if IsRectEmpty(&raw mut info_rect as LPCRECT) != 0 {
        taskdialog_layout(dialog_info);
        return;
    }
    height = info_rect.bottom - info_rect.top + (*dialog_info).m.v_spacing;
    offset = if (*dialog_info).expanded != 0 {
        height
    } else {
        -height
    };

    /* Update vertical layout, move all controls after expanded information */
    /* Move dialog */
    GetWindowRect((*dialog_info).hwnd, &raw mut rect);
    SetWindowPos(
        (*dialog_info).hwnd,
        core::ptr::null_mut(),
        0,
        0,
        rect.right - rect.left,
        rect.bottom - rect.top + offset,
        (SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );
    /* Move controls */
    if ((*taskconfig).dwFlags & (TDF_EXPAND_FOOTER_AREA as i32)) == 0 {
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            &raw mut (*dialog_info).progress_bar,
            1,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            &raw mut (*dialog_info).expando_button,
            1,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            &raw mut (*dialog_info).verification_box,
            1,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            &raw mut (*dialog_info).footer_icon,
            1,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            &raw mut (*dialog_info).footer_text,
            1,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            (*dialog_info).buttons,
            (*dialog_info).button_count,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            (*dialog_info).radio_buttons,
            (*dialog_info).radio_button_count,
            offset,
        );
        taskdialog_move_controls_vertically(
            (*dialog_info).hwnd,
            (*dialog_info).command_links,
            (*dialog_info).command_link_count,
            offset,
        );
    }
}

unsafe extern "C" fn taskdialog_on_button_click(dialog_info: *mut taskdialog_info, hwnd: HWND, id: WORD) {
    let mut command_id: INT = 0;
    let mut button: HWND = core::ptr::null_mut();
    let mut radio_button: HWND = core::ptr::null_mut();

    /* Prefer the id from hwnd because the id from WM_COMMAND is truncated to WORD */
    command_id = if !hwnd.is_null() {
        GetWindowLongW(hwnd, GWLP_ID) as INT
    } else {
        id as INT
    };

    if !hwnd.is_null() && hwnd == (*dialog_info).expando_button {
        taskdialog_toggle_expando_control(dialog_info);
        taskdialog_notify(
            dialog_info,
            TDN_EXPANDO_BUTTON_CLICKED as UINT,
            (*dialog_info).expanded as WPARAM,
            0 as LPARAM,
        );
        return;
    }

    if !hwnd.is_null() && hwnd == (*dialog_info).verification_box {
        (*dialog_info).verification_checked = ((*dialog_info).verification_checked == 0) as BOOL;
        taskdialog_notify(
            dialog_info,
            TDN_VERIFICATION_CLICKED as UINT,
            (*dialog_info).verification_checked as WPARAM,
            0 as LPARAM,
        );
        return;
    }

    radio_button = taskdialog_find_button(
        (*dialog_info).radio_buttons,
        (*dialog_info).radio_button_count,
        command_id,
    );
    if !radio_button.is_null() {
        (*dialog_info).selected_radio_id = command_id;
        taskdialog_notify(
            dialog_info,
            TDN_RADIO_BUTTON_CLICKED as UINT,
            command_id as WPARAM,
            0 as LPARAM,
        );
        return;
    }

    button = taskdialog_find_button(
        (*dialog_info).command_links,
        (*dialog_info).command_link_count,
        command_id,
    );
    if button.is_null() {
        button = taskdialog_find_button(
            (*dialog_info).buttons,
            (*dialog_info).button_count,
            command_id,
        );
    }
    if button.is_null() && (command_id) as u32 == IDOK {
        button = if (*dialog_info).command_link_count > 0 {
            *(*dialog_info).command_links.offset(0 as isize)
        } else {
            *(*dialog_info).buttons.offset(0 as isize)
        };
        command_id = GetWindowLongW(button, GWLP_ID) as INT;
    }

    if !button.is_null()
        && taskdialog_notify(
            dialog_info,
            TDN_BUTTON_CLICKED as UINT,
            command_id as WPARAM,
            0 as LPARAM,
        ) == S_OK
    {
        EndDialog((*dialog_info).hwnd, command_id as INT_PTR);
    }
}

unsafe extern "C" fn taskdialog_gettext(dialog_info: *mut taskdialog_info, user_resource: BOOL, text: *const WCHAR) -> *mut WCHAR {
    let mut textW: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut length: INT = 0;
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    if IS_INTRESOURCE!(text) != false {
        length = LoadStringW(
            if user_resource != 0 {
                (*(*dialog_info).taskconfig).hInstance
            } else {
                COMCTL32_hModule as HINSTANCE
            },
            text as UINT_PTR as UINT,
            ::core::ptr::addr_of_mut!(textW) as LPWSTR,
            0,
        ) as INT;
        if length == 0 {
            return ::core::ptr::null_mut::<WCHAR>();
        }
    } else {
        textW = text;
        length = lstrlenW(textW as LPCWSTR) as INT;
    }
    ret = Alloc(
        ((length + 1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
    ) as *mut WCHAR;
    if !ret.is_null() {
        memcpy(
            ret as *mut ::core::ffi::c_void,
            textW as *const ::core::ffi::c_void,
            (length as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize),
        );
    }
    return ret;
}

unsafe extern "C" fn taskdialog_hyperlink_enabled(dialog_info: *mut taskdialog_info) -> BOOL {
    return (((*(*dialog_info).taskconfig).dwFlags as BOOL)
        & (TDF_ENABLE_HYPERLINKS as ::core::ffi::c_int as BOOL)) as BOOL;
}

unsafe extern "C" fn taskdialog_use_command_link(dialog_info: *mut taskdialog_info) -> BOOL {
    return (*(*dialog_info).taskconfig).dwFlags as BOOL
        & (TDF_USE_COMMAND_LINKS as ::core::ffi::c_int as BOOL
            | TDF_USE_COMMAND_LINKS_NO_ICON as ::core::ffi::c_int as BOOL);
}

unsafe extern "C" fn taskdialog_get_label_size(
    mut dialog_info: *mut taskdialog_info,
    mut hwnd: HWND,
    mut max_width: LONG,
    mut size: *mut SIZE,
    mut syslink: BOOL,
) {
    let mut style: DWORD = (DT_EXPANDTABS | DT_CALCRECT | DT_WORDBREAK) as DWORD;
    let mut hfont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut old_hfont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut text: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut text_length: INT = 0;

    if syslink != 0 {
        SendMessageW(
            hwnd,
            LM_GETIDEALSIZE as UINT,
            max_width as WPARAM,
            size as LPARAM,
        );
        return;
    }

    if (((*(*dialog_info).taskconfig).dwFlags as DWORD) & (TDF_RTL_LAYOUT as DWORD)) != 0 {
        style |= (DT_RIGHT | DT_RTLREADING) as DWORD;
    } else {
        style |= DT_LEFT as DWORD;
    }

    hfont = SendMessageW(
        hwnd,
        WM_GETFONT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HFONT;
    text_length = GetWindowTextLengthW(hwnd) as INT;
    text = Alloc(
        (((text_length + 1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize)) as DWORD,
    ) as *mut WCHAR;
    if text.is_null() {
        (*size).cx = 0 as LONG;
        (*size).cy = 0 as LONG;
        return;
    }
    GetWindowTextW(hwnd, text as LPWSTR, text_length + 1);
    hdc = GetDC(hwnd);
    old_hfont = SelectObject(hdc, hfont as HGDIOBJ) as HFONT;
    rect.right = max_width;
    (*size).cy = DrawTextW(
        hdc,
        text as LPCWSTR,
        text_length,
        &raw mut rect,
        style as UINT,
    ) as LONG;
    (*size).cx = min(max_width, rect.right - rect.left);
    if !old_hfont.is_null() {
        SelectObject(hdc, old_hfont as HGDIOBJ);
    }
    ReleaseDC(hwnd, hdc);
    Free(text as LPVOID);
}

unsafe extern "C" fn taskdialog_get_button_size(hwnd: HWND, max_width: LONG, size: *mut SIZE) {
    (*size).cx = max_width;
    (*size).cy = 0 as LONG;
    SendMessageW(hwnd, BCM_GETIDEALSIZE as UINT, 0 as WPARAM, size as LPARAM);
}

unsafe extern "C" fn taskdialog_get_expando_size(
    mut dialog_info: *mut taskdialog_info,
    mut hwnd: HWND,
    mut size: *mut SIZE,
) {
    let mut style: DWORD = (DT_EXPANDTABS | DT_CALCRECT | DT_WORDBREAK) as DWORD;
    let mut hfont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut old_hfont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut rect: RECT = tagRECT {
        left: 0 as LONG,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut icon_width: LONG = 0;
    let mut icon_height: LONG = 0;
    let mut text_offset: LONG = 0;
    let mut max_width: LONG = 0;
    let mut max_text_height: LONG = 0;
    hdc = GetDC(hwnd);
    hfont = SendMessageW(
        hwnd,
        WM_GETFONT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HFONT;
    old_hfont = SelectObject(hdc, hfont as HGDIOBJ) as HFONT;
    icon_width = DIALOG_EXPANDO_ICON_WIDTH as LONG;
    icon_height = DIALOG_EXPANDO_ICON_HEIGHT as LONG;
    taskdialog_du_to_px(dialog_info, &raw mut icon_width, &raw mut icon_height);
    GetCharWidthW(
        hdc,
        '0' as i32 as UINT,
        '0' as i32 as UINT,
        &raw mut text_offset,
    );
    text_offset /= 2 as ::core::ffi::c_int;
    if ((*(*dialog_info).taskconfig).dwFlags as ::core::ffi::c_int
        & TDF_RTL_LAYOUT as ::core::ffi::c_int)
        != 0
    {
        style |= (DT_RIGHT | DT_RTLREADING) as DWORD;
    } else {
        style |= DT_LEFT as DWORD;
    }
    max_width = (DIALOG_MIN_WIDTH / 2) as LONG;
    taskdialog_du_to_px(
        dialog_info,
        &raw mut max_width,
        ::core::ptr::null_mut::<LONG>(),
    );
    rect.right = max_width - icon_width - text_offset;
    max_text_height = DrawTextW(
        hdc,
        (*dialog_info).expanded_text as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut rect,
        style as UINT,
    ) as LONG;
    (*size).cy = max(max_text_height, icon_height);
    (*size).cx = rect.right - rect.left;
    rect.right = max_width - icon_width - text_offset;
    max_text_height = DrawTextW(
        hdc,
        (*dialog_info).collapsed_text as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut rect,
        style as UINT,
    ) as LONG;
    (*size).cy = max((*size).cy, max_text_height);
    (*size).cx = max((*size).cx, rect.right - rect.left);
    (*size).cx = min((*size).cx, max_width);
    if !old_hfont.is_null() {
        SelectObject(hdc, old_hfont as HGDIOBJ);
    }
    ReleaseDC(hwnd, hdc);
}

unsafe extern "C" fn taskdialog_get_standard_icon(icon: LPCWSTR) -> ULONG_PTR {
    if icon == TD_WARNING_ICON as LPCWSTR {
        return IDI_WARNING as ULONG_PTR;
    } else if icon == TD_ERROR_ICON as LPCWSTR {
        return IDI_ERROR as ULONG_PTR;
    } else if icon == TD_INFORMATION_ICON as LPCWSTR {
        return IDI_INFORMATION as ULONG_PTR;
    } else if icon == TD_SHIELD_ICON as LPCWSTR {
        return IDI_SHIELD as ULONG_PTR;
    } else {
        return icon as ULONG_PTR;
    }
}

unsafe extern "C" fn taskdialog_set_icon(dialog_info: *mut taskdialog_info, element: INT, icon: HICON) {
    let flags: DWORD = ((*(*dialog_info).taskconfig).dwFlags) as u32;
    let mut cx: INT = 0 as INT;
    let mut cy: INT = 0 as INT;
    let mut hicon: HICON = core::ptr::null_mut();

    if icon.is_null() {
        return;
    }

    if ((flags & (TDF_USE_HICON_MAIN as DWORD)) != 0 && element == (TDIE_ICON_MAIN as INT))
        || ((flags & (TDF_USE_HICON_FOOTER as DWORD)) != 0 && element == (TDIE_ICON_FOOTER as INT))
    {
        hicon = icon;
    } else {
        if element == (TDIE_ICON_FOOTER as INT) {
            cx = GetSystemMetrics(SM_CXSMICON as INT) as INT;
            cy = GetSystemMetrics(SM_CYSMICON as INT) as INT;
        }
        hicon = LoadImageW(
            (*(*dialog_info).taskconfig).hInstance,
            icon as LPCWSTR,
            IMAGE_ICON as UINT,
            cx,
            cy,
            ((LR_SHARED as UINT) | (LR_DEFAULTSIZE as UINT)),
        ) as HICON;
        if hicon.is_null() {
            hicon = LoadImageW(
                core::ptr::null_mut(),
                taskdialog_get_standard_icon(icon as LPCWSTR) as LPCWSTR,
                IMAGE_ICON as UINT,
                cx,
                cy,
                ((LR_SHARED as UINT) | (LR_DEFAULTSIZE as UINT)),
            ) as HICON;
        }
    }

    if hicon.is_null() {
        return;
    }

    if element == (TDIE_ICON_MAIN as INT) {
        SendMessageW(
            (*dialog_info).hwnd,
            WM_SETICON as UINT,
            ICON_BIG as WPARAM,
            hicon as LPARAM,
        );
        SendMessageW(
            (*dialog_info).main_icon,
            STM_SETICON as UINT,
            hicon as WPARAM,
            0 as LPARAM,
        );
    } else if element == (TDIE_ICON_FOOTER as INT) {
        SendMessageW(
            (*dialog_info).footer_icon,
            STM_SETICON as UINT,
            hicon as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn taskdialog_set_element_text(mut dialog_info: *mut taskdialog_info, mut element: TASKDIALOG_ELEMENTS, mut text: *const WCHAR) {
    let mut hwnd: HWND = core::ptr::null_mut::<HWND__>();
    let mut textW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    if element as ::core::ffi::c_uint == TDE_CONTENT as ::core::ffi::c_int as ::core::ffi::c_uint {
        hwnd = (*dialog_info).content;
    } else if element as ::core::ffi::c_uint == TDE_EXPANDED_INFORMATION as ::core::ffi::c_int as ::core::ffi::c_uint {
        hwnd = (*dialog_info).expanded_info;
    } else if element as ::core::ffi::c_uint == TDE_FOOTER as ::core::ffi::c_int as ::core::ffi::c_uint {
        hwnd = (*dialog_info).footer_text;
    } else if element as ::core::ffi::c_uint == TDE_MAIN_INSTRUCTION as ::core::ffi::c_int as ::core::ffi::c_uint {
        hwnd = (*dialog_info).main_instruction;
    }
    if hwnd.is_null() {
        return;
    }
    textW = taskdialog_gettext(dialog_info, TRUE, text);
    SendMessageW(hwnd, WM_SETTEXT as UINT, 0 as WPARAM, textW as LPARAM);
    Free(textW as LPVOID);
}

unsafe extern "C" fn taskdialog_check_default_radio_buttons(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut default_button: HWND;

    if (*dialog_info).radio_button_count == 0 {
        return;
    }

    default_button = taskdialog_find_button(
        (*dialog_info).radio_buttons,
        (*dialog_info).radio_button_count,
        (*taskconfig).nDefaultRadioButton as INT,
    );

    if default_button.is_null() && ((*taskconfig).dwFlags & TDF_NO_DEFAULT_RADIO_BUTTON as INT) == 0 {
        default_button = *(*dialog_info)
            .radio_buttons
            .offset(0 as isize);
    }

    if !default_button.is_null() {
        SendMessageW(
            default_button,
            BM_SETCHECK as UINT,
            BST_CHECKED as WPARAM,
            0 as LPARAM,
        );
        taskdialog_on_button_click(dialog_info, default_button, 0 as WORD);
    }
}

unsafe extern "C" fn taskdialog_add_main_icon(dialog_info: *mut taskdialog_info) {
    if (*(*dialog_info).taskconfig).u.hMainIcon.is_null() {
        return;
    }
    (*dialog_info).main_icon = CreateWindowW!(
        WC_STATICW.as_ptr(),
        core::ptr::null_mut(),
        WS_CHILD | WS_VISIBLE | SS_ICON,
        0,
        0,
        0,
        0,
        (*dialog_info).hwnd,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    taskdialog_set_icon(
        dialog_info,
        (TDIE_ICON_MAIN) as INT,
        (*(*dialog_info).taskconfig).u.hMainIcon,
    );
}

unsafe extern "C" fn taskdialog_create_label(
    mut dialog_info: *mut taskdialog_info,
    mut text: *const WCHAR,
    mut font: HFONT,
    mut syslink: BOOL,
) -> HWND {
    let mut textW: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut hwnd: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut class: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut style: DWORD = (WS_CHILD | WS_VISIBLE) as DWORD;
    if text.is_null() {
        return ::core::ptr::null_mut::<HWND__>();
    }
    class = if syslink != 0 {
        WC_LINK.as_ptr()
    } else {
        WC_STATICW.as_ptr()
    };
    if syslink != 0 {
        style |= WS_TABSTOP as DWORD;
    }
    textW = taskdialog_gettext(dialog_info, TRUE, text);
    hwnd = CreateWindowW!(
        class as LPCWSTR,
        textW as LPCWSTR,
        style,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (*dialog_info).hwnd,
        ::core::ptr::null_mut::<HMENU__>(),
        core::ptr::null_mut(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    );
    Free(textW as LPVOID);
    SendMessageW(hwnd, WM_SETFONT as UINT, font as WPARAM, 0 as LPARAM);
    return hwnd;
}

unsafe extern "C" fn taskdialog_add_main_instruction(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();

    if (*taskconfig).pszMainInstruction.is_null() {
        return;
    }

    ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, ncm.cbSize, core::ptr::addr_of_mut!(ncm) as PVOID, 0);
    /* 1.25 times the height */
    ncm.lfMessageFont.lfHeight = (ncm.lfMessageFont.lfHeight as i32 * 5 / 4) as LONG;
    ncm.lfMessageFont.lfWeight = FW_BOLD as LONG;
    (*dialog_info).main_instruction_font = CreateFontIndirectW(core::ptr::addr_of_mut!(ncm.lfMessageFont));

    (*dialog_info).main_instruction =
        taskdialog_create_label(dialog_info, (*taskconfig).pszMainInstruction as *const WCHAR, (*dialog_info).main_instruction_font, FALSE);
}

unsafe extern "C" fn taskdialog_add_content(dialog_info: *mut taskdialog_info) {
    (*dialog_info).content = taskdialog_create_label(
        dialog_info,
        (*(*dialog_info).taskconfig).pszContent as *const WCHAR,
        (*dialog_info).font,
        taskdialog_hyperlink_enabled(dialog_info),
    );
}

unsafe extern "C" fn taskdialog_add_progress_bar(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut style: DWORD = PBS_SMOOTH | PBS_SMOOTHREVERSE | WS_CHILD | WS_VISIBLE;

    if ((*taskconfig).dwFlags as DWORD) & (TDF_SHOW_PROGRESS_BAR | TDF_SHOW_MARQUEE_PROGRESS_BAR) == 0 {
        return;
    }
    if ((*taskconfig).dwFlags as DWORD) & TDF_SHOW_MARQUEE_PROGRESS_BAR != 0 {
        style |= PBS_MARQUEE;
    }
    (*dialog_info).progress_bar = CreateWindowW!(
        PROGRESS_CLASSW.as_ptr(),
        core::ptr::null_mut(),
        style,
        0,
        0,
        0,
        0,
        (*dialog_info).hwnd,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

unsafe extern "C" fn taskdialog_add_radio_buttons(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    static mut style: DWORD = BS_AUTORADIOBUTTON | BS_MULTILINE | BS_TOP | WS_CHILD | WS_VISIBLE | WS_TABSTOP;
    let mut textW: *mut WCHAR;
    let mut i: INT;

    if (*taskconfig).cRadioButtons == 0 || (*taskconfig).pRadioButtons.is_null() {
        return;
    }

    (*dialog_info).radio_buttons = Alloc(
        ((*taskconfig).cRadioButtons as usize).wrapping_mul(::core::mem::size_of::<HWND>()) as DWORD,
    ) as *mut HWND;
    if (*dialog_info).radio_buttons.is_null() {
        return;
    }

    (*dialog_info).radio_button_count = (*taskconfig).cRadioButtons as INT;
    i = 0;
    while i < (*dialog_info).radio_button_count {
        textW = taskdialog_gettext(
            dialog_info,
            TRUE,
            (*(*taskconfig).pRadioButtons.offset(i as isize)).pszButtonText as *const WCHAR,
        );
        *(*dialog_info).radio_buttons.offset(i as isize) = CreateWindowW!(
            WC_BUTTONW.as_ptr(),
            textW as LPCWSTR,
            if i == 0 {
                style | WS_GROUP
            } else {
                style
            },
            0,
            0,
            0,
            0,
            (*dialog_info).hwnd,
            LongToHandle!((*(*taskconfig).pRadioButtons.offset(i as isize)).nButtonID as isize) as HMENU,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        SendMessageW(
            *(*dialog_info).radio_buttons.offset(i as isize),
            WM_SETFONT,
            (*dialog_info).font as WPARAM,
            0,
        );
        Free(textW as LPVOID);
        i += 1;
    }
}

unsafe extern "C" fn taskdialog_add_command_links(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let default_style: DWORD = BS_MULTILINE | BS_LEFT | BS_TOP | WS_CHILD | WS_VISIBLE | WS_TABSTOP;
    let mut style: DWORD;
    let mut is_default: BOOL;
    let mut textW: *mut WCHAR;
    let mut i: INT;

    if (*taskconfig).cButtons == 0 || (*taskconfig).pButtons.is_null() || taskdialog_use_command_link(dialog_info) == 0 {
        return;
    }

    (*dialog_info).command_links = Alloc(((*taskconfig).cButtons as usize).wrapping_mul(core::mem::size_of::<HWND>()) as DWORD) as *mut HWND;
    if (*dialog_info).command_links.is_null() {
        return;
    }

    (*dialog_info).command_link_count = (*taskconfig).cButtons as INT;
    i = 0 as INT;
    while i < (*dialog_info).command_link_count {
        is_default = ((*(*taskconfig).pButtons.offset(i as isize)).nButtonID == (*taskconfig).nDefaultButton) as BOOL;
        style = if is_default != 0 {
            default_style | BS_DEFCOMMANDLINK
        } else {
            default_style | BS_COMMANDLINK
        };
        textW = taskdialog_gettext(dialog_info, TRUE, (*(*taskconfig).pButtons.offset(i as isize)).pszButtonText as *const WCHAR);
        *(*dialog_info).command_links.offset(i as isize) = CreateWindowW!(
            WC_BUTTONW.as_ptr(),
            textW,
            style,
            0,
            0,
            0,
            0,
            (*dialog_info).hwnd,
            LongToHandle!((*(*taskconfig).pButtons.offset(i as isize)).nButtonID as c_long) as HMENU,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        SendMessageW(*(*dialog_info).command_links.offset(i as isize), WM_SETFONT, (*dialog_info).font as WPARAM, 0);
        Free(textW as LPVOID);

        if is_default != 0 && (*dialog_info).default_button.is_null() {
            (*dialog_info).default_button = *(*dialog_info).command_links.offset(i as isize);
        }
        i += 1;
    }
}

unsafe extern "C" fn taskdialog_add_expanded_info(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    if (*taskconfig).pszExpandedInformation.is_null() {
        return;
    }
    (*dialog_info).expanded = (((*taskconfig).dwFlags as ::core::ffi::c_int)
        & TDF_EXPANDED_BY_DEFAULT as ::core::ffi::c_int) as BOOL;
    (*dialog_info).expanded_info = taskdialog_create_label(
        dialog_info,
        (*taskconfig).pszExpandedInformation as *const WCHAR,
        (*dialog_info).font,
        taskdialog_hyperlink_enabled(dialog_info),
    );
    ShowWindow(
        (*dialog_info).expanded_info,
        if (*dialog_info).expanded != 0 {
            (SW_SHOWDEFAULT) as i32
        } else {
            (SW_HIDE) as i32
        },
    );
}

unsafe extern "C" fn taskdialog_add_expando_button(mut dialog_info: *mut taskdialog_info) {
    let mut taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut textW: *const WCHAR = ::core::ptr::null::<WCHAR>();
    if (*taskconfig).pszExpandedInformation.is_null() {
        return;
    }
    if (*taskconfig).pszCollapsedControlText.is_null()
        && (*taskconfig).pszExpandedControlText.is_null()
    {
        (*dialog_info).expanded_text = taskdialog_gettext(
            dialog_info,
            FALSE,
            MAKEINTRESOURCEW!(IDS_TD_EXPANDED) as *const WCHAR,
        );
        (*dialog_info).collapsed_text = taskdialog_gettext(
            dialog_info,
            FALSE,
            MAKEINTRESOURCEW!(IDS_TD_COLLAPSED) as *const WCHAR,
        );
    } else {
        textW = (if !(*taskconfig).pszExpandedControlText.is_null() {
            (*taskconfig).pszExpandedControlText
        } else {
            (*taskconfig).pszCollapsedControlText
        }) as *const WCHAR;
        (*dialog_info).expanded_text = taskdialog_gettext(
            dialog_info,
            TRUE,
            textW,
        );
        textW = (if !(*taskconfig).pszCollapsedControlText.is_null() {
            (*taskconfig).pszCollapsedControlText
        } else {
            (*taskconfig).pszExpandedControlText
        }) as *const WCHAR;
        (*dialog_info).collapsed_text = taskdialog_gettext(
            dialog_info,
            TRUE,
            textW,
        );
    }
    textW = if (*dialog_info).expanded != 0 {
        (*dialog_info).expanded_text
    } else {
        (*dialog_info).collapsed_text
    };
    (*dialog_info).expando_button = CreateWindowW!(
        WC_BUTTONW.as_ptr(),
        textW,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW,
        0,
        0,
        0,
        0,
        (*dialog_info).hwnd,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    SendMessageW(
        (*dialog_info).expando_button,
        WM_SETFONT,
        (*dialog_info).font as WPARAM,
        0,
    );
}

unsafe extern "C" fn taskdialog_add_verification_box(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    static mut style: DWORD = (BS_AUTOCHECKBOX | BS_MULTILINE | BS_LEFT | BS_TOP | WS_CHILD | WS_VISIBLE | WS_TABSTOP) as DWORD;
    let mut textW: *mut WCHAR = core::ptr::null_mut();

    /* TDF_VERIFICATION_FLAG_CHECKED works even if pszVerificationText is not set */
    if ((*taskconfig).dwFlags as DWORD & TDF_VERIFICATION_FLAG_CHECKED) != 0 {
        (*dialog_info).verification_checked = TRUE as BOOL;
    }

    if (*taskconfig).pszVerificationText.is_null() {
        return;
    }

    textW = taskdialog_gettext(dialog_info, TRUE as BOOL, (*taskconfig).pszVerificationText as *const WCHAR);
    (*dialog_info).verification_box = CreateWindowW!(WC_BUTTONW as LPCWSTR, textW as LPCWSTR, style, 0 as core::ffi::c_int, 0 as core::ffi::c_int, 0 as core::ffi::c_int, 0 as core::ffi::c_int, (*dialog_info).hwnd, 0 as HMENU, 0 as HINSTANCE, 0 as *mut core::ffi::c_void);
    SendMessageW((*dialog_info).verification_box, WM_SETFONT as UINT, (*dialog_info).font as WPARAM, 0 as LPARAM);
    Free(textW as LPVOID);

    if ((*taskconfig).dwFlags as DWORD & TDF_VERIFICATION_FLAG_CHECKED) != 0 {
        SendMessageW((*dialog_info).verification_box, BM_SETCHECK as UINT, BST_CHECKED as WPARAM, 0 as LPARAM);
    }
}

unsafe extern "C" fn taskdialog_add_button(dialog_info: *mut taskdialog_info, button: *mut HWND, id: INT_PTR, text: *const WCHAR, custom_button: BOOL) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut textW: *mut WCHAR;

    textW = taskdialog_gettext(dialog_info, custom_button, text);
    *button = CreateWindowW!(WC_BUTTONW.as_ptr(), textW, WS_CHILD | WS_VISIBLE | WS_TABSTOP, 0, 0, 0, 0, (*dialog_info).hwnd,
                            id as HMENU, core::ptr::null_mut(), core::ptr::null_mut());
    Free(textW as LPVOID);
    SendMessageW(*button, WM_SETFONT, (*dialog_info).font as WPARAM, 0);

    if id == (*taskconfig).nDefaultButton as INT_PTR && (*dialog_info).default_button.is_null() {
        (*dialog_info).default_button = *button;
    }
}

unsafe extern "C" fn taskdialog_add_buttons(dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let use_command_links: BOOL = taskdialog_use_command_link(dialog_info);
    let flags: DWORD = (*taskconfig).dwCommonButtons as DWORD;
    let mut count: INT = 0;
    let mut max_count: INT = 0;

    max_count = 6 as INT;
    if use_command_links == 0 && (*taskconfig).cButtons != 0 && !(*taskconfig).pButtons.is_null() {
        max_count = (max_count as UINT).wrapping_add((*taskconfig).cButtons) as INT;
    }

    (*dialog_info).buttons = Alloc(
        (max_count as usize).wrapping_mul(core::mem::size_of::<HWND>()) as DWORD,
    ) as *mut HWND;
    if (*dialog_info).buttons.is_null() {
        return;
    }

    count = 0 as INT;
    while use_command_links == 0 && (count as UINT) < (*taskconfig).cButtons {
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(count as isize),
            (*(*taskconfig).pButtons.offset(count as isize)).nButtonID as INT_PTR,
            (*(*taskconfig).pButtons.offset(count as isize)).pszButtonText as *const WCHAR,
            TRUE as BOOL,
        );
        count += 1;
    }

    if (flags & (TDCBF_OK_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDOK as INT_PTR,
            (3003 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }
    if (flags & (TDCBF_YES_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDYES as INT_PTR,
            (3000 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }
    if (flags & (TDCBF_NO_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDNO as INT_PTR,
            (3001 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }
    if (flags & (TDCBF_RETRY_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDRETRY as INT_PTR,
            (3002 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }
    if (flags & (TDCBF_CANCEL_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDCANCEL as INT_PTR,
            (3004 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }
    if (flags & (TDCBF_CLOSE_BUTTON)) != 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDCLOSE as INT_PTR,
            (3005 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }

    if count == 0 && (*dialog_info).command_link_count == 0 {
        let fresh = count;
        count += 1;
        taskdialog_add_button(
            dialog_info,
            (*dialog_info).buttons.offset(fresh as isize),
            IDOK as INT_PTR,
            (3003 as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as *const WCHAR,
            FALSE,
        );
    }

    (*dialog_info).button_count = count;
}

unsafe extern "C" fn taskdialog_add_footer_icon(dialog_info: *mut taskdialog_info) {
    if (*(*dialog_info).taskconfig).u2.hFooterIcon.is_null() {
        return;
    }

    (*dialog_info).footer_icon = CreateWindowW!(
        WC_STATICW.as_ptr(),
        core::ptr::null_mut(),
        WS_CHILD | WS_VISIBLE | SS_ICON,
        0,
        0,
        0,
        0,
        (*dialog_info).hwnd,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    taskdialog_set_icon(
        dialog_info,
        (TDIE_ICON_FOOTER) as INT,
        (*(*dialog_info).taskconfig).u2.hFooterIcon,
    );
}

unsafe extern "C" fn taskdialog_add_footer_text(dialog_info: *mut taskdialog_info) {
    (*dialog_info).footer_text = taskdialog_create_label(
        dialog_info,
        (*(*dialog_info).taskconfig).pszFooter as *const WCHAR,
        (*dialog_info).font,
        taskdialog_hyperlink_enabled(dialog_info),
    );
}

unsafe extern "C" fn taskdialog_get_dialog_width(dialog_info: *mut taskdialog_info) -> LONG {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let syslink: BOOL = taskdialog_hyperlink_enabled(dialog_info);
    let mut max_width: LONG = 0;
    let mut main_icon_width: LONG = 0;
    let mut screen_width: LONG = 0;
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };

    screen_width = taskdialog_get_reference_rect(taskconfig, &raw mut rect) as LONG;
    if ((*taskconfig).dwFlags & (TDF_SIZE_TO_CONTENT as i32)) != 0 && (*taskconfig).cxWidth == 0
    {
        max_width = DIALOG_MIN_WIDTH as LONG;
        taskdialog_du_to_px(dialog_info, &raw mut max_width, core::ptr::null_mut::<LONG>());
        main_icon_width = (*dialog_info).m.h_spacing;
        if !(*dialog_info).main_icon.is_null() {
            main_icon_width += GetSystemMetrics((SM_CXICON) as i32);
        }
        if !(*dialog_info).content.is_null() {
            taskdialog_get_label_size(
                dialog_info,
                (*dialog_info).content,
                0,
                &raw mut size,
                syslink,
            );
            max_width = max(
                max_width,
                size.cx + main_icon_width + (*dialog_info).m.h_spacing * 2,
            );
        }
    } else {
        max_width = max(((*taskconfig).cxWidth) as i32, DIALOG_MIN_WIDTH as LONG);
        taskdialog_du_to_px(dialog_info, &raw mut max_width, core::ptr::null_mut::<LONG>());
    }
    max_width = min(max_width, screen_width);
    return max_width;
}

unsafe extern "C" fn taskdialog_label_layout(
    mut dialog_info: *mut taskdialog_info,
    mut hwnd: HWND,
    mut start_x: INT,
    mut dialog_width: LONG,
    mut dialog_height: *mut LONG,
    mut syslink: BOOL,
) {
    let mut x: LONG = 0;
    let mut y: LONG = 0;
    let mut max_width: LONG = 0;
    let mut size: SIZE = tagSIZE { cx: 0, cy: 0 };
    if hwnd.is_null() {
        return;
    }
    x = start_x as LONG + (*dialog_info).m.h_spacing;
    y = *dialog_height + (*dialog_info).m.v_spacing;
    max_width = dialog_width - x - (*dialog_info).m.h_spacing;
    taskdialog_get_label_size(dialog_info, hwnd, max_width, &raw mut size, syslink);
    SetWindowPos(
        hwnd,
        core::ptr::null_mut(),
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        size.cx as ::core::ffi::c_int,
        size.cy as ::core::ffi::c_int,
        SWP_NOZORDER,
    );
    *dialog_height = y + size.cy;
}

unsafe extern "C" fn taskdialog_layout(mut dialog_info: *mut taskdialog_info) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let syslink: BOOL = taskdialog_hyperlink_enabled(dialog_info);
    static mut first_time: BOOL = TRUE;
    let mut ref_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut dialog_width: LONG = 0;
    let mut dialog_height: LONG = 0;
    let mut h_spacing: LONG = 0;
    let mut v_spacing: LONG = 0;
    let mut main_icon_right: LONG = 0;
    let mut main_icon_bottom: LONG = 0;
    let mut expando_right: LONG = 0;
    let mut expando_bottom: LONG = 0;
    let mut button_layout_infos: *mut button_layout_info = core::ptr::null_mut::<button_layout_info>();
    let mut button_min_width: LONG = 0;
    let mut button_height: LONG = 0;
    let mut line_widths: *mut LONG = core::ptr::null_mut::<LONG>();
    let mut line_count: LONG = 0;
    let mut align: LONG = 0;
    let mut footer_icon_right: LONG = 0;
    let mut footer_icon_bottom: LONG = 0;
    let mut x: LONG = 0;
    let mut y: LONG = 0;
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut i: INT = 0;

    taskdialog_get_reference_rect((*dialog_info).taskconfig, core::ptr::addr_of_mut!(ref_rect));
    dialog_width = taskdialog_get_dialog_width(dialog_info);

    h_spacing = (*dialog_info).m.h_spacing;
    v_spacing = (*dialog_info).m.v_spacing;

    /* Main icon */
    main_icon_right = 0;
    main_icon_bottom = 0;
    if !(*dialog_info).main_icon.is_null() {
        x = h_spacing;
        y = dialog_height + v_spacing;
        size.cx = GetSystemMetrics((SM_CXICON) as i32) as LONG;
        size.cy = GetSystemMetrics((SM_CYICON) as i32) as LONG;
        SetWindowPos(
            (*dialog_info).main_icon,
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        main_icon_right = x + size.cx;
        main_icon_bottom = y + size.cy;
    }

    /* Main instruction */
    taskdialog_label_layout(
        dialog_info,
        (*dialog_info).main_instruction,
        main_icon_right,
        dialog_width,
        core::ptr::addr_of_mut!(dialog_height),
        FALSE,
    );

    /* Content */
    taskdialog_label_layout(
        dialog_info,
        (*dialog_info).content,
        main_icon_right,
        dialog_width,
        core::ptr::addr_of_mut!(dialog_height),
        syslink,
    );

    /* Expanded information */
    if ((*taskconfig).dwFlags & (TDF_EXPAND_FOOTER_AREA as i32)) == 0 && (*dialog_info).expanded != 0 {
        taskdialog_label_layout(
            dialog_info,
            (*dialog_info).expanded_info,
            main_icon_right,
            dialog_width,
            core::ptr::addr_of_mut!(dialog_height),
            syslink,
        );
    }

    /* Progress bar */
    if !(*dialog_info).progress_bar.is_null() {
        x = main_icon_right + h_spacing;
        y = dialog_height + v_spacing;
        size.cx = dialog_width - x - h_spacing;
        size.cy = GetSystemMetrics((SM_CYVSCROLL) as i32) as LONG;
        SetWindowPos(
            (*dialog_info).progress_bar,
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        dialog_height = y + size.cy;
    }

    /* Radio buttons */
    i = 0;
    while i < (*dialog_info).radio_button_count {
        x = main_icon_right + h_spacing;
        y = dialog_height + v_spacing;
        taskdialog_get_button_size(
            *(*dialog_info).radio_buttons.offset(i as isize),
            dialog_width - x - h_spacing,
            core::ptr::addr_of_mut!(size),
        );
        size.cx = dialog_width - x - h_spacing;
        SetWindowPos(
            *(*dialog_info).radio_buttons.offset(i as isize),
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        dialog_height = y + size.cy;
        i += 1;
    }

    /* Command links */
    i = 0;
    while i < (*dialog_info).command_link_count {
        x = main_icon_right + h_spacing;
        y = dialog_height;
        /* Only add spacing for the first command links. There is no vertical spacing between command links */
        if i == 0 {
            y += v_spacing;
        }
        taskdialog_get_button_size(
            *(*dialog_info).command_links.offset(i as isize),
            dialog_width - x - h_spacing,
            core::ptr::addr_of_mut!(size),
        );
        size.cx = dialog_width - x - h_spacing;
        /* Add spacing */
        size.cy += 4;
        SetWindowPos(
            *(*dialog_info).command_links.offset(i as isize),
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        dialog_height = y + size.cy;
        i += 1;
    }

    dialog_height = max(dialog_height, main_icon_bottom);

    expando_right = 0;
    expando_bottom = dialog_height;
    /* Expando control */
    if !(*dialog_info).expando_button.is_null() {
        x = h_spacing;
        y = dialog_height + v_spacing;
        taskdialog_get_expando_size(
            dialog_info,
            (*dialog_info).expando_button,
            core::ptr::addr_of_mut!(size),
        );
        SetWindowPos(
            (*dialog_info).expando_button,
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        expando_right = x + size.cx;
        expando_bottom = y + size.cy;
    }

    /* Verification box */
    if !(*dialog_info).verification_box.is_null() {
        x = h_spacing;
        y = expando_bottom + v_spacing;
        size.cx = (DIALOG_MIN_WIDTH / 2) as i32;
        taskdialog_du_to_px(
            dialog_info,
            core::ptr::addr_of_mut!(size.cx),
            core::ptr::null_mut::<LONG>(),
        );
        taskdialog_get_button_size(
            (*dialog_info).verification_box,
            size.cx,
            core::ptr::addr_of_mut!(size),
        );
        SetWindowPos(
            (*dialog_info).verification_box,
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        expando_right = max(expando_right, x + size.cx);
        expando_bottom = y + size.cy;
    }

    /* Common and custom buttons */
    button_layout_infos = Alloc(
        ((*dialog_info).button_count as usize)
            .wrapping_mul(core::mem::size_of::<button_layout_info>()) as DWORD,
    ) as *mut button_layout_info;
    line_widths = Alloc(
        ((*dialog_info).button_count as usize).wrapping_mul(core::mem::size_of::<LONG>()) as DWORD,
    ) as *mut LONG;

    button_min_width = (DIALOG_BUTTON_WIDTH) as i32;
    button_height = (DIALOG_BUTTON_HEIGHT) as i32;
    taskdialog_du_to_px(
        dialog_info,
        core::ptr::addr_of_mut!(button_min_width),
        core::ptr::addr_of_mut!(button_height),
    );
    i = 0;
    while i < (*dialog_info).button_count {
        taskdialog_get_button_size(
            *(*dialog_info).buttons.offset(i as isize),
            dialog_width - expando_right - h_spacing * 2,
            core::ptr::addr_of_mut!(size),
        );
        (*button_layout_infos.offset(i as isize)).width = max(size.cx, button_min_width);
        i += 1;
    }

    /* Separate buttons into lines */
    x = expando_right + h_spacing;
    i = 0;
    line_count = 0;
    while i < (*dialog_info).button_count {
        (*button_layout_infos.offset(i as isize)).line = line_count;
        x += (*button_layout_infos.offset(i as isize)).width + h_spacing;
        *line_widths.offset(line_count as isize) +=
            (*button_layout_infos.offset(i as isize)).width + h_spacing;

        if (i + 1 < (*dialog_info).button_count)
            && (x
                + (*button_layout_infos.offset((i + 1) as isize)).width
                + h_spacing
                >= dialog_width)
        {
            x = expando_right + h_spacing;
            line_count += 1;
        }
        i += 1;
    }
    line_count += 1;

    /* Try to balance lines so they are about the same size */
    i = 1;
    while i < line_count - 1 {
        let diff_now: core::ffi::c_int = abs(
            *line_widths.offset(i as isize) - *line_widths.offset((i - 1) as isize),
        );
        let mut j: core::ffi::c_uint = 0;
        let mut last_button: core::ffi::c_uint = 0;
        let diff_changed: core::ffi::c_int;

        j = 0;
        while j < (*dialog_info).button_count as core::ffi::c_uint {
            if (*button_layout_infos.offset(j as isize)).line == i - 1 {
                last_button = j;
            }
            j = j.wrapping_add(1);
        }

        /* Difference in length of both lines if we wrapped the last button from the last line into this one */
        diff_changed = abs(
            2 * (*button_layout_infos.offset(last_button as isize)).width
                + *line_widths.offset(i as isize)
                - *line_widths.offset((i - 1) as isize),
        );

        if diff_changed < diff_now {
            (*button_layout_infos.offset(last_button as isize)).line = i;
            *line_widths.offset(i as isize) +=
                (*button_layout_infos.offset(last_button as isize)).width;
            *line_widths.offset((i - 1) as isize) -=
                (*button_layout_infos.offset(last_button as isize)).width;
        }
        i += 1;
    }

    /* Calculate left alignment so all lines are as far right as possible. */
    align = dialog_width - h_spacing;
    i = 0;
    while i < line_count {
        let new_alignment: LONG = dialog_width - *line_widths.offset(i as isize);
        if new_alignment < align {
            align = new_alignment;
        }
        i += 1;
    }

    /* Now that we got them all positioned, move all buttons */
    x = align;
    size.cy = button_height;
    i = 0;
    while i < (*dialog_info).button_count {
        /* New line */
        if i > 0
            && (*button_layout_infos.offset(i as isize)).line
                != (*button_layout_infos.offset((i - 1) as isize)).line
        {
            x = align;
            dialog_height += size.cy + v_spacing;
        }

        y = dialog_height + v_spacing;
        size.cx = (*button_layout_infos.offset(i as isize)).width;
        SetWindowPos(
            *(*dialog_info).buttons.offset(i as isize),
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        x += (*button_layout_infos.offset(i as isize)).width + h_spacing;
        i += 1;
    }

    /* Add height for last row button and spacing */
    dialog_height += size.cy + v_spacing;
    dialog_height = max(dialog_height, expando_bottom);

    Free(button_layout_infos as LPVOID);
    Free(line_widths as LPVOID);

    /* Footer icon */
    footer_icon_right = 0;
    footer_icon_bottom = dialog_height;
    if !(*dialog_info).footer_icon.is_null() {
        x = h_spacing;
        y = dialog_height + v_spacing;
        size.cx = GetSystemMetrics((SM_CXSMICON) as i32) as LONG;
        size.cy = GetSystemMetrics((SM_CYSMICON) as i32) as LONG;
        SetWindowPos(
            (*dialog_info).footer_icon,
            core::ptr::null_mut(),
            x,
            y,
            size.cx,
            size.cy,
            SWP_NOZORDER as UINT,
        );
        footer_icon_right = x + size.cx;
        footer_icon_bottom = y + size.cy;
    }

    /* Footer text */
    taskdialog_label_layout(
        dialog_info,
        (*dialog_info).footer_text,
        footer_icon_right,
        dialog_width,
        core::ptr::addr_of_mut!(dialog_height),
        syslink,
    );
    dialog_height = max(dialog_height, footer_icon_bottom);

    /* Expanded information */
    if ((*taskconfig).dwFlags & (TDF_EXPAND_FOOTER_AREA as i32)) != 0 && (*dialog_info).expanded != 0 {
        taskdialog_label_layout(
            dialog_info,
            (*dialog_info).expanded_info,
            0,
            dialog_width,
            core::ptr::addr_of_mut!(dialog_height),
            syslink,
        );
    }

    /* Add height for spacing, title height and frame height */
    dialog_height += v_spacing;
    dialog_height += GetSystemMetrics((SM_CYCAPTION) as i32) as LONG;
    dialog_height += GetSystemMetrics((SM_CXDLGFRAME) as i32) as LONG;

    if first_time != 0 {
        x = (ref_rect.left + ref_rect.right - dialog_width) / 2;
        y = (ref_rect.top + ref_rect.bottom - dialog_height) / 2;
        SetWindowPos(
            (*dialog_info).hwnd,
            core::ptr::null_mut(),
            x,
            y,
            dialog_width,
            dialog_height,
            SWP_NOZORDER as UINT,
        );
        first_time = FALSE;
    } else {
        SetWindowPos(
            (*dialog_info).hwnd,
            core::ptr::null_mut(),
            0,
            0,
            dialog_width,
            dialog_height,
            (SWP_NOMOVE | SWP_NOZORDER) as UINT,
        );
    }
}

unsafe extern "C" fn taskdialog_draw_expando_control(
    dialog_info: *mut taskdialog_info,
    dis: LPDRAWITEMSTRUCT,
) {
    let mut hwnd: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut text: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut icon_width: LONG = 0;
    let mut icon_height: LONG = 0;
    let mut text_offset: LONG = 0;
    let mut style: UINT = DFCS_FLAT as UINT;
    let mut draw_focus: BOOL = 0;

    hdc = (*dis).hDC;
    hwnd = (*dis).hwndItem;

    SendMessageW(hwnd, WM_ERASEBKGND as UINT, hdc as WPARAM, 0 as LPARAM);

    icon_width = DIALOG_EXPANDO_ICON_WIDTH as LONG;
    icon_height = DIALOG_EXPANDO_ICON_HEIGHT as LONG;
    taskdialog_du_to_px(dialog_info, &raw mut icon_width, &raw mut icon_height);
    rect.right = icon_width;
    rect.bottom = icon_height;
    style |= (if (*dialog_info).expanded != 0 {
        DFCS_SCROLLUP as UINT
    } else {
        DFCS_SCROLLDOWN as UINT
    }) as UINT;
    DrawFrameControl(hdc, &raw mut rect, DFC_SCROLL as UINT, style);

    GetCharWidthW(hdc, '0' as i32 as UINT, '0' as i32 as UINT, &raw mut text_offset);
    text_offset /= 2 as LONG;

    rect = (*dis).rcItem;
    rect.left += icon_width + text_offset;
    text = if (*dialog_info).expanded != 0 {
        (*dialog_info).expanded_text
    } else {
        (*dialog_info).collapsed_text
    };
    DrawTextW(
        hdc,
        text as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut rect,
        ((DT_WORDBREAK | DT_END_ELLIPSIS | DT_EXPANDTABS) as UINT),
    );

    draw_focus = (((*dis).itemState & (ODS_FOCUS as UINT)) != 0
        && ((*dis).itemState & (ODS_NOFOCUSRECT as UINT)) == 0) as ::core::ffi::c_int
        as BOOL;
    if draw_focus != 0 {
        DrawFocusRect(hdc, &raw mut rect as LPCRECT);
    }
}

unsafe extern "C" fn taskdialog_init(dialog_info: *mut taskdialog_info, hwnd: HWND) {
    let taskconfig: *const TASKDIALOGCONFIG = (*dialog_info).taskconfig;
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed::<NONCLIENTMETRICSW>();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut id: INT = 0;

    ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS as UINT,
        ncm.cbSize,
        core::ptr::addr_of_mut!(ncm) as PVOID,
        0 as UINT,
    );

    memset(
        dialog_info as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<taskdialog_info>() as size_t,
    );
    (*dialog_info).taskconfig = taskconfig;
    (*dialog_info).hwnd = hwnd;
    (*dialog_info).font = CreateFontIndirectW(core::ptr::addr_of_mut!(ncm.lfMessageFont));

    hdc = GetDC((*dialog_info).hwnd);
    SelectObject(hdc, (*dialog_info).font as HGDIOBJ);
    (*dialog_info).m.x_baseunit = GdiGetCharDimensions(
        hdc,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*dialog_info).m.y_baseunit),
    );
    ReleaseDC((*dialog_info).hwnd, hdc);

    (*dialog_info).m.h_spacing = DIALOG_SPACING as LONG;
    (*dialog_info).m.v_spacing = DIALOG_SPACING as LONG;
    taskdialog_du_to_px(
        dialog_info,
        core::ptr::addr_of_mut!((*dialog_info).m.h_spacing),
        core::ptr::addr_of_mut!((*dialog_info).m.v_spacing),
    );

    if ((*taskconfig).dwFlags & TDF_CALLBACK_TIMER as i32) != 0 {
        SetTimer(
            hwnd,
            ID_TIMER as UINT_PTR,
            DIALOG_TIMER_MS as UINT,
            None,
        );
        (*dialog_info).last_timer_tick = GetTickCount();
    }

    taskdialog_add_main_icon(dialog_info);
    taskdialog_add_main_instruction(dialog_info);
    taskdialog_add_content(dialog_info);
    taskdialog_add_expanded_info(dialog_info);
    taskdialog_add_progress_bar(dialog_info);
    taskdialog_add_radio_buttons(dialog_info);
    taskdialog_add_command_links(dialog_info);
    taskdialog_add_expando_button(dialog_info);
    taskdialog_add_verification_box(dialog_info);
    taskdialog_add_buttons(dialog_info);
    taskdialog_add_footer_icon(dialog_info);
    taskdialog_add_footer_text(dialog_info);

    /* Set default button */
    if (*dialog_info).default_button.is_null() && !(*dialog_info).command_links.is_null() {
        (*dialog_info).default_button =
            *(*dialog_info).command_links.offset(0 as isize);
    }
    if (*dialog_info).default_button.is_null() {
        (*dialog_info).default_button =
            *(*dialog_info).buttons.offset(0 as isize);
    }
    SendMessageW(
        (*dialog_info).hwnd,
        WM_NEXTDLGCTL as UINT,
        (*dialog_info).default_button as WPARAM,
        TRUE as LPARAM,
    );
    id = GetWindowLongW(
        (*dialog_info).default_button,
        GWLP_ID,
    ) as INT;
    SendMessageW(
        (*dialog_info).hwnd,
        DM_SETDEFID as UINT,
        id as WPARAM,
        0 as LPARAM,
    );

    (*dialog_info).has_cancel = ((((*taskconfig).dwFlags & TDF_ALLOW_DIALOG_CANCELLATION as i32) != 0)
        || !taskdialog_find_button(
            (*dialog_info).command_links,
            (*dialog_info).command_link_count,
            (IDCANCEL) as INT,
        )
        .is_null()
        || !taskdialog_find_button(
            (*dialog_info).buttons,
            (*dialog_info).button_count,
            (IDCANCEL) as INT,
        )
        .is_null()) as BOOL;

    if (*dialog_info).has_cancel == 0 {
        DeleteMenu(
            GetSystemMenu(hwnd, FALSE as BOOL),
            SC_CLOSE as UINT,
            MF_BYCOMMAND as UINT,
        );
    }

    taskdialog_layout(dialog_info);
}

unsafe extern "system" fn takdialog_destroy_control(hwnd: HWND, lParam: LPARAM) -> BOOL {
    DestroyWindow(hwnd);
    return TRUE;
}

unsafe extern "C" fn taskdialog_destroy(mut dialog_info: *mut taskdialog_info) {
    EnumChildWindows(
        (*dialog_info).hwnd,
        Some(takdialog_destroy_control),
        0 as LPARAM,
    );

    if (((*(*dialog_info).taskconfig).dwFlags as DWORD) & TDF_CALLBACK_TIMER as DWORD) != 0 {
        KillTimer((*dialog_info).hwnd, ID_TIMER as UINT_PTR);
    }
    if !(*dialog_info).font.is_null() {
        DeleteObject((*dialog_info).font as HGDIOBJ);
    }
    if !(*dialog_info).main_instruction_font.is_null() {
        DeleteObject((*dialog_info).main_instruction_font as HGDIOBJ);
    }
    Free((*dialog_info).buttons as LPVOID);
    Free((*dialog_info).radio_buttons as LPVOID);
    Free((*dialog_info).command_links as LPVOID);
    Free((*dialog_info).expanded_text as LPVOID);
    Free((*dialog_info).collapsed_text as LPVOID);
}
unsafe extern "system" fn taskdialog_proc(
    mut hwnd: HWND,
    mut msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    static mut taskdialog_info_propnameW: [WCHAR; 15] = [
    0x0054u16, 0x0061, 0x0073, 0x006B, 0x0044, 0x0069, 0x0061, 0x006C, 0x006F, 0x0067,
    0x0049, 0x006E, 0x0066, 0x006F, 0x0000,
];
    let mut dialog_info: *mut taskdialog_info = ::core::ptr::null_mut::<taskdialog_info>();
    let mut result: LRESULT = 0;
    if msg != (WM_INITDIALOG as UINT) {
        dialog_info = GetPropW(
            hwnd,
            ::core::ptr::addr_of!(taskdialog_info_propnameW) as LPCWSTR,
        ) as *mut taskdialog_info;
    }
    match msg {
        TDM_NAVIGATE_PAGE => {
            (*dialog_info).taskconfig = lParam as *const TASKDIALOGCONFIG;
            taskdialog_destroy(dialog_info);
            taskdialog_init(dialog_info, hwnd);
            taskdialog_notify(
                dialog_info,
                TDN_DIALOG_CONSTRUCTED as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            taskdialog_check_default_radio_buttons(dialog_info);
            taskdialog_notify(
                dialog_info,
                TDN_NAVIGATED as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }
        TDM_CLICK_BUTTON => {
            taskdialog_click_button(dialog_info, wParam as INT);
        }
        TDM_ENABLE_BUTTON => {
            taskdialog_enable_button(dialog_info, wParam as INT, lParam as BOOL);
        }
        TDM_SET_MARQUEE_PROGRESS_BAR => {
            let mut marquee: BOOL = wParam as BOOL;
            let mut style: LONG = 0;
            if !(*dialog_info).progress_bar.is_null() {
                style = GetWindowLongW(
                    (*dialog_info).progress_bar,
                    GWL_STYLE as INT,
                ) as LONG;
                style = if marquee != 0 {
                    style | (PBS_MARQUEE as LONG)
                } else {
                    style & !(PBS_MARQUEE as LONG)
                };
                SetWindowLongW(
                    (*dialog_info).progress_bar,
                    GWL_STYLE as INT,
                    style,
                );
            }
        }
        TDM_SET_PROGRESS_BAR_STATE => {
            result = SendMessageW(
                (*dialog_info).progress_bar,
                PBM_SETSTATE as UINT,
                wParam,
                0 as LPARAM,
            );
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT as INT, result as LONG_PTR);
        }
        TDM_SET_PROGRESS_BAR_RANGE => {
            result = SendMessageW(
                (*dialog_info).progress_bar,
                PBM_SETRANGE as UINT,
                0 as WPARAM,
                lParam,
            );
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT as INT, result as LONG_PTR);
        }
        TDM_SET_PROGRESS_BAR_POS => {
            result = 0 as LRESULT;
            if !(*dialog_info).progress_bar.is_null() {
                let mut style_0: LONG = GetWindowLongW(
                    (*dialog_info).progress_bar,
                    GWL_STYLE as INT,
                ) as LONG;
                if (style_0 & (PBS_MARQUEE as LONG)) == 0 {
                    result = SendMessageW(
                        (*dialog_info).progress_bar,
                        PBM_SETPOS as UINT,
                        wParam,
                        0 as LPARAM,
                    );
                }
            }
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT as INT, result as LONG_PTR);
        }
        TDM_SET_PROGRESS_BAR_MARQUEE => {
            SendMessageW(
                (*dialog_info).progress_bar,
                PBM_SETMARQUEE as UINT,
                wParam,
                lParam,
            );
        }
        TDM_SET_ELEMENT_TEXT => {
            taskdialog_set_element_text(dialog_info, (wParam as INT) as TASKDIALOG_ELEMENTS, lParam as *const WCHAR);
            taskdialog_layout(dialog_info);
        }
        TDM_UPDATE_ELEMENT_TEXT => {
            taskdialog_set_element_text(dialog_info, (wParam as INT) as TASKDIALOG_ELEMENTS, lParam as *const WCHAR);
        }
        TDM_CLICK_RADIO_BUTTON => {
            taskdialog_click_radio_button(dialog_info, wParam as INT);
        }
        TDM_ENABLE_RADIO_BUTTON => {
            taskdialog_enable_radio_button(dialog_info, wParam as INT, lParam as BOOL);
        }
        TDM_CLICK_VERIFICATION => {
            let mut checked: BOOL = wParam as BOOL;
            let mut focused: BOOL = lParam as BOOL;
            (*dialog_info).verification_checked = checked;
            if !(*dialog_info).verification_box.is_null() {
                SendMessageW(
                    (*dialog_info).verification_box,
                    BM_SETCHECK as UINT,
                    (if checked != 0 {
                        BST_CHECKED
                    } else {
                        BST_UNCHECKED
                    }) as WPARAM,
                    0 as LPARAM,
                );
                taskdialog_notify(
                    dialog_info,
                    TDN_VERIFICATION_CLICKED as UINT,
                    checked as WPARAM,
                    0 as LPARAM,
                );
                if focused != 0 {
                    SetFocus((*dialog_info).verification_box);
                }
            }
        }
        TDM_SET_BUTTON_ELEVATION_REQUIRED_STATE => {
            taskdialog_button_set_shield(dialog_info, wParam as INT, lParam as BOOL);
        }
        TDM_UPDATE_ICON => {
            taskdialog_set_icon(dialog_info, wParam as INT, lParam as HICON);
        }
        WM_INITDIALOG => {
            dialog_info = lParam as *mut taskdialog_info;

            taskdialog_init(dialog_info, hwnd);

            SetPropW(
                hwnd,
                ::core::ptr::addr_of!(taskdialog_info_propnameW) as LPCWSTR,
                dialog_info as HANDLE,
            );
            taskdialog_notify(
                dialog_info,
                TDN_DIALOG_CONSTRUCTED as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            taskdialog_notify(
                dialog_info,
                TDN_CREATED as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            taskdialog_check_default_radio_buttons(dialog_info);
            return FALSE as INT_PTR;
        }
        WM_COMMAND => {
            if (HIWORD!(wParam) == (BN_CLICKED as WORD)) {
                taskdialog_on_button_click(dialog_info, lParam as HWND, LOWORD!(wParam));
            } else {
                return FALSE as INT_PTR;
            }
        }
        WM_HELP => {
            taskdialog_notify(
                dialog_info,
                TDN_HELP as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }
        WM_TIMER => {
            if (ID_TIMER as WPARAM) == wParam {
                let mut elapsed: DWORD =
                    GetTickCount().wrapping_sub((*dialog_info).last_timer_tick);
                if taskdialog_notify(
                    dialog_info,
                    TDN_TIMER as UINT,
                    elapsed as WPARAM,
                    0 as LPARAM,
                ) == (S_FALSE as HRESULT)
                {
                    (*dialog_info).last_timer_tick = GetTickCount();
                }
            }
        }
        WM_NOTIFY => {
            let mut pnmLink: PNMLINK = lParam as PNMLINK;
            let mut hwndFrom: HWND = (*pnmLink).hdr.hwndFrom;
            if taskdialog_hyperlink_enabled(dialog_info) != 0
                && (hwndFrom == (*dialog_info).content
                    || hwndFrom == (*dialog_info).expanded_info
                    || hwndFrom == (*dialog_info).footer_text)
                && ((*pnmLink).hdr.code == (NM_CLICK as UINT)
                    || (*pnmLink).hdr.code == (NM_RETURN as UINT))
            {
                taskdialog_notify(
                    dialog_info,
                    TDN_HYPERLINK_CLICKED as UINT,
                    0 as WPARAM,
                    &raw mut (*pnmLink).item.szUrl as *mut WCHAR as LPARAM,
                );
            } else {
                return FALSE as INT_PTR;
            }
        }
        WM_DRAWITEM => {
            let mut dis: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;
            if (*dis).hwndItem == (*dialog_info).expando_button {
                taskdialog_draw_expando_control(dialog_info, dis);
                SetWindowLongPtrW(
                    hwnd,
                    DWLP_MSGRESULT as INT,
                    TRUE as LONG_PTR,
                );
            } else {
                return FALSE as INT_PTR;
            }
        }
        WM_DESTROY => {
            taskdialog_notify(
                dialog_info,
                TDN_DESTROYED as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
            RemovePropW(
                hwnd,
                ::core::ptr::addr_of!(taskdialog_info_propnameW) as LPCWSTR,
            );
            taskdialog_destroy(dialog_info);
        }
        WM_CLOSE => {
            if (*dialog_info).has_cancel != 0 {
                if taskdialog_notify(
                    dialog_info,
                    TDN_BUTTON_CLICKED as UINT,
                    IDCANCEL as WPARAM,
                    0 as LPARAM,
                ) == (S_OK as HRESULT)
                {
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                }
                SetWindowLongPtrW(
                    hwnd,
                    DWLP_MSGRESULT as INT,
                    0 as LONG_PTR,
                );
            } else {
                return FALSE as INT_PTR;
            }
        }
        _ => return FALSE as INT_PTR,
    }
    return TRUE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn TaskDialogIndirect(taskconfig: *const TASKDIALOGCONFIG, button: *mut INT, radio_button: *mut INT, verification_flag_checked: *mut BOOL) -> HRESULT {
    let mut dialog_info: taskdialog_info = core::mem::zeroed();
    let mut template: *mut DLGTEMPLATE;
    let mut ret: INT;

    if taskconfig.is_null() || (*taskconfig).cbSize as usize != core::mem::size_of::<TASKDIALOGCONFIG>() as usize {
        return E_INVALIDARG;
    }

    dialog_info.taskconfig = taskconfig;

    template = create_taskdialog_template(taskconfig);
    ret = DialogBoxIndirectParamW(
        (*taskconfig).hInstance,
        template as LPCDLGTEMPLATE,
        (*taskconfig).hwndParent,
        Some(taskdialog_proc),
        &raw mut dialog_info as LPARAM,
    ) as i16 as INT;
    Free(template as LPVOID);

    if !button.is_null() {
        *button = ret;
    }
    if !radio_button.is_null() {
        *radio_button = dialog_info.selected_radio_id;
    }
    if !verification_flag_checked.is_null() {
        *verification_flag_checked = dialog_info.verification_checked;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn TaskDialog(
    mut owner: HWND,
    mut hinst: HINSTANCE,
    mut title: *const WCHAR,
    mut main_instruction: *const WCHAR,
    mut content: *const WCHAR,
    mut common_buttons: TASKDIALOG_COMMON_BUTTON_FLAGS,
    mut icon: *const WCHAR,
    mut button: *mut ::core::ffi::c_int,
) -> HRESULT {
    let mut taskconfig: TASKDIALOGCONFIG = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/taskdialog.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"TaskDialog\0".as_ptr() as *const ::core::ffi::c_char,
            1408 as ::core::ffi::c_int,
            b"%p, %p, %s, %s, %s, %#x, %s, %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            owner,
            hinst,
            debugstr_w(title),
            debugstr_w(main_instruction),
            debugstr_w(content),
            common_buttons,
            debugstr_w(icon),
            button,
        );
    }
    memset(
        core::ptr::addr_of_mut!(taskconfig) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<TASKDIALOGCONFIG>() as size_t,
    );
    taskconfig.cbSize = ::core::mem::size_of::<TASKDIALOGCONFIG>() as UINT;
    taskconfig.hwndParent = owner;
    taskconfig.hInstance = hinst;
    taskconfig.dwCommonButtons = common_buttons;
    taskconfig.pszWindowTitle = title as PCWSTR;
    taskconfig.u.pszMainIcon = icon as PCWSTR;
    taskconfig.pszMainInstruction = main_instruction as PCWSTR;
    taskconfig.pszContent = content as PCWSTR;
    return TaskDialogIndirect(
        core::ptr::addr_of_mut!(taskconfig),
        button,
        core::ptr::null_mut::<::core::ffi::c_int>(),
        core::ptr::null_mut::<BOOL>(),
    );
}
