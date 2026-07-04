//! explorer-sys — FFI substrate for the explorer.exe Rust translation
//! (explorer-rust). Sits on top of winsrv-sys (the broad user-mode Win32
//! superset) and adds the three things it stops short of:
//!
//! 1. **COM object/vtable types** (`com`): hand-rolled `#[repr(C)]`
//!    lpVtbl-first layouts — the ABI ATL's `CComObject`/`BEGIN_COM_MAP`
//!    compiles down to. In-tree precedent: `ks-rust/src/filterfactory.rs`.
//!    Starts with `IUnknown`; the shell interfaces explorer implements
//!    (ITrayWindow, IShellDesktopTray, IDeskBand, IBandSite, …) are added
//!    here as their tier-C classes are translated.
//! 2. **Externs** (`externs`): shlwapi (SHReg*/wnsprintfW), version.dll
//!    (GetFileVersionInfo*/VerQueryValueW), msvcrt wide-CRT — DLLs the
//!    winsrv bindgen wrapper never pulled.
//! 3. **Cohab globals** (`globals`): `extern "C"` homes for explorer.exe's
//!    C++ globals (hkExplorer, hProcessHeap, hExplorerInstance) while they
//!    are still defined in explorer.cpp. When explorer.cpp flips to Rust
//!    these become real Rust statics and this module inverts.
//!
//! See docs/engineering/explorer-rust-translation-plan.md.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_void;

// Re-export the substrate so explorer-rust modules can `use explorer_sys::*`
// and get one coherent namespace.
pub use winsrv_sys;

// Explorer-wide shared C++ types (TaskbarSettings, TW_STRUCKRECTS2, …).
pub mod types;

pub mod com {
    use super::c_void;
    use winsrv_sys::winsrv::{HRESULT, IID};

    /// `REFIID` — winerror.h/`guiddef.h`: `const IID &` in C++, ABI = pointer.
    pub type REFIID = *const IID;

    /// winerror.h HRESULTs (not in the winsrv bindgen surface).
    pub const S_OK: HRESULT = 0;
    pub const S_FALSE: HRESULT = 1;
    pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as HRESULT;
    pub const E_FAIL: HRESULT = 0x8000_4005u32 as HRESULT;
    pub const E_INVALIDARG: HRESULT = 0x8007_0057u32 as HRESULT;
    pub const E_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as HRESULT;
    pub const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as HRESULT;
    pub const E_POINTER: HRESULT = 0x8000_4003u32 as HRESULT;

    /// `FAILED_UNEXPECTEDLY(hr)` — wine/debug.h macro (FAILED + ERR log; the
    /// log side is compiled out like TRACE!). Returns BOOL to match C call
    /// sites (`if (FAILED_UNEXPECTEDLY(hr)) …`).
    #[inline]
    pub fn FAILED_UNEXPECTEDLY(hr: HRESULT) -> i32 {
        (hr < 0) as i32
    }

    /// `IID_PPV_ARG(&IID_X, &mut p as …)` — identity pair matching the shape
    /// rustconv emits for the C `IID_PPV_ARG(IFace, &p)` macro (which expands
    /// to two arguments: the IID and the void** slot).
    #[inline]
    pub fn IID_PPV_ARG(
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> (*const IID, *mut *mut c_void) {
        (riid, ppv)
    }

    /// `IID_IUnknown` — {00000000-0000-0000-C000-000000000046}.
    /// (`static`, not `const`: translated code takes its address.)
    pub static IID_IUnknown: IID = IID {
        Data1: 0x0000_0000,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    /// IUnknown vtable — slot order is the COM ABI, never reorder.
    /// `extern "system"` = stdcall on i386, the one true COM convention.
    #[repr(C)]
    pub struct IUnknownVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IUnknown, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IUnknown) -> u32,
        pub Release: unsafe extern "system" fn(*mut IUnknown) -> u32,
    }

    /// COM interface pointer target: lpVtbl-first, exactly what a C++
    /// `IUnknown*` points at. Inherent methods give translated code the
    /// natural `(*punk).QueryInterface(…)` call shape.
    #[repr(C)]
    pub struct IUnknown {
        pub lpVtbl: *const IUnknownVtbl,
    }

    impl IUnknown {
        #[inline]
        pub unsafe fn QueryInterface(
            &mut self,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            ((*self.lpVtbl).QueryInterface)(self, riid, ppv)
        }
        #[inline]
        pub unsafe fn AddRef(&mut self) -> u32 {
            ((*self.lpVtbl).AddRef)(self)
        }
        #[inline]
        pub unsafe fn Release(&mut self) -> u32 {
            ((*self.lpVtbl).Release)(self)
        }
    }

    // ------------------------------------------------------------------
    // Shell COM interfaces explorer CONSUMES (vtable orders verified
    // against sdk/include psdk/reactos idl+headers — see the plan doc).
    // Slot lists are the COM ABI: NEVER reorder. `iface_consumed!`
    // generates the lpVtbl-first struct + IUnknown inherent wrappers;
    // interface-specific methods are added per interface.
    // ------------------------------------------------------------------

    use winsrv_sys::winsrv::{
        BOOL, DWORD, GUID, HBITMAP, HMONITOR, HWND, LPWSTR, LRESULT, MSG, RECT, UINT, ULONG,
    };

    macro_rules! iface_consumed {
        ($iface:ident, $vtbl:ident) => {
            #[repr(C)]
            pub struct $iface {
                pub lpVtbl: *const $vtbl,
            }
            impl $iface {
                #[inline]
                pub unsafe fn QueryInterface(
                    &mut self,
                    riid: REFIID,
                    ppv: *mut *mut c_void,
                ) -> HRESULT {
                    ((*self.lpVtbl).QueryInterface)(self, riid, ppv)
                }
                #[inline]
                pub unsafe fn AddRef(&mut self) -> ULONG {
                    ((*self.lpVtbl).AddRef)(self)
                }
                #[inline]
                pub unsafe fn Release(&mut self) -> ULONG {
                    ((*self.lpVtbl).Release)(self)
                }
            }
        };
    }

    /// IMenuPopup — IUnknown → IOleWindow → IDeskBar → IMenuPopup
    /// (shobjidl.idl; {D1E7AFEB-6A2E-11d0-8C78-00C04FD918B4}).
    #[repr(C)]
    pub struct IMenuPopupVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IMenuPopup, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IMenuPopup) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IMenuPopup) -> ULONG,
        // IOleWindow
        pub GetWindow: unsafe extern "system" fn(*mut IMenuPopup, *mut HWND) -> HRESULT,
        pub ContextSensitiveHelp: unsafe extern "system" fn(*mut IMenuPopup, BOOL) -> HRESULT,
        // IDeskBar
        pub SetClient: unsafe extern "system" fn(*mut IMenuPopup, *mut IUnknown) -> HRESULT,
        pub GetClient: unsafe extern "system" fn(*mut IMenuPopup, *mut *mut IUnknown) -> HRESULT,
        pub OnPosRectChangeDB: unsafe extern "system" fn(*mut IMenuPopup, *mut RECT) -> HRESULT,
        // IMenuPopup
        pub Popup: unsafe extern "system" fn(*mut IMenuPopup, *mut c_void, *mut c_void, u32) -> HRESULT,
        pub OnSelect: unsafe extern "system" fn(*mut IMenuPopup, DWORD) -> HRESULT,
        pub SetSubMenu: unsafe extern "system" fn(*mut IMenuPopup, *mut IMenuPopup, BOOL) -> HRESULT,
    }
    iface_consumed!(IMenuPopup, IMenuPopupVtbl);
    impl IMenuPopup {
        #[inline]
        pub unsafe fn GetClient(&mut self, ppunk: *mut *mut IUnknown) -> HRESULT {
            ((*self.lpVtbl).GetClient)(self, ppunk)
        }
        #[inline]
        pub unsafe fn Popup(
            &mut self,
            ppt: *mut c_void,
            prcExclude: *mut c_void,
            dwFlags: u32,
        ) -> HRESULT {
            ((*self.lpVtbl).Popup)(self, ppt, prcExclude, dwFlags)
        }
        #[inline]
        pub unsafe fn OnSelect(&mut self, dwSelectType: DWORD) -> HRESULT {
            ((*self.lpVtbl).OnSelect)(self, dwSelectType)
        }
    }
    /// MPOS_* (IMenuPopup::OnSelect)
    pub const MPOS_EXECUTE: DWORD = 0;
    pub const MPOS_FULLCANCEL: DWORD = 1;
    pub const MPOS_CANCELLEVEL: DWORD = 2;

    /// IBanneredBar (shlobj_undoc.h; {596A9A94-013E-11d1-8D34-00A0C90F2719}).
    #[repr(C)]
    pub struct IBanneredBarVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IBanneredBar, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IBanneredBar) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IBanneredBar) -> ULONG,
        pub SetIconSize: unsafe extern "system" fn(*mut IBanneredBar, DWORD) -> HRESULT,
        pub GetIconSize: unsafe extern "system" fn(*mut IBanneredBar, *mut DWORD) -> HRESULT,
        pub SetBitmap: unsafe extern "system" fn(*mut IBanneredBar, HBITMAP) -> HRESULT,
        pub GetBitmap: unsafe extern "system" fn(*mut IBanneredBar, *mut HBITMAP) -> HRESULT,
    }
    iface_consumed!(IBanneredBar, IBanneredBarVtbl);
    impl IBanneredBar {
        #[inline]
        pub unsafe fn SetIconSize(&mut self, iIcon: DWORD) -> HRESULT {
            ((*self.lpVtbl).SetIconSize)(self, iIcon)
        }
        #[inline]
        pub unsafe fn SetBitmap(&mut self, hBitmap: HBITMAP) -> HRESULT {
            ((*self.lpVtbl).SetBitmap)(self, hBitmap)
        }
    }
    /// BMICON_* (shlobj_undoc.h: LARGE first)
    pub const BMICON_LARGE: DWORD = 0;
    pub const BMICON_SMALL: DWORD = 1;

    /// IMenuBand (shobjidl.idl; {568804CD-CBD7-11d0-9816-00C04FD91972}).
    #[repr(C)]
    pub struct IMenuBandVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IMenuBand, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IMenuBand) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IMenuBand) -> ULONG,
        pub IsMenuMessage: unsafe extern "system" fn(*mut IMenuBand, *mut MSG) -> HRESULT,
        pub TranslateMenuMessage:
            unsafe extern "system" fn(*mut IMenuBand, *mut MSG, *mut LRESULT) -> HRESULT,
    }
    iface_consumed!(IMenuBand, IMenuBandVtbl);
    impl IMenuBand {
        #[inline]
        pub unsafe fn IsMenuMessage(&mut self, pmsg: *mut MSG) -> HRESULT {
            ((*self.lpVtbl).IsMenuMessage)(self, pmsg)
        }
        #[inline]
        pub unsafe fn TranslateMenuMessage(
            &mut self,
            pmsg: *mut MSG,
            plRet: *mut LRESULT,
        ) -> HRESULT {
            ((*self.lpVtbl).TranslateMenuMessage)(self, pmsg, plRet)
        }
    }

    /// IInitializeObject (shlobj_undoc.h; {4622AD16-FF23-11d0-8D34-00A0C90F2719}).
    #[repr(C)]
    pub struct IInitializeObjectVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IInitializeObject, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IInitializeObject) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IInitializeObject) -> ULONG,
        pub Initialize: unsafe extern "system" fn(*mut IInitializeObject) -> HRESULT,
    }
    iface_consumed!(IInitializeObject, IInitializeObjectVtbl);
    impl IInitializeObject {
        #[inline]
        pub unsafe fn Initialize(&mut self) -> HRESULT {
            ((*self.lpVtbl).Initialize)(self)
        }
    }

    /// IBandSite (shobjidl.idl; {4CF504B0-DE96-11D0-8B3F-00A0C911E8E5}).
    #[repr(C)]
    pub struct IBandSiteVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IBandSite, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IBandSite) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IBandSite) -> ULONG,
        pub AddBand: unsafe extern "system" fn(*mut IBandSite, *mut IUnknown) -> HRESULT,
        pub EnumBands: unsafe extern "system" fn(*mut IBandSite, UINT, *mut DWORD) -> HRESULT,
        pub QueryBand: unsafe extern "system" fn(
            *mut IBandSite,
            DWORD,
            *mut *mut c_void,
            *mut DWORD,
            LPWSTR,
            i32,
        ) -> HRESULT,
        pub SetBandState: unsafe extern "system" fn(*mut IBandSite, DWORD, DWORD, DWORD) -> HRESULT,
        pub RemoveBand: unsafe extern "system" fn(*mut IBandSite, DWORD) -> HRESULT,
        pub GetBandObject:
            unsafe extern "system" fn(*mut IBandSite, DWORD, REFIID, *mut *mut c_void) -> HRESULT,
        pub SetBandSiteInfo: unsafe extern "system" fn(*mut IBandSite, *const c_void) -> HRESULT,
        pub GetBandSiteInfo: unsafe extern "system" fn(*mut IBandSite, *mut c_void) -> HRESULT,
    }
    iface_consumed!(IBandSite, IBandSiteVtbl);
    impl IBandSite {
        #[inline]
        pub unsafe fn EnumBands(&mut self, uBand: UINT, pdwBandID: *mut DWORD) -> HRESULT {
            ((*self.lpVtbl).EnumBands)(self, uBand, pdwBandID)
        }
        #[inline]
        pub unsafe fn GetBandObject(
            &mut self,
            dwBandID: DWORD,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            ((*self.lpVtbl).GetBandObject)(self, dwBandID, riid, ppv)
        }
    }

    /// IOleCommandTarget (docobj.idl; {B722BCCB-4E68-101B-A2BC-00AA00404770}).
    #[repr(C)]
    pub struct IOleCommandTargetVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IOleCommandTarget, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IOleCommandTarget) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IOleCommandTarget) -> ULONG,
        pub QueryStatus: unsafe extern "system" fn(
            *mut IOleCommandTarget,
            *const GUID,
            ULONG,
            *mut c_void,
            *mut c_void,
        ) -> HRESULT,
        pub Exec: unsafe extern "system" fn(
            *mut IOleCommandTarget,
            *const GUID,
            DWORD,
            DWORD,
            *mut c_void,
            *mut c_void,
        ) -> HRESULT,
    }
    iface_consumed!(IOleCommandTarget, IOleCommandTargetVtbl);
    impl IOleCommandTarget {
        #[inline]
        pub unsafe fn Exec(
            &mut self,
            pguidCmdGroup: *const GUID,
            nCmdID: DWORD,
            nCmdexecopt: DWORD,
            pvaIn: *mut c_void,
            pvaOut: *mut c_void,
        ) -> HRESULT {
            ((*self.lpVtbl).Exec)(self, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
        }
    }
    /// docobj.idl enums
    pub const OLECMDID_NEW: DWORD = 2;
    pub const OLECMDID_SAVE: DWORD = 3;
    pub const OLECMDEXECOPT_DODEFAULT: DWORD = 0;

    /// ITrayWindow — explorer's OWN interface (precomp.h; no public IID).
    #[repr(C)]
    pub struct ITrayWindowVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut ITrayWindow, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut ITrayWindow) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut ITrayWindow) -> ULONG,
        pub Open: unsafe extern "system" fn(*mut ITrayWindow) -> HRESULT,
        pub Close: unsafe extern "system" fn(*mut ITrayWindow) -> HRESULT,
        pub GetHWND: unsafe extern "system" fn(*mut ITrayWindow) -> HWND,
        pub IsSpecialHWND: unsafe extern "system" fn(*mut ITrayWindow, HWND) -> BOOL,
        pub IsHorizontal: unsafe extern "system" fn(*mut ITrayWindow) -> BOOL,
        pub DisplayProperties: unsafe extern "system" fn(*mut ITrayWindow) -> HWND,
        pub ExecContextMenuCmd: unsafe extern "system" fn(*mut ITrayWindow, UINT) -> BOOL,
        pub Lock: unsafe extern "system" fn(*mut ITrayWindow, BOOL) -> BOOL,
        pub IsTaskWnd: unsafe extern "system" fn(*mut ITrayWindow, HWND) -> BOOL,
        pub NotifyFullScreenToAppBars:
            unsafe extern "system" fn(*mut ITrayWindow, HMONITOR, BOOL) -> HRESULT,
    }
    iface_consumed!(ITrayWindow, ITrayWindowVtbl);

    /// IShellDesktopTray (shlobj_undoc.h; IID hand-defined in traywnd.cpp).
    #[repr(C)]
    pub struct IShellDesktopTrayVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IShellDesktopTray, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IShellDesktopTray) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IShellDesktopTray) -> ULONG,
        pub GetState: unsafe extern "system" fn(*mut IShellDesktopTray) -> ULONG,
        pub GetTrayWindow: unsafe extern "system" fn(*mut IShellDesktopTray, *mut HWND) -> HRESULT,
        pub RegisterDesktopWindow: unsafe extern "system" fn(*mut IShellDesktopTray, HWND) -> HRESULT,
        pub Unknown: unsafe extern "system" fn(*mut IShellDesktopTray, DWORD, DWORD) -> HRESULT,
    }
    iface_consumed!(IShellDesktopTray, IShellDesktopTrayVtbl);

    // IIDs/CLSIDs resolved from the cmake `uuid` static lib at exe link time
    // (sdk/lib/uuid emits real const-GUID data symbols; verified). The one
    // exception, IID_IShellDesktopTray, is defined in traywnd.cpp (C++ still)
    // and extern-declared in precomp.h — link resolves it the same way.
    extern "C" {
        pub static IID_IMenuPopup: IID;
        pub static IID_IBanneredBar: IID;
        pub static IID_IMenuBand: IID;
        pub static IID_IInitializeObject: IID;
        pub static IID_IBandSite: IID;
        pub static IID_IOleCommandTarget: IID;
        pub static CGID_ShellServiceObject: GUID;
        pub static CLSID_StartMenu: GUID;
        pub static CLSID_BandSiteMenu: GUID;
        pub static CLSID_RebarBandSite: GUID;
        pub static IID_IShellDesktopTray: IID;
    }

    /// CLSCTX_INPROC_SERVER — wtypesbase.h.
    pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;

    // ole32.dll
    extern "system" {
        pub fn CoCreateInstance(
            rclsid: *const GUID,
            pUnkOuter: *mut IUnknown,
            dwClsContext: DWORD,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT;
    }
}

pub mod externs {
    use super::c_void;
    use winsrv_sys::winsrv::{BOOL, DWORD, LPCWSTR, LPWSTR, UINT};

    /// `va_list` — re-export winsrv's bindgen alias (same item, so the
    /// double glob import in translated modules stays unambiguous). Stable
    /// Rust can't DEFINE C-variadic fns, so variadic shells stay as C++
    /// trampolines that forward this to a Rust `…V` worker.
    pub use winsrv_sys::winsrv::va_list;

    // Predefined registry roots (winreg.h) — same shape as userinit's preamble.
    pub const HKEY_LOCAL_MACHINE: winsrv_sys::winsrv::HKEY =
        0x8000_0002usize as winsrv_sys::winsrv::HKEY;
    pub const HKEY_CURRENT_USER: winsrv_sys::winsrv::HKEY =
        0x8000_0001usize as winsrv_sys::winsrv::HKEY;

    // shlwapi.dll
    extern "system" {
        pub fn SHRegGetBoolUSValueW(
            pszSubKey: LPCWSTR,
            pszValueName: LPCWSTR,
            fIgnoreHKCU: BOOL,
            fDefault: BOOL,
        ) -> BOOL;
        pub fn SHRegSetUSValueW(
            pszSubKey: LPCWSTR,
            pszValueName: LPCWSTR,
            dwType: DWORD,
            pvData: *const c_void,
            cbData: DWORD,
            dwFlags: UINT,
        ) -> i32;
    }
    /// `SHREGSET_FORCE_HKCU` — shlwapi.h.
    pub const SHREGSET_FORCE_HKCU: UINT = 0x0000_0001;

    // shlwapi wnsprintfW is cdecl-variadic.
    extern "C" {
        pub fn wnsprintfW(pszDest: LPWSTR, cchDest: i32, pszFmt: LPCWSTR, ...) -> i32;
    }

    // shlwapi COM helper (exported by name) + shell32 policy query + the
    // shlwapi registry-value convenience wrappers explorer settings use.
    extern "system" {
        pub fn IUnknown_SetSite(
            punk: *mut crate::com::IUnknown,
            punkSite: *mut crate::com::IUnknown,
        ) -> winsrv_sys::winsrv::HRESULT;
        pub fn SHRestricted(rest: u32) -> DWORD;
        // LSTATUS return typed as DWORD so `== ERROR_SUCCESS` (a u32 const in
        // the bindgen surface) needs no cast at the many call sites.
        pub fn SHSetValueW(
            hkey: winsrv_sys::winsrv::HKEY,
            pszSubKey: LPCWSTR,
            pszValue: LPCWSTR,
            dwType: DWORD,
            pvData: *const c_void,
            cbData: DWORD,
        ) -> DWORD;
        pub fn SHGetValueW(
            hkey: winsrv_sys::winsrv::HKEY,
            pszSubKey: LPCWSTR,
            pszValue: LPCWSTR,
            pdwType: *mut DWORD,
            pvData: *mut c_void,
            pcbData: *mut DWORD,
        ) -> DWORD;
    }

    // RESTRICTIONS values used by explorer (shlobj.h enum; implicit-increment
    // values computed from the enum text — keep in sync if more are needed).
    pub const REST_NORUN: u32 = 0x0000_0001;
    pub const REST_NOSETFOLDERS: u32 = 0x0000_0010;
    pub const REST_STARTMENULOGOFF: u32 = 0x1000_0000;
    pub const REST_NOFAVORITESMENU: u32 = 0x4000_0006;
    pub const REST_NOCONTROLPANEL: u32 = 0x4000_0027;
    pub const REST_NONETWORKCONNECTIONS: u32 = 0x4000_0031;
    pub const REST_NOLOCALMACHINERUN: u32 = 0x4000_0046;
    pub const REST_NOLOCALMACHINERUNONCE: u32 = 0x4000_0048;
    pub const REST_NOCURRENTUSERRUNONCE: u32 = 0x4000_0049;
    pub const REST_NOSMMYDOCS: u32 = 0x4000_004F;
    pub const REST_NOSMMYPICS: u32 = 0x4000_0050;

    // comctl32 DPA (dynamic pointer array) — shellservice.cpp's HDPA of
    // started shell-service IOleCommandTargets.
    pub type HDPA = *mut c_void;
    pub type PFNDPAENUMCALLBACK =
        Option<unsafe extern "system" fn(p: *mut c_void, pData: *mut c_void) -> i32>;
    extern "system" {
        pub fn DPA_Create(cItemGrow: i32) -> HDPA;
        pub fn DPA_Destroy(hdpa: HDPA) -> BOOL;
        pub fn DPA_InsertPtr(hdpa: HDPA, i: i32, p: *mut c_void) -> i32;
        pub fn DPA_GetPtr(hdpa: HDPA, i: i32) -> *mut c_void;
        pub fn DPA_DeletePtr(hdpa: HDPA, i: i32) -> *mut c_void;
        pub fn DPA_EnumCallback(hdpa: HDPA, pfnCB: PFNDPAENUMCALLBACK, pData: *mut c_void);
        pub fn DPA_DestroyCallback(hdpa: HDPA, pfnCB: PFNDPAENUMCALLBACK, pData: *mut c_void);
    }
    /// `DPA_AppendPtr` — commctrl.h macro over DPA_InsertPtr(DA_LAST).
    #[inline]
    pub unsafe fn DPA_AppendPtr(hdpa: HDPA, p: *mut c_void) -> i32 {
        DPA_InsertPtr(hdpa, 0x7FFF_FFFF, p)
    }

    // version.dll
    extern "system" {
        pub fn GetFileVersionInfoSizeW(lptstrFilename: LPCWSTR, lpdwHandle: *mut DWORD) -> DWORD;
        pub fn GetFileVersionInfoW(
            lptstrFilename: LPCWSTR,
            dwHandle: DWORD,
            dwLen: DWORD,
            lpData: *mut c_void,
        ) -> BOOL;
        pub fn VerQueryValueW(
            pBlock: *const c_void,
            lpSubBlock: LPCWSTR,
            lplpBuffer: *mut *mut c_void,
            puLen: *mut UINT,
        ) -> BOOL;
    }

    // msvcrt wide CRT (cdecl).
    extern "C" {
        pub fn wcsncpy(dest: *mut u16, src: *const u16, count: usize) -> *mut u16;
        pub fn wcslen(s: *const u16) -> usize;
        pub fn _vsnwprintf(
            buffer: *mut u16,
            count: usize,
            format: *const u16,
            argptr: va_list,
        ) -> i32;
    }

    use winsrv_sys::winsrv::{HANDLE, HINSTANCE, HKEY, HRESULT, HWND, INT_PTR, LPARAM, WPARAM};

    pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE; // (HANDLE)-1
    /// shlobj.h CSIDL values used by startup.cpp.
    pub const CSIDL_STARTUP: i32 = 0x0007;
    pub const CSIDL_COMMON_STARTUP: i32 = 0x0018;

    /// winerror.h `HRESULT_FROM_WIN32`.
    #[inline]
    pub fn HRESULT_FROM_WIN32(x: u32) -> HRESULT {
        if (x as i32) <= 0 {
            x as i32
        } else {
            ((x & 0x0000_FFFF) | 0x8007_0000) as i32
        }
    }

    pub type DLGPROC =
        Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> INT_PTR>;

    // user32 / ole32 / shell32 gap-fill beyond the winsrv bindgen surface.
    extern "system" {
        pub fn DialogBoxParamW(
            hInstance: HINSTANCE,
            lpTemplateName: LPCWSTR,
            hWndParent: HWND,
            lpDialogFunc: DLGPROC,
            dwInitParam: LPARAM,
        ) -> INT_PTR;
        pub fn CLSIDFromString(
            lpsz: *const u16,
            pclsid: *mut winsrv_sys::winsrv::GUID,
        ) -> HRESULT;
        /// undocshell.h; exported by shell32.
        pub fn SHCreateSessionKey(samDesired: u32, phKey: *mut HKEY) -> HRESULT;
    }

    /// strsafe.h `StringCbCatW` — C-side it's an inline, so no import exists;
    /// faithful minimal impl (cb = BYTE capacity incl. NUL).
    #[inline]
    pub unsafe fn StringCbCatW(dst: *mut u16, cb_dst: usize, src: *const u16) -> HRESULT {
        let cch_dst = cb_dst / 2;
        if cch_dst == 0 {
            return 0x8007_0057u32 as i32; // STRSAFE_E_INVALID_PARAMETER
        }
        let used = wcslen(dst);
        let mut i = used;
        let mut j = 0usize;
        while i < cch_dst - 1 && *src.add(j) != 0 {
            *dst.add(i) = *src.add(j);
            i += 1;
            j += 1;
        }
        *dst.add(i) = 0;
        if *src.add(j) != 0 {
            0x8007_007Au32 as i32 // STRSAFE_E_INSUFFICIENT_BUFFER
        } else {
            0
        }
    }
}

pub mod globals {
    use winsrv_sys::winsrv::{HANDLE, HINSTANCE, HKEY};

    // explorer.exe C++ globals (defined in explorer.cpp; C++ namespace-scope
    // variables are NOT name-mangled, so plain extern "C" statics bind).
    // When explorer.cpp is translated these move into explorer-rust.
    extern "C" {
        pub static mut hkExplorer: HKEY;
        pub static mut hProcessHeap: HANDLE;
        pub static mut hExplorerInstance: HINSTANCE;
        pub static mut bExplorerIsShell: winsrv_sys::winsrv::BOOL;
    }

    // C++-side helpers still owned by untranslated .cpp files, exposed with
    // C linkage via precomp.h extern "C" declarations.
    extern "C" {
        /// traywnd.cpp — pump the tray window's message queue.
        pub fn TrayProcessMessages(Tray: *mut crate::com::ITrayWindow);
    }
}

/// `LOBYTE` — windef.h macro.
#[inline]
pub fn LOBYTE(w: u16) -> u8 {
    (w & 0xFF) as u8
}

/// `MAKEINTRESOURCEW` — winuser.h macro (ordinal-as-pointer).
#[inline]
pub fn MAKEINTRESOURCEW(i: u16) -> *const u16 {
    i as usize as *const u16
}

// Wine debug-channel macros (TRACE/WARN/ERR/FIXME). The C++ side routes these
// through wine_dbg_log with runtime channel gating; the Rust cohab compiles
// them out for now (channels are off in normal boots). TODO: route to
// OutputDebugStringA behind a feature if a debug session needs them.
#[macro_export]
macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
#[macro_export]
macro_rules! WARN {
    ($($t:tt)*) => {{}};
}
#[macro_export]
macro_rules! ERR {
    ($($t:tt)*) => {{}};
}
#[macro_export]
macro_rules! FIXME {
    ($($t:tt)*) => {{}};
}
