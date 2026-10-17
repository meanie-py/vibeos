// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu {
    pub lpVtbl: *const IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}
extern "C" {
    pub static IID_IShellExtInit: IID;
}

// Deterministic COM scaffold for CCopyAsPathMenu; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CCopyAsPathMenu {
    pub IContextMenu_iface: IContextMenu,
    pub IShellExtInit_iface: IShellExtInit,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pDataObject: *mut IDataObject,
}

unsafe fn CCopyAsPathMenu_cpp_com_assign_m_pDataObject(this: *mut CCopyAsPathMenu, value: *mut IDataObject) {
    let old = (*this).m_pDataObject;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pDataObject = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_AddRef(this)
}
unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_Release(this)
}
unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CCopyAsPathMenu_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IContextMenu_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CCOPYASPATHMENU_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CCopyAsPathMenu_IContextMenu_QueryInterface,
    AddRef: CCopyAsPathMenu_IContextMenu_AddRef,
    Release: CCopyAsPathMenu_IContextMenu_Release,
    QueryContextMenu: CCopyAsPathMenu_IContextMenu_QueryContextMenu,
    InvokeCommand: CCopyAsPathMenu_IContextMenu_InvokeCommand,
    GetCommandString: CCopyAsPathMenu_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CCopyAsPathMenu_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IShellExtInit_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CCopyAsPathMenu_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IShellExtInit_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_AddRef(this)
}
unsafe extern "system" fn CCopyAsPathMenu_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IShellExtInit_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_Release(this)
}
unsafe extern "system" fn CCopyAsPathMenu_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CCopyAsPathMenu, IShellExtInit_iface)) as *mut CCopyAsPathMenu;
    CCopyAsPathMenu_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CCOPYASPATHMENU_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CCopyAsPathMenu_IShellExtInit_QueryInterface,
    AddRef: CCopyAsPathMenu_IShellExtInit_AddRef,
    Release: CCopyAsPathMenu_IShellExtInit_Release,
    Initialize: CCopyAsPathMenu_IShellExtInit_Initialize,
};

unsafe fn CCopyAsPathMenu_QueryInterface(
    this: *mut CCopyAsPathMenu, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CCopyAsPathMenu_AddRef(this);
    S_OK
}

unsafe fn CCopyAsPathMenu_AddRef(this: *mut CCopyAsPathMenu) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CCopyAsPathMenu_Release(this: *mut CCopyAsPathMenu) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CCopyAsPathMenu_dtor_body(this);
        CCopyAsPathMenu_cpp_com_assign_m_pDataObject(this, core::ptr::null_mut());
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

// Activation: CLSID_CopyAsPathMenu currently maps to CCopyAsPathMenu in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CCopyAsPathMenu_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<CCopyAsPathMenu>()) as _) as *mut CCopyAsPathMenu;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CCopyAsPathMenu {
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CCOPYASPATHMENU_ICONTEXTMENU_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CCOPYASPATHMENU_ISHELLEXTINIT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pDataObject: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CCopyAsPathMenu_ctor_body(this);
    let hr = CCopyAsPathMenu_QueryInterface(this, riid, ppv);
    CCopyAsPathMenu_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDC_COPYASPATH: UINT = 0;

pub const IDS_COPYASPATHMENU: UINT = 30328;

#[repr(C)]
pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }
pub static mut g_VerbMap: [CMVERBMAP; 2] = [
    CMVERBMAP { Verb: b"copyaspath\0".as_ptr() as LPCSTR, CmdId: IDC_COPYASPATH as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 as WORD },
];

extern "system" {
    fn GlobalLock(memory: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;
    fn GlobalFree(memory: HGLOBAL) -> HGLOBAL;
    fn EmptyClipboard() -> BOOL;
    fn SetClipboardData(format: UINT, memory: HANDLE) -> HANDLE;
    fn CloseClipboard() -> BOOL;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHCreateShellItemArrayFromDataObject(data: *mut IDataObject, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"] fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, map: *const CMVERBMAP) -> INT;
    #[link_name = "Shell32RustGetCommandStringImpl"] fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, capacity: UINT, map: *const CMVERBMAP) -> HRESULT;
}

extern "C" {
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
}
#[inline]
unsafe fn CopyMemory(dst: LPVOID, src: LPCVOID, bytes: usize) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory(dst: LPVOID, bytes: usize) {
    let _ = memset(dst, 0, bytes);
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
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, (bytes) as _)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), (bytes) as SIZE_T)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len).cast(), text.cast(),
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
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
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer.cast()); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_80E09B2F645F: [WCHAR; 3] = [0x000D, 0x000A, 0x0000];

use core::ffi::c_void;
// One declaration per external symbol: island.rs.
use crate::island::GetModuleHandleW;
use crate::island::GlobalAlloc;
use crate::island::HeapAlloc;
use crate::island::HeapReAlloc;
use crate::island::InsertMenuItemW;
use crate::island::LoadStringW;
use crate::island::OpenClipboard;
use crate::shellcom::*;
use crate::types::*;

pub unsafe fn CCopyAsPathMenu_ctor_body(this: *mut CCopyAsPathMenu) -> () {}

pub unsafe fn CCopyAsPathMenu_dtor_body(this: *mut CCopyAsPathMenu) -> () {}

pub unsafe fn SetClipboard_body(cf: UINT, data: *const core::ffi::c_void, size: SIZE_T) -> DWORD {
    let mut succ: i32 = FALSE;
    let handle = GlobalAlloc(GMEM_MOVEABLE | GMEM_SHARE, (size) as SIZE_T);
    if !handle.is_null() {
        let clipdata = GlobalLock(handle);
        if !clipdata.is_null() {
            CopyMemory(clipdata, data, (size) as _);
            GlobalUnlock(handle);
            let opened = OpenClipboard(core::ptr::null_mut());
            if opened != 0 {
                EmptyClipboard();
                succ = if SetClipboardData(cf, handle).is_null() { 0 } else { 1 };
                CloseClipboard();
            }
        }
        if succ == 0 {
            GlobalFree(handle);
        }
    }
    if succ != 0 { ERROR_SUCCESS as DWORD } else { GetLastError() }
}

pub unsafe fn SetClipboardFromString_body(str: LPCWSTR) -> DWORD {
    let cch: SIZE_T = lstrlenW(str) as SIZE_T + 1;
    let size = cch.wrapping_mul((core::mem::size_of::<u16>()) as _);
    if size > cch {
        SetClipboard_body(CF_UNICODETEXT, str as *const core::ffi::c_void, size)
    } else {
        ERROR_BUFFER_OVERFLOW
    }
}

pub unsafe fn AppendToPathList_body(paths: &mut CStringW, path: LPCWSTR, index: DWORD) -> () {
    if index != 0 {
        *paths += CPP_COM_WSTR_80E09B2F645F.as_ptr();
    }
    let quote = StrChrW(path, 0x0020u16);
    if !quote.is_null() {
        *paths += 0x0022u16;
    }
    *paths += path;
    if !quote.is_null() {
        *paths += 0x0022u16;
    }
}

pub unsafe fn CCopyAsPathMenu_DoCopyAsPath_body(this: *mut CCopyAsPathMenu, pdto: *mut IDataObject) -> HRESULT {
    let _ = this; // unused
    let mut paths = CStringW::new();
    let mut array: *mut IShellItemArray = core::ptr::null_mut();
    let mut hr: HRESULT = SHCreateShellItemArrayFromDataObject(pdto, core::ptr::addr_of!(IID_IShellItemArray), core::ptr::addr_of_mut!(array).cast());
    if hr >= 0 {
        let mut i: u32 = 0;
        let mut count: u32 = 0;
        let _: HRESULT = ((*(*array).lpVtbl).GetCount)(array, &mut count);
        while i < count && hr >= 0 {
            let mut item: *mut IShellItem = core::ptr::null_mut();
            hr = ((*(*array).lpVtbl).GetItemAt)(array, i, &mut item);
            if hr >= 0 {
                let mut path: *mut u16 = core::ptr::null_mut();
                hr = ((*(*item).lpVtbl).GetDisplayName)(item, SIGDN_DESKTOPABSOLUTEPARSING, &mut path);
                if hr >= 0 {
                    AppendToPathList_body(&mut paths, path, i);
                }
                if !path.is_null() {
                    CoTaskMemFree(path as *mut _);
                }
            }
            if !item.is_null() {
                ((*(*item).lpVtbl).Release)(item);
            }
            i += 1;
        }
    }
    if hr >= 0 {
        let err = SetClipboardFromString_body(paths.as_ptr());
        hr = HRESULT_FROM_WIN32(err as LONG);
    }
    if !array.is_null() {
        ((*(*array).lpVtbl).Release)(array);
    }
    hr
}

pub unsafe fn CCopyAsPathMenu_QueryContextMenu_body(this: *mut CCopyAsPathMenu, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();

    if (uFlags & CMF_NOVERBS) != 0 || (uFlags & CMF_EXTENDEDVERBS) == 0 {
        return MAKE_HRESULT(
            SEVERITY_SUCCESS,
            0,
            0,
        );
    }

    // Insert "Copy as path" – load the resource string
    let mut str_text = CStringW::from_resource(IDS_COPYASPATHMENU as UINT); // MAKINTRESOURCEW

    ZeroMemory(
        &mut mii as *mut MENUITEMINFOW as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFOW>(),
    );
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_ID | MIIM_TYPE;
    mii.fType = MFT_STRING;
    mii.wID = idCmdFirst.wrapping_add(IDC_COPYASPATH);
    mii.dwTypeData = str_text.as_mut_ptr();

    let result = InsertMenuItemW(hMenu, indexMenu, TRUE, &mii);
    if result != 0 {
        let command_count = mii
            .wID
            .wrapping_sub(idCmdFirst)
            .wrapping_add(1);
        return MAKE_HRESULT(
            SEVERITY_SUCCESS,
            0,
            command_count,
        );
    }

    MAKE_HRESULT(SEVERITY_SUCCESS, 0, 0)
}

pub unsafe fn CCopyAsPathMenu_InvokeCommand_body(this: *mut CCopyAsPathMenu, lpcmi: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let cmd_id = SHELL_MapContextMenuVerbToCmdId(lpcmi, core::ptr::addr_of!(g_VerbMap).cast());
    if cmd_id == IDC_COPYASPATH as INT {
        return CCopyAsPathMenu_DoCopyAsPath_body(this, (*this).m_pDataObject as *mut IDataObject);
    }
    E_FAIL
}

pub unsafe fn CCopyAsPathMenu_GetCommandString_body(this: *mut CCopyAsPathMenu, idCommand: UINT_PTR, uFlags: UINT, lpReserved: *mut UINT, lpszName: LPSTR, uMaxNameLen: UINT) -> HRESULT {
    SHELL_GetCommandStringImpl(
        (idCommand) as _,
        uFlags,
        lpszName,
        uMaxNameLen,
        core::ptr::addr_of!(g_VerbMap).cast(),
    )
}

pub unsafe fn CCopyAsPathMenu_Initialize_body(this: *mut CCopyAsPathMenu, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    CCopyAsPathMenu_cpp_com_assign_m_pDataObject(this, pdtobj);
    return S_OK;
}
