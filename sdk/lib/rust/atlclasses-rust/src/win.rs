//! ATL's window machinery -- `CWindowImpl` / `CDialogImpl` -- without the machine-code thunk.
//!
//! ATL routes a window's messages to its C++ object through a per-instance THUNK: generated
//! instruction bytes that swap the `HWND` argument for `this`. Reproducing that means a code
//! generator per architecture (i386, amd64 AND arm64), on a tree whose point is that arm64 is
//! first-class. shell32-rust's `atlwin.rs` established the alternative this module shares:
//! find `this` from the `HWND` instead. Here it is a window PROPERTY rather than
//! `GWLP_USERDATA`, so the same mechanism serves a window we create, a dialog, and a control we
//! subclass (whose `GWLP_USERDATA` belongs to the control).
//!
//! The ONE behavioural difference, stated because it is not free: ATL picks `this` up on the
//! first message the window receives, whatever it is; we pick it up from `lpCreateParams` at
//! `WM_NCCREATE` (`WM_INITDIALOG`'s `lParam` for a dialog). A message delivered before that
//! (`WM_GETMINMAXINFO` to an overlapped window) has no object yet and goes to the default
//! procedure. A window class that must handle such a message cannot use this path silently.
//!
//! Types are the primitive basis (see the crate docs). The per-class glue `cpp_app` generates
//! casts its island's `HWND`/`WPARAM`/`LPARAM` to these, which is ABI-identical: `WPARAM` is
//! `c_uint` on i386 and `c_ulonglong` on 64-bit, exactly `usize`'s width on each.

use core::ffi::c_void;

pub type HWND = *mut c_void;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
type DlgProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize;

const WM_NCCREATE: u32 = 0x0081;
const WM_NCDESTROY: u32 = 0x0082;
const WM_INITDIALOG: u32 = 0x0110;
const GWLP_WNDPROC: i32 = -4;
const DWLP_MSGRESULT: i32 = 0;

#[repr(C)]
struct CreateStructW {
    lp_create_params: *mut c_void,
    // the rest is never read here
}

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<WndProc>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: *mut c_void,
}

extern "system" {
    fn RegisterClassExW(wc: *const WndClassExW) -> u16;
    fn GetClassInfoExW(h: *mut c_void, name: *const u16, wc: *mut WndClassExW) -> i32;
    fn CreateWindowExW(
        ex_style: u32,
        class: *const u16,
        name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: *mut c_void,
        inst: *mut c_void,
        param: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    fn CallWindowProcW(p: WndProc, h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    fn SetPropW(h: HWND, name: *const u16, data: *mut c_void) -> i32;
    fn GetPropW(h: HWND, name: *const u16) -> *mut c_void;
    fn RemovePropW(h: HWND, name: *const u16) -> *mut c_void;
    fn DialogBoxParamW(
        inst: *mut c_void,
        template: *const u16,
        parent: HWND,
        proc_: DlgProc,
        param: LPARAM,
    ) -> isize;
    fn CreateDialogParamW(
        inst: *mut c_void,
        template: *const u16,
        parent: HWND,
        proc_: DlgProc,
        param: LPARAM,
    ) -> HWND;
    fn LoadCursorW(inst: *mut c_void, name: *const u16) -> *mut c_void;
}

// SetWindowLongPtrW is a MACRO over SetWindowLongW on i386: there is no such export.
#[cfg(target_arch = "x86")]
extern "system" {
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtrW(h: HWND, index: i32, value: isize) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn SetWindowLongPtrW(h: HWND, index: i32, value: isize) -> isize;
}

/// L"AtlRustThis" / L"AtlRustSuper": the object, and a subclassed window's previous proc.
static PROP_THIS: [u16; 12] = utf16("AtlRustThis\0");
static PROP_SUPER: [u16; 13] = utf16("AtlRustSuper\0");

const fn utf16<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < N {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

/// What `BEGIN_MSG_MAP` generates: ATL's `ProcessWindowMessage`. `cpp_app` implements it per
/// class from the map, in the map's order (ATL dispatches the FIRST matching entry).
pub trait MessageMap {
    /// TRUE when an entry handled the message; `*result` is then the window's answer.
    unsafe fn process_window_message(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        result: &mut LRESULT,
        map_id: u32,
    ) -> bool;
    /// The object's `m_hWnd` (its `CWindow` base), which this module sets and clears.
    unsafe fn hwnd_slot(&mut self) -> *mut HWND;
    /// ATL's `OnFinalMessage`, after `WM_NCDESTROY`. A no-op unless the class defines it.
    unsafe fn on_final_message(&mut self, _hwnd: HWND) {}
}

/// `DECLARE_WND_CLASS_EX(name, style, bkgnd)` -- the class a `CWindowImpl` registers.
pub struct WndClassInfo {
    pub class_name: *const u16,
    pub style: u32,
    /// `bkgnd` is a COLOR_* index; ATL registers `(HBRUSH)(bkgnd + 1)`.
    pub background_color: i32,
}

unsafe impl Sync for WndClassInfo {}

unsafe fn object<T>(hwnd: HWND) -> *mut T {
    GetPropW(hwnd, PROP_THIS.as_ptr()) as *mut T
}

unsafe fn route<T: MessageMap>(
    this: *mut T,
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    let mut result: LRESULT = 0;
    if (*this).process_window_message(hwnd, msg, w, l, &mut result, 0) {
        return result;
    }
    def_window_proc(hwnd, msg, w, l)
}

unsafe extern "system" fn window_proc<T: MessageMap>(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    let mut this = object::<T>(hwnd);
    if this.is_null() && msg == WM_NCCREATE {
        this = (*(l as *const CreateStructW)).lp_create_params as *mut T;
        if !this.is_null() {
            SetPropW(hwnd, PROP_THIS.as_ptr(), this as *mut c_void);
            *(*this).hwnd_slot() = hwnd;
        }
    }
    if this.is_null() {
        return def_window_proc(hwnd, msg, w, l);
    }
    let r = route(this, hwnd, msg, w, l);
    if msg == WM_NCDESTROY {
        RemovePropW(hwnd, PROP_THIS.as_ptr());
        RemovePropW(hwnd, PROP_SUPER.as_ptr());
        *(*this).hwnd_slot() = core::ptr::null_mut();
        (*this).on_final_message(hwnd);
    }
    r
}

/// `CWindowImpl::DefWindowProc`: the subclassed window's previous procedure, else the default.
pub unsafe fn def_window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let sup = GetPropW(hwnd, PROP_SUPER.as_ptr());
    if sup.is_null() {
        DefWindowProcW(hwnd, msg, w, l)
    } else {
        CallWindowProcW(
            core::mem::transmute::<*mut c_void, WndProc>(sup),
            hwnd,
            msg,
            w,
            l,
        )
    }
}

/// `CWindowImpl::Create`: register the class on first use, then create with `this` as the
/// create parameter. `rect` is `*const RECT` (null: CW_USEDEFAULT), as ATL's `_U_RECT`.
#[allow(clippy::too_many_arguments)]
pub unsafe fn create_window<T: MessageMap>(
    this: *mut T,
    info: &WndClassInfo,
    parent: HWND,
    rect: *const i32,
    name: *const u16,
    style: u32,
    ex_style: u32,
    menu_or_id: *mut c_void,
) -> HWND {
    let inst = crate::resource_instance();
    let mut wc: WndClassExW = core::mem::zeroed();
    wc.cb_size = core::mem::size_of::<WndClassExW>() as u32;
    if GetClassInfoExW(inst, info.class_name, &mut wc) == 0 {
        wc = core::mem::zeroed();
        wc.cb_size = core::mem::size_of::<WndClassExW>() as u32;
        wc.style = info.style;
        wc.lpfn_wnd_proc = Some(window_proc::<T>);
        wc.h_instance = inst;
        wc.h_cursor = LoadCursorW(core::ptr::null_mut(), 32512 as *const u16); // IDC_ARROW
        wc.hbr_background = (info.background_color as isize + 1) as *mut c_void;
        wc.lpsz_class_name = info.class_name;
        if RegisterClassExW(&wc) == 0 {
            return core::ptr::null_mut();
        }
    }
    const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
    let (x, y, w, h) = if rect.is_null() {
        (CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT)
    } else {
        let r = core::slice::from_raw_parts(rect, 4);
        (r[0], r[1], r[2] - r[0], r[3] - r[1])
    };
    CreateWindowExW(
        ex_style,
        info.class_name,
        name,
        style,
        x,
        y,
        w,
        h,
        parent,
        menu_or_id,
        inst,
        this as *mut c_void,
    )
}

/// `CWindowImpl::SubclassWindow`: route an existing window (a control) through `this`.
pub unsafe fn subclass_window<T: MessageMap>(this: *mut T, hwnd: HWND) -> bool {
    if hwnd.is_null() || !object::<T>(hwnd).is_null() {
        return false;
    }
    SetPropW(hwnd, PROP_THIS.as_ptr(), this as *mut c_void);
    let old = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, window_proc::<T> as usize as isize);
    SetPropW(hwnd, PROP_SUPER.as_ptr(), old as *mut c_void);
    *(*this).hwnd_slot() = hwnd;
    true
}

unsafe extern "system" fn dialog_proc<T: MessageMap>(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> isize {
    // A dialog's object rides WM_INITDIALOG's lParam (DialogBoxParam's dwInitParam).
    dialog_message::<T>(hwnd, msg, w, l, l as *mut T)
}

unsafe extern "system" fn page_dialog_proc<T: PropertyPage>(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> isize {
    // A property page's WM_INITDIALOG lParam is comctl32's COPY of the PROPSHEETPAGE; the
    // object is that copy's lParam, which `create_property_page` set.
    let init = if msg == WM_INITDIALOG && l != 0 {
        (*(l as *const PropSheetPageV1)).l_param as *mut T
    } else {
        core::ptr::null_mut()
    };
    dialog_message::<T>(hwnd, msg, w, l, init)
}

unsafe fn dialog_message<T: MessageMap>(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    init: *mut T,
) -> isize {
    let mut this = object::<T>(hwnd);
    if this.is_null() && msg == WM_INITDIALOG && !init.is_null() {
        this = init;
        SetPropW(hwnd, PROP_THIS.as_ptr(), this as *mut c_void);
        *(*this).hwnd_slot() = hwnd;
    }
    if this.is_null() {
        return 0;
    }
    let mut result: LRESULT = 0;
    let handled = (*this).process_window_message(hwnd, msg, w, l, &mut result, 0);
    // ReactOS ATL's CDialogImplBaseT::DialogProc: a handled message's result goes to
    // DWLP_MSGRESULT AND is returned. user32's DefDlgProc returns the procedure's own value only
    // for WM_COMPAREITEM/VKEYTOITEM/CHARTOITEM/QUERYDRAGICON/INITDIALOG and reads
    // DWLP_MSGRESULT for everything else -- WM_NOTIFY included, so a PSN_APPLY handler's
    // PSNRET_INVALID is lost unless it is stored there.
    if handled && msg != WM_NCDESTROY {
        SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, result);
    }
    if msg == WM_NCDESTROY {
        RemovePropW(hwnd, PROP_THIS.as_ptr());
        *(*this).hwnd_slot() = core::ptr::null_mut();
        (*this).on_final_message(hwnd);
    }
    result
}

/// `CDialogImpl::DoModal`.
pub unsafe fn do_modal<T: MessageMap>(
    this: *mut T,
    idd: u16,
    parent: HWND,
    _init: LPARAM,
) -> isize {
    DialogBoxParamW(
        crate::resource_instance(),
        idd as usize as *const u16,
        parent,
        dialog_proc::<T>,
        this as LPARAM,
    )
}

/// `CDialogImpl::Create` (modeless).
pub unsafe fn create_dialog<T: MessageMap>(this: *mut T, idd: u16, parent: HWND) -> HWND {
    CreateDialogParamW(
        crate::resource_instance(),
        idd as usize as *const u16,
        parent,
        dialog_proc::<T>,
        this as LPARAM,
    )
}

/// `PROPSHEETPAGEW` through `pcRefParent` -- `PROPSHEETPAGEW_V1_SIZE`, which every comctl32
/// accepts. The C++ passes `sizeof(PROPSHEETPAGE)` of whatever `_WIN32_IE` it compiled
/// against; none of the later fields (header titles, activation context, header bitmap) are
/// set by `CPropertyPageImpl`, so the V1 size says exactly what the C says.
#[repr(C)]
struct PropSheetPageV1 {
    dw_size: u32,
    dw_flags: u32,
    h_instance: *mut c_void,
    psz_template: *const u16,
    h_icon: *mut c_void,
    psz_title: *const u16,
    pfn_dlg_proc: Option<DlgProc>,
    l_param: LPARAM,
    pfn_callback: Option<PageCallback>,
    pc_ref_parent: *mut u32,
}

type PageCallback = unsafe extern "system" fn(HWND, u32, *mut PropSheetPageV1) -> u32;

const PSP_USETITLE: u32 = 0x0008;
const PSP_USECALLBACK: u32 = 0x0080;
const PSPCB_ADDREF: u32 = 0;
const PSPCB_RELEASE: u32 = 1;
const PSPCB_CREATE: u32 = 2;

#[repr(C)]
struct NmHdr {
    hwnd_from: HWND,
    id_from: usize,
    code: u32,
}

extern "system" {
    fn CreatePropertySheetPageW(page: *const PropSheetPageV1) -> *mut c_void;
    fn GetParent(h: HWND) -> HWND;
}

/// ReactOS `CPropertyPageImpl<T>` (rosdlgs.h): the overridables its callback and its
/// `WM_NOTIFY` handler dispatch to. Each default is the C++ base's; `cpp_app` overrides the
/// ones the module's class declares.
#[allow(unused_variables)]
pub trait PropertyPage: MessageMap {
    unsafe fn on_page_create(&mut self) -> i32 {
        1
    }
    unsafe fn on_page_add_ref(&mut self) {}
    unsafe fn on_page_release(&mut self) {}
    unsafe fn on_set_active(&mut self) -> i32 {
        0
    }
    unsafe fn on_kill_active(&mut self) -> i32 {
        0
    }
    unsafe fn on_apply(&mut self) -> i32 {
        0 // PSNRET_NOERROR
    }
    unsafe fn on_reset(&mut self) {}
    unsafe fn on_help(&mut self) {}
    unsafe fn on_wizard_back(&mut self) -> i32 {
        0
    }
    unsafe fn on_wizard_next(&mut self) -> i32 {
        0
    }
    unsafe fn on_wizard_finish(&mut self) -> i32 {
        0
    }
    unsafe fn on_query_cancel(&mut self) -> i32 {
        0
    }
    unsafe fn on_get_object(&mut self, notify: LPARAM) -> i32 {
        0
    }
    unsafe fn on_translate_accelerator(&mut self, msg: LPARAM) -> i32 {
        0 // PSNRET_NOERROR
    }
    unsafe fn on_query_initial_focus(&mut self, focus: HWND) -> HWND {
        core::ptr::null_mut()
    }
}

/// `CPropertyPageImpl::Create`: the page's `HPROPSHEETPAGE`. comctl32 copies the struct, so
/// it need not outlive the call; the object must outlive the sheet, as in the C++.
pub unsafe fn create_property_page<T: PropertyPage>(
    this: *mut T,
    idd: u16,
    title: *const u16,
) -> *mut c_void {
    let page = PropSheetPageV1 {
        dw_size: core::mem::size_of::<PropSheetPageV1>() as u32,
        dw_flags: PSP_USECALLBACK | if title.is_null() { 0 } else { PSP_USETITLE },
        h_instance: crate::resource_instance(),
        psz_template: idd as usize as *const u16,
        h_icon: core::ptr::null_mut(),
        psz_title: title,
        pfn_dlg_proc: Some(page_dialog_proc::<T>),
        l_param: this as LPARAM,
        pfn_callback: Some(page_callback::<T>),
        pc_ref_parent: core::ptr::null_mut(),
    };
    CreatePropertySheetPageW(&page)
}

unsafe extern "system" fn page_callback<T: PropertyPage>(
    _hwnd: HWND,
    msg: u32,
    page: *mut PropSheetPageV1,
) -> u32 {
    let this = (*page).l_param as *mut T;
    match msg {
        PSPCB_CREATE => return (*this).on_page_create() as u32,
        PSPCB_ADDREF => (*this).on_page_add_ref(),
        PSPCB_RELEASE => (*this).on_page_release(),
        _ => {}
    }
    0
}

/// `CPropertyPageImpl`'s own message map -- `MESSAGE_HANDLER(WM_NOTIFY, OnNotify)` -- which a
/// page reaches by `CHAIN_MSG_MAP(CPropertyPageImpl<T>)`. TRUE when handled.
pub unsafe fn property_page_map<T: PropertyPage>(
    this: *mut T,
    hwnd: HWND,
    msg: u32,
    l: LPARAM,
    result: &mut LRESULT,
) -> bool {
    const WM_NOTIFY: u32 = 0x004E;
    if msg != WM_NOTIFY {
        return false;
    }
    let nm = &*(l as *const NmHdr);
    if nm.hwnd_from != GetParent(hwnd) && nm.hwnd_from != hwnd {
        *result = 1; // the C++ returns TRUE with bHandled = FALSE
        return false;
    }
    let t = &mut *this;
    *result = match nm.code as i32 {
        -200 => t.on_set_active() as LRESULT,  // PSN_SETACTIVE
        -201 => t.on_kill_active() as LRESULT, // PSN_KILLACTIVE
        -202 => t.on_apply() as LRESULT,       // PSN_APPLY
        -203 => {
            t.on_reset(); // PSN_RESET
            0
        }
        -205 => {
            t.on_help(); // PSN_HELP
            0
        }
        -206 => t.on_wizard_back() as LRESULT,   // PSN_WIZBACK
        -207 => t.on_wizard_next() as LRESULT,   // PSN_WIZNEXT
        -208 => t.on_wizard_finish() as LRESULT, // PSN_WIZFINISH
        -209 => t.on_query_cancel() as LRESULT,  // PSN_QUERYCANCEL
        -210 => t.on_get_object(l) as LRESULT,   // PSN_GETOBJECT
        -212 => t.on_translate_accelerator(l) as LRESULT, // PSN_TRANSLATEACCELERATOR
        -213 => t.on_query_initial_focus(l as HWND) as LRESULT, // PSN_QUERYINITIALFOCUS
        _ => {
            *result = 0;
            return false;
        }
    };
    true
}

/// `CPropertyPageImpl::SetModified`.
pub unsafe fn property_page_set_modified(hwnd: HWND, changed: bool) {
    extern "system" {
        fn SendMessageW(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    }
    // PSM_CHANGED / PSM_UNCHANGED
    SendMessageW(
        GetParent(hwnd),
        if changed { 0x0468 } else { 0x046D },
        hwnd as WPARAM,
        0,
    );
}
