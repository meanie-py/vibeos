// Deterministic from shobjidl.h
#[repr(C)]
pub struct IUserNotification {
    pub lpVtbl: *const IUserNotificationVtbl,
}
#[repr(C)]
pub struct IUserNotificationVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUserNotification, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUserNotification) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUserNotification) -> ULONG,
    pub SetBalloonInfo: unsafe extern "system" fn(This: *mut IUserNotification, pszTitle: LPCWSTR, pszText: LPCWSTR, dwInfoFlags: DWORD) -> HRESULT,
    pub SetBalloonRetry: unsafe extern "system" fn(This: *mut IUserNotification, dwShowTime: DWORD, dwInterval: DWORD, cRetryCount: UINT) -> HRESULT,
    pub SetIconInfo: unsafe extern "system" fn(This: *mut IUserNotification, hIcon: HICON, pszToolTip: LPCWSTR) -> HRESULT,
    pub Show: unsafe extern "system" fn(This: *mut IUserNotification, pqc: *mut IQueryContinue, dwContinuePollInterval: DWORD) -> HRESULT,
    pub PlaySound: unsafe extern "system" fn(This: *mut IUserNotification, pszSoundName: LPCWSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IUserNotification: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IQueryContinue {
    pub lpVtbl: *const IQueryContinueVtbl,
}
#[repr(C)]
pub struct IQueryContinueVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IQueryContinue, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IQueryContinue) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IQueryContinue) -> ULONG,
    pub QueryContinue: unsafe extern "system" fn(This: *mut IQueryContinue) -> HRESULT,
}
extern "C" {
    pub static IID_IQueryContinue: IID;
}

// Deterministic COM scaffold for CUserNotification; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CUserNotification {
    pub IUserNotification_iface: IUserNotification,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWorkerWnd: HWND,
    pub m_hIcon: HICON,
    pub m_dwInfoFlags: DWORD,
    pub m_uShowTime: UINT,
    pub m_uInterval: UINT,
    pub m_cRetryCount: UINT,
    pub m_uContinuePoolInterval: UINT,
    pub m_bIsShown: BOOL,
    pub m_hRes: HRESULT,
    pub m_pqc: *mut IQueryContinue,
    pub m_szTip: CStringW,
    pub m_szInfo: CStringW,
    pub m_szInfoTitle: CStringW,
}

unsafe fn CUserNotification_cpp_com_assign_m_szTip(this: *mut CUserNotification, value: LPCWSTR) {
    (*this).m_szTip.assign(value);
}

unsafe fn CUserNotification_cpp_com_assign_m_szInfo(this: *mut CUserNotification, value: LPCWSTR) {
    (*this).m_szInfo.assign(value);
}

unsafe fn CUserNotification_cpp_com_assign_m_szInfoTitle(this: *mut CUserNotification, value: LPCWSTR) {
    (*this).m_szInfoTitle.assign(value);
}

unsafe extern "system" fn CUserNotification_IUserNotification_QueryInterface(This: *mut IUserNotification, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CUserNotification_IUserNotification_AddRef(This: *mut IUserNotification) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_AddRef(this)
}
unsafe extern "system" fn CUserNotification_IUserNotification_Release(This: *mut IUserNotification) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_Release(this)
}
unsafe extern "system" fn CUserNotification_IUserNotification_SetBalloonInfo(This: *mut IUserNotification, pszTitle: LPCWSTR, pszText: LPCWSTR, dwInfoFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_SetBalloonInfo_body(this, pszTitle, pszText, dwInfoFlags)
}
unsafe extern "system" fn CUserNotification_IUserNotification_SetBalloonRetry(This: *mut IUserNotification, dwShowTime: DWORD, dwInterval: DWORD, cRetryCount: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_SetBalloonRetry_body(this, dwShowTime, dwInterval, cRetryCount)
}
unsafe extern "system" fn CUserNotification_IUserNotification_SetIconInfo(This: *mut IUserNotification, hIcon: HICON, pszToolTip: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_SetIconInfo_body(this, hIcon, pszToolTip)
}
unsafe extern "system" fn CUserNotification_IUserNotification_Show(This: *mut IUserNotification, pqc: *mut IQueryContinue, dwContinuePollInterval: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_Show_body(this, pqc, dwContinuePollInterval)
}
unsafe extern "system" fn CUserNotification_IUserNotification_PlaySound(This: *mut IUserNotification, pszSoundName: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CUserNotification, IUserNotification_iface)) as *mut CUserNotification;
    CUserNotification_PlaySound_body(this, pszSoundName)
}

pub static CUSERNOTIFICATION_IUSERNOTIFICATION_VTBL: IUserNotificationVtbl = IUserNotificationVtbl {
    QueryInterface: CUserNotification_IUserNotification_QueryInterface,
    AddRef: CUserNotification_IUserNotification_AddRef,
    Release: CUserNotification_IUserNotification_Release,
    SetBalloonInfo: CUserNotification_IUserNotification_SetBalloonInfo,
    SetBalloonRetry: CUserNotification_IUserNotification_SetBalloonRetry,
    SetIconInfo: CUserNotification_IUserNotification_SetIconInfo,
    Show: CUserNotification_IUserNotification_Show,
    PlaySound: CUserNotification_IUserNotification_PlaySound,
};

unsafe fn CUserNotification_QueryInterface(
    this: *mut CUserNotification, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUserNotification)) {
        *ppv = core::ptr::addr_of_mut!((*this).IUserNotification_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CUserNotification_AddRef(this);
    S_OK
}

unsafe fn CUserNotification_AddRef(this: *mut CUserNotification) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CUserNotification_Release(this: *mut CUserNotification) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CUserNotification_dtor_body(this);
        core::ptr::drop_in_place(&mut (*this).m_szTip);
        core::ptr::drop_in_place(&mut (*this).m_szInfo);
        core::ptr::drop_in_place(&mut (*this).m_szInfoTitle);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_UserNotification currently maps to CUserNotification in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CUserNotification_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CUserNotification>()) as *mut CUserNotification;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CUserNotification {
        IUserNotification_iface: IUserNotification { lpVtbl: core::ptr::addr_of!(CUSERNOTIFICATION_IUSERNOTIFICATION_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWorkerWnd: core::mem::zeroed(),
        m_hIcon: core::mem::zeroed(),
        m_dwInfoFlags: core::mem::zeroed(),
        m_uShowTime: core::mem::zeroed(),
        m_uInterval: core::mem::zeroed(),
        m_cRetryCount: core::mem::zeroed(),
        m_uContinuePoolInterval: core::mem::zeroed(),
        m_bIsShown: core::mem::zeroed(),
        m_hRes: core::mem::zeroed(),
        m_pqc: core::mem::zeroed(),
        m_szTip: CStringW::new(),
        m_szInfo: CStringW::new(),
        m_szInfoTitle: CStringW::new(),
    });
    Shell32RustObjectLock();
    CUserNotification_ctor_body(this);
    let hr = CUserNotification_QueryInterface(this, riid, ppv);
    CUserNotification_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const BALLOON_DELAYREMOVE_TIMEOUT: UINT = 250;

pub const IDR_USERNOTIFICATION: UINT = 155;

pub const ID_BALLOON_DELAYREMOVE: UINT = 2;

pub const ID_BALLOON_QUERYCONT: UINT = 3;

pub const ID_BALLOON_SHOWTIME: UINT = 4;

pub const ID_BALLOON_TIMEOUT: UINT = 1;

pub const ID_NOTIFY_ICON: UINT = 0;

pub const IDI_ERROR: LPCWSTR = 32513usize as LPCWSTR;

pub const IDI_INFORMATION: LPCWSTR = 32516usize as LPCWSTR;

pub const IDI_WARNING: LPCWSTR = 32515usize as LPCWSTR;

pub const IDI_WINLOGO: LPCWSTR = 32517usize as LPCWSTR;

pub const NIN_BALLOONHIDE: DWORD = WM_USER + 3;

pub const NIN_BALLOONSHOW: DWORD = WM_USER + 2;

pub const NIN_BALLOONTIMEOUT: DWORD = WM_USER + 4;

pub const NIN_BALLOONUSERCLICK: DWORD = WM_USER + 5;

pub const WM_TRAYNOTIFY: UINT = WM_USER + 100;

pub const ERROR_CANCELLED: DWORD = 1223;

pub const NIF_ICON: DWORD = 2;

pub const NIF_INFO: DWORD = 16;

pub const NIF_MESSAGE: DWORD = 1;

pub const NIF_TIP: DWORD = 4;

pub const NIIF_ERROR: DWORD = 3;

pub const NIIF_ICON_MASK: DWORD = 0xf;

pub const NIIF_INFO: DWORD = 1;

pub const NIIF_USER: DWORD = 4;

pub const NIIF_WARNING: DWORD = 2;

pub const NIM_ADD: DWORD = 0;

pub const NIM_DELETE: DWORD = 2;

pub const NIM_MODIFY: DWORD = 1;

pub const SND_ALIAS: DWORD = 0x10000;

pub const SND_APPLICATION: DWORD = 0x80;

pub const SND_ASYNC: DWORD = 1;

pub const SND_NODEFAULT: DWORD = 2;

pub const SND_NOSTOP: DWORD = 16;

pub const WM_DESTROY: DWORD = 2;

pub const WM_LBUTTONDOWN: DWORD = 513;

pub const WM_NCDESTROY: DWORD = 130;

pub const WM_QUERYENDSESSION: DWORD = 17;

pub const WM_RBUTTONDOWN: DWORD = 516;

pub const WM_TIMER: DWORD = 275;

pub const WM_USER: DWORD = 1024;

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IUserNotificationCallback {
    pub lpVtbl: *const IUserNotificationCallbackVtbl,
}
#[repr(C)]
pub struct IUserNotificationCallbackVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUserNotificationCallback, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUserNotificationCallback) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUserNotificationCallback) -> ULONG,
    pub OnBalloonUserClick: unsafe extern "system" fn(This: *mut IUserNotificationCallback, pt: *mut POINT) -> HRESULT,
    pub OnLeftClick: unsafe extern "system" fn(This: *mut IUserNotificationCallback, pt: *mut POINT) -> HRESULT,
    pub OnContextMenu: unsafe extern "system" fn(This: *mut IUserNotificationCallback, pt: *mut POINT) -> HRESULT,
}
extern "C" {
    pub static IID_IUserNotificationCallback: IID;
}

pub type WPARAM = UINT_PTR;
pub type LRESULT = LONG_PTR;
pub use crate::island::MSG;
pub type WNDPROC = Option<unsafe extern "system" fn(
    HWND, UINT, WPARAM, LPARAM,
) -> LRESULT>;
#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: DWORD, pub hWnd: HWND, pub uID: UINT, pub uFlags: UINT,
    pub uCallbackMessage: UINT, pub hIcon: HICON, pub szTip: [WCHAR; 128],
    pub dwState: DWORD, pub dwStateMask: DWORD, pub szInfo: [WCHAR; 256],
    pub uTimeout: UINT, pub szInfoTitle: [WCHAR; 64], pub dwInfoFlags: DWORD,
    pub guidItem: GUID, pub hBalloonIcon: HICON,
}
pub type PNOTIFYICONDATAW = *mut NOTIFYICONDATAW;
pub const NOTIFYICONDATAW_V3_SIZE: DWORD =
    core::mem::offset_of!(NOTIFYICONDATAW, hBalloonIcon) as DWORD;

macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }

unsafe extern "system" fn CUserNotification_WorkerWndProc(
    window: HWND, message: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    let this = GetWindowLongPtrW(window, 0) as *mut CUserNotification;
    CUserNotification_WorkerWndProc_body(
        this, window, message, wparam, lparam,
    )
}

extern "system" {
    fn CopyIcon(icon: HICON) -> HICON;
    #[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")]
    fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;
    #[link_name = "LoadIconW"] fn LoadIcon(instance: HINSTANCE, name: LPCWSTR) -> HICON;
    fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL;
    fn PostQuitMessage(exit_code: INT);
    #[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")]
    fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
    fn Shell_NotifyIconW(message: DWORD, data: *mut NOTIFYICONDATAW) -> BOOL;
    fn lstrlenW(text: LPCWSTR) -> INT;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],
) -> HRESULT {
    // strsafe.h StringCbPrintfW, for the conversions translated call sites use: `%s` and `%%`.
    // Anything else is REFUSED (STRSAFE_E_INVALID_PARAMETER, empty result). This used to copy
    // an unknown conversion through literally and leave its argument unconsumed — and it did
    // not know `%%` either, so COpenWithList's `"%s" "%%1"` registered `"app.exe" "%%1"` as an
    // Open-With command line instead of `"app.exe" "%1"`, with nothing red anywhere.
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit == b'%' as WCHAR {
            let spec = *format.add(input + 1);
            if spec == b'%' as WCHAR {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = b'%' as WCHAR; out += 1; input += 2;
                continue;
            }
            if spec != b's' as WCHAR { *dest = 0; return 0x80070057u32 as HRESULT; }
            if arg_index >= args.len() { return E_FAIL; }
            let arg = args[arg_index]; arg_index += 1; input += 2;
            if arg.is_null() { continue; }
            let mut index = 0usize;
            while *arg.add(index) != 0 {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = *arg.add(index); out += 1; index += 1;
            }
            continue;
        }
        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
        *dest.add(out) = unit; out += 1; input += 1;
    }
    *dest.add(out) = 0; S_OK
}
macro_rules! StringCbPrintfW {
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])
    };
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])
    };
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "C" {
    static CLSID_UserNotification: GUID;
}

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

extern "system" {
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            GetModuleHandleW(core::ptr::null()), id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub unsafe fn new_from_lpcwstr(text: LPCWSTR) -> Self {
        if (text as usize) <= 0xffff { return Self::from_resource(text as usize as UINT); }
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), (bytes) as SIZE_T)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
    pub unsafe fn concat(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    pub unsafe fn assign(&mut self, text: LPCWSTR) {
        self.len = 0;
        if !self.buffer.is_null() { *self.buffer = 0; }
        self.append_ptr(text);
    }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(GetModuleHandleW(core::ptr::null()), format as usize as UINT,
                resource.as_mut_ptr(), resource.len() as INT);
            resource.as_ptr()
        } else { format };
        self.len = 0;
        if self.reserve(512) {
            let _ = StringCbPrintfW!(self.buffer, self.capacity * 2, format, arg);
            self.len = lstrlenW(self.buffer) as usize;
        }
    }
    pub unsafe fn format_message(&mut self, arg: LPCWSTR) {
        let mut format = [0u16; 512];
        let count = lstrlenW(self.as_ptr()).max(0) as usize;
        let count = count.min(format.len() - 1);
        let _ = memcpy(format.as_mut_ptr(), self.as_ptr(), count * 2);
        self.format(format.as_ptr(), arg);
    }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

use core::ffi::c_void;
// One declaration per external symbol: island.rs.
use crate::island::DefWindowProcW;
use crate::island::DestroyIcon;
use crate::island::DestroyWindow;
use crate::island::DispatchMessageW;
use crate::island::GetMessageW;
use crate::island::GetModuleHandleW;
use crate::island::HeapReAlloc;
use crate::island::KillTimer;
use crate::island::LoadIconW;
use crate::island::LoadStringW;
use crate::island::SHCreateWorkerWindowW;
use crate::island::SetTimer;
use crate::island::TranslateMessage;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CUserNotification_ctor_body(this: *mut CUserNotification) -> () {
    (*this).m_hWorkerWnd = core::ptr::null_mut();
    (*this).m_hIcon = core::ptr::null_mut();
    (*this).m_dwInfoFlags = 0;
    (*this).m_uShowTime = 15000;
    (*this).m_uInterval = 10000;
    (*this).m_cRetryCount = UINT::MAX;
    (*this).m_uContinuePoolInterval = 0;
    (*this).m_bIsShown = FALSE;
    (*this).m_hRes = S_OK;
    (*this).m_pqc = core::ptr::null_mut();
}

pub unsafe fn CUserNotification_dtor_body(this: *mut CUserNotification) -> () {
    if !(*this).m_hWorkerWnd.is_null() {
        CUserNotification_RemoveIcon_body(this);
        DestroyWindow((*this).m_hWorkerWnd);
        (*this).m_hWorkerWnd = core::ptr::null_mut();
    }
    if !(*this).m_hIcon.is_null() {
        DestroyIcon(((*this).m_hIcon).cast());
    }
}

pub unsafe fn CUserNotification_RemoveIcon_body(this: *mut CUserNotification) -> () {
    let mut nid: NOTIFYICONDATAW = NOTIFYICONDATAW {
        cbSize: 0,
        ..unsafe { core::mem::zeroed() }
    };
    nid.cbSize = NOTIFYICONDATAW_V3_SIZE;
    nid.hWnd = (*this).m_hWorkerWnd;
    nid.uID = ID_NOTIFY_ICON;
    Shell_NotifyIconW(NIM_DELETE, &mut nid);
}

pub unsafe fn CUserNotification_DelayRemoveIcon_body(this: *mut CUserNotification, hRes: HRESULT) -> () {
    (*this).m_hRes = hRes;
    SetTimer(
        (*this).m_hWorkerWnd,
        ID_BALLOON_DELAYREMOVE as UINT_PTR,
        BALLOON_DELAYREMOVE_TIMEOUT,
        None,
    );
}

pub unsafe fn CUserNotification_TimeoutIcon_body(this: *mut CUserNotification) -> () {
    if (*this).m_cRetryCount > 0 {
        (*this).m_cRetryCount -= 1;
        SetTimer((*this).m_hWorkerWnd, ID_BALLOON_TIMEOUT as UINT_PTR, (*this).m_uInterval, None);
    } else {
        CUserNotification_DelayRemoveIcon_body(this, HRESULT_FROM_WIN32(ERROR_CANCELLED as LONG));
    }
}

pub unsafe fn CUserNotification_SetUpNotifyData_body(this: *mut CUserNotification, uFlags: UINT, pnid: PNOTIFYICONDATAW) -> () {
    (*pnid).cbSize = NOTIFYICONDATAW_V3_SIZE;
    (*pnid).hWnd = (*this).m_hWorkerWnd;
    (*pnid).uID = ID_NOTIFY_ICON;

    if (uFlags & NIF_MESSAGE) != 0 {
        (*pnid).uFlags |= NIF_MESSAGE;
        (*pnid).uCallbackMessage = WM_TRAYNOTIFY;
    }

    if (uFlags & NIF_ICON) != 0 {
        (*pnid).uFlags |= NIF_ICON;
        (*pnid).hIcon = if !(*this).m_hIcon.is_null() {
            (*this).m_hIcon
        } else {
            LoadIcon(core::ptr::null_mut(), IDI_WINLOGO)
        };
    }

    if (uFlags & NIF_TIP) != 0 {
        (*pnid).uFlags |= NIF_TIP;
        StringCchCopyW(
            core::ptr::addr_of_mut!((*pnid).szTip) as *mut u16,
            ((*pnid).szTip).len(),
            (*this).m_szTip.as_ptr(),
        );
    }

    if (uFlags & NIF_INFO) != 0 {
        (*pnid).uFlags |= NIF_INFO;
        (*pnid).dwInfoFlags = (*this).m_dwInfoFlags;
        StringCchCopyW(
            core::ptr::addr_of_mut!((*pnid).szInfo) as *mut u16,
            ((*pnid).szInfo).len(),
            (*this).m_szInfo.as_ptr(),
        );
        StringCchCopyW(
            core::ptr::addr_of_mut!((*pnid).szInfoTitle) as *mut u16,
            ((*pnid).szInfoTitle).len(),
            (*this).m_szInfoTitle.as_ptr(),
        );
    }
}

pub unsafe fn CUserNotification_SetBalloonInfo_body(this: *mut CUserNotification, pszTitle: LPCWSTR, pszText: LPCWSTR, dwInfoFlags: DWORD) -> HRESULT {
    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
    nid.cbSize = 0;

    CUserNotification_cpp_com_assign_m_szInfo(this, pszText);
    CUserNotification_cpp_com_assign_m_szInfoTitle(this, pszTitle);
    (*this).m_dwInfoFlags = dwInfoFlags;

    /* Update the notification icon if we have one */
    if (*this).m_hWorkerWnd.is_null() {
        return S_OK;
    }

    /* Modify the notification icon */
    CUserNotification_SetUpNotifyData_body(this, NIF_INFO, &mut nid);
    if Shell_NotifyIconW(NIM_MODIFY, &mut nid) != 0 {
        return S_OK;
    } else {
        return E_FAIL;
    }
}

pub unsafe fn CUserNotification_SetBalloonRetry_body(this: *mut CUserNotification, dwShowTime: DWORD, dwInterval: DWORD, cRetryCount: UINT) -> HRESULT {
    (*this).m_uShowTime = dwShowTime;
    (*this).m_uInterval = dwInterval;
    (*this).m_cRetryCount = cRetryCount;
    S_OK
}

pub unsafe fn CUserNotification_SetIconInfo_body(this: *mut CUserNotification, hIcon: HICON, pszToolTip: LPCWSTR) -> HRESULT {
    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
    nid.cbSize = 0;

    // Destroy our local icon copy
    if !(*this).m_hIcon.is_null() {
        DestroyIcon(((*this).m_hIcon).cast());
    }

    if !hIcon.is_null() {
        // Copy the icon from the user
        (*this).m_hIcon = CopyIcon(hIcon);
    } else {
        // Use the same icon as the one for the balloon if specified
        let uIcon = (*this).m_dwInfoFlags & NIIF_ICON_MASK;
        let mut pIcon: LPCWSTR = core::ptr::null();
        if uIcon == NIIF_INFO {
            pIcon = IDI_INFORMATION;
        } else if uIcon == NIIF_WARNING {
            pIcon = IDI_WARNING;
        } else if uIcon == NIIF_ERROR {
            pIcon = IDI_ERROR;
        } else if uIcon == NIIF_USER {
            pIcon = core::ptr::null();
        }

        (*this).m_hIcon = if !pIcon.is_null() {
            (LoadIconW(core::ptr::null_mut(), pIcon)).cast()
        } else {
            core::ptr::null_mut()
        };
    }

    CUserNotification_cpp_com_assign_m_szTip(this, pszToolTip);

    // Update the notification icon if we have one
    if (*this).m_hWorkerWnd.is_null() {
        return S_OK;
    }

    // Modify the notification icon
    CUserNotification_SetUpNotifyData_body(this, NIF_ICON | NIF_TIP, &mut nid);
    if Shell_NotifyIconW(NIM_MODIFY, &mut nid) != 0 {
        S_OK
    } else {
        E_FAIL
    }
}

pub unsafe fn CUserNotification_WorkerWndProc_body(this: *mut CUserNotification, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let _ = GetWindowLongPtrW(hWnd, 0);

    ASSERT!(this);
    ASSERT!(hWnd == (*this).m_hWorkerWnd);

    match uMsg {
        WM_DESTROY => {
            if (*this).m_bIsShown != 0 {
                PostQuitMessage(0);
            }
            return 0 as LRESULT;
        }

        WM_NCDESTROY => {
            SetWindowLongPtrW(hWnd, 0, (0 as isize) as _);
            (*this).m_hWorkerWnd = core::ptr::null_mut();
            return 0 as LRESULT;
        }

        WM_QUERYENDSESSION => {
            (*this).m_hRes = HRESULT_FROM_WIN32(ERROR_CANCELLED as LONG);
            CUserNotification_RemoveIcon_body(this);
            DestroyWindow((*this).m_hWorkerWnd);
            return TRUE as LRESULT;
        }

        WM_TIMER => {
            KillTimer(hWnd, wParam as UINT_PTR);

            if wParam == ID_BALLOON_TIMEOUT as WPARAM {
                let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
                nid.cbSize = 0 as u32;
                CUserNotification_SetUpNotifyData_body(this, NIF_INFO, &raw mut nid);
                Shell_NotifyIconW(NIM_MODIFY, &raw mut nid);
            } else if wParam == ID_BALLOON_DELAYREMOVE as WPARAM {
                CUserNotification_RemoveIcon_body(this);
                DestroyWindow((*this).m_hWorkerWnd);
            } else if wParam == ID_BALLOON_QUERYCONT as WPARAM {
                let pqc = (*this).m_pqc;
                if !pqc.is_null() && ((*(*pqc).lpVtbl).QueryContinue)(pqc) == S_OK {
                    SetTimer(
                        hWnd,
                        ID_BALLOON_QUERYCONT as UINT_PTR,
                        (*this).m_uContinuePoolInterval,
                        None,
                    );
                } else {
                    CUserNotification_DelayRemoveIcon_body(this, S_FALSE);
                }
            } else if wParam == ID_BALLOON_SHOWTIME as WPARAM {
                CUserNotification_TimeoutIcon_body(this);
            }
            return 0 as LRESULT;
        }

        WM_TRAYNOTIFY => {
            ASSERT!((wParam == ID_NOTIFY_ICON as WPARAM) as i32);

            match lParam as u32 {
                NIN_BALLOONSHOW => {}
                NIN_BALLOONHIDE => {}
                NIN_BALLOONTIMEOUT => {
                    CUserNotification_TimeoutIcon_body(this);
                }
                NIN_BALLOONUSERCLICK | WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
                    CUserNotification_DelayRemoveIcon_body(this, S_OK);
                }
                _ => {}
            }
            return 0 as LRESULT;
        }

        _ => {}
    }

    DefWindowProcW(hWnd, uMsg, wParam, lParam)
}

pub unsafe fn CUserNotification_Show_body(this: *mut CUserNotification, pqc: *mut IQueryContinue, dwContinuePollInterval: DWORD) -> HRESULT {
    let mut nid: NOTIFYICONDATAW = ::core::mem::zeroed();
    nid.cbSize = 0 as DWORD;

    if (*this).m_hWorkerWnd.is_null() {
        (*this).m_hWorkerWnd = SHCreateWorkerWindowW(
            Some(CUserNotification_WorkerWndProc),
            ::core::ptr::null_mut(),
            0,
            0,
            ::core::ptr::null_mut(),
            this as LONG_PTR,
        );
        if (*this).m_hWorkerWnd.is_null() {
            return E_FAIL;
        }

        CUserNotification_SetUpNotifyData_body(
            this,
            NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_INFO,
            &mut nid,
        );
        if Shell_NotifyIconW(NIM_ADD, &mut nid) == 0 {
            DestroyWindow((*this).m_hWorkerWnd);
            (*this).m_hWorkerWnd = ::core::ptr::null_mut();
            return E_FAIL;
        }
    }

    (*this).m_hRes = S_OK;
    (*this).m_pqc = pqc;
    if !pqc.is_null() {
        (*this).m_uContinuePoolInterval = dwContinuePollInterval;
        SetTimer(
            (*this).m_hWorkerWnd,
            ID_BALLOON_QUERYCONT as UINT_PTR,
            (*this).m_uContinuePoolInterval,
            None,
        );
    }

    if (nid.uFlags & NIF_INFO) != 0 && nid.szInfo[0] == 0 {
        SetTimer(
            (*this).m_hWorkerWnd,
            ID_BALLOON_SHOWTIME as UINT_PTR,
            (*this).m_uShowTime,
            None,
        );
    }

    let mut msg: MSG = ::core::mem::zeroed();
    (*this).m_bIsShown = TRUE;
    while GetMessageW(
        &mut msg,
        ::core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    (*this).m_bIsShown = FALSE;

    if !pqc.is_null() {
        KillTimer((*this).m_hWorkerWnd, ID_BALLOON_QUERYCONT as UINT_PTR);
        (*this).m_uContinuePoolInterval = 0;
    }
    (*this).m_pqc = ::core::ptr::null_mut();

    (*this).m_hRes
}

pub unsafe fn CUserNotification_PlaySound_body(this: *mut CUserNotification, pszSoundName: LPCWSTR) -> HRESULT {
    /* Call the Win32 API - Ignore the PlaySoundW() return value as on Windows */
    PlaySoundW(
        pszSoundName,
        core::ptr::null_mut(),
        SND_ALIAS | SND_APPLICATION | SND_NOSTOP | SND_NODEFAULT | SND_ASYNC,
    );
    S_OK
}
