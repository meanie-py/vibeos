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
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate. Re-exported at the root
// so explorer-rust's `use explorer_sys::*;` keeps finding them.
pub use winedebug_rust::{ERR, FIXME, TRACE, WARN};

pub mod types;

// ATL/WTL replacement substrate for the tier-C COM/window-server classes
// (CWindowImpl→WndProc thunk, DECLARE_WND_CLASS_EX→register-once, uxtheme/
// tooltip surface). See docs/engineering/explorer-rust-translation-plan.md
// §"tier-C COM-server / WTL substrate".
pub mod server;

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

    /// `IID_IOleWindow` — {00000114-0000-0000-C000-000000000046} (oleidl.h).
    /// Well-known OLE IID; value-compared in QI (symbol identity irrelevant).
    pub static IID_IOleWindow: IID = IID {
        Data1: 0x0000_0114,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    /// IOleWindow (oleidl.h) — IUnknown + GetWindow + ContextSensitiveHelp.
    /// explorer's tier-C window controls (trayclock, …) IMPLEMENT this, so the
    /// full vtable is spelled out (implemented, not just consumed).
    #[repr(C)]
    pub struct IOleWindowVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IOleWindow, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IOleWindow) -> u32,
        pub Release: unsafe extern "system" fn(*mut IOleWindow) -> u32,
        pub GetWindow:
            unsafe extern "system" fn(*mut IOleWindow, *mut winsrv_sys::winsrv::HWND) -> HRESULT,
        pub ContextSensitiveHelp:
            unsafe extern "system" fn(*mut IOleWindow, winsrv_sys::winsrv::BOOL) -> HRESULT,
    }
    #[repr(C)]
    pub struct IOleWindow {
        pub lpVtbl: *const IOleWindowVtbl,
    }

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

    // =====================================================================
    // IMPLEMENTED interfaces for the deskband/band-site tier-C COM servers
    // (taskband, tbsite, …). These are spelled out as full vtables because
    // the tier-C classes IMPLEMENT them (build a static vtbl of Rust fns).
    // Slot order is the COM ABI (verified against psdk shobjidl.idl / objidl.idl
    // / ocidl.idl + reactos shlobj_undoc.h) — NEVER reorder. IIDs are fixed
    // GUIDs compared by value in QI (symbol identity irrelevant).
    // =====================================================================

    use winsrv_sys::winsrv::{COLORREF, LONG, WPARAM, LPARAM, CLSID};

    pub static IID_IObjectWithSite: IID = IID {
        Data1: 0xFC48_01A3, Data2: 0x2BA9, Data3: 0x11CF,
        Data4: [0xA2, 0x29, 0x00, 0xAA, 0x00, 0x3D, 0x73, 0x52],
    };
    pub static IID_IDeskBand: IID = IID {
        Data1: 0xEB0F_E172, Data2: 0x1A3A, Data3: 0x11D0,
        Data4: [0x89, 0xB3, 0x00, 0xA0, 0xC9, 0x0A, 0x90, 0xAC],
    };
    pub static IID_IPersist: IID = IID {
        Data1: 0x0000_010C, Data2: 0x0000, Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    pub static IID_IPersistStream: IID = IID {
        Data1: 0x0000_0109, Data2: 0x0000, Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    pub static IID_IWinEventHandler: IID = IID {
        Data1: 0xEA5F_2D61, Data2: 0xE008, Data3: 0x11CF,
        Data4: [0x99, 0xCB, 0x00, 0xC0, 0x4F, 0xD6, 0x44, 0x97],
    };
    /// `IID_IBandSite` value (also externed from uuid for consumers); inlined
    /// here for the taskband Exec CGID compare.
    pub static IID_IBandSite_VAL: IID = IID {
        Data1: 0x4CF5_04B0, Data2: 0xDE96, Data3: 0x11D0,
        Data4: [0x8B, 0x3F, 0x00, 0xA0, 0xC9, 0x11, 0xE8, 0xE5],
    };

    /// `POINTL` (wtypes) + `DESKBANDINFO` (shobjidl.idl) — exact layout; the
    /// band fills pdbi fields the band site reads.
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct POINTL {
        pub x: LONG,
        pub y: LONG,
    }
    #[repr(C)]
    pub struct DESKBANDINFO {
        pub dwMask: DWORD,
        pub ptMinSize: POINTL,
        pub ptMaxSize: POINTL,
        pub ptIntegral: POINTL,
        pub ptActual: POINTL,
        pub wszTitle: [u16; 256],
        pub dwModeFlags: DWORD,
        pub crBkgnd: COLORREF,
    }
    pub const DBIM_TITLE: DWORD = 0x0010;
    pub const DBIMF_VARIABLEHEIGHT: DWORD = 0x0008;
    pub const DBIF_VIEWMODE_VERTICAL: DWORD = 0x0001;
    // GW_CHILD comes from winsrv consts. TB_GETBUTTONSIZE is comctl32-only.
    pub const TB_GETBUTTONSIZE: UINT = 0x0400 + 58; // WM_USER+58

    /// IObjectWithSite (ocidl) — IUnknown + SetSite + GetSite.
    #[repr(C)]
    pub struct IObjectWithSiteVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IObjectWithSite, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IObjectWithSite) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IObjectWithSite) -> ULONG,
        pub SetSite: unsafe extern "system" fn(*mut IObjectWithSite, *mut IUnknown) -> HRESULT,
        pub GetSite:
            unsafe extern "system" fn(*mut IObjectWithSite, REFIID, *mut *mut c_void) -> HRESULT,
    }
    #[repr(C)]
    pub struct IObjectWithSite {
        pub lpVtbl: *const IObjectWithSiteVtbl,
    }

    /// IDeskBand : IDockingWindow : IOleWindow (shobjidl) — 9 slots.
    #[repr(C)]
    pub struct IDeskBandVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IDeskBand, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IDeskBand) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IDeskBand) -> ULONG,
        // IOleWindow
        pub GetWindow: unsafe extern "system" fn(*mut IDeskBand, *mut HWND) -> HRESULT,
        pub ContextSensitiveHelp: unsafe extern "system" fn(*mut IDeskBand, BOOL) -> HRESULT,
        // IDockingWindow
        pub ShowDW: unsafe extern "system" fn(*mut IDeskBand, BOOL) -> HRESULT,
        pub CloseDW: unsafe extern "system" fn(*mut IDeskBand, DWORD) -> HRESULT,
        pub ResizeBorderDW: unsafe extern "system" fn(
            *mut IDeskBand,
            *const RECT,
            *mut IUnknown,
            BOOL,
        ) -> HRESULT,
        // IDeskBand
        pub GetBandInfo:
            unsafe extern "system" fn(*mut IDeskBand, DWORD, DWORD, *mut DESKBANDINFO) -> HRESULT,
    }
    #[repr(C)]
    pub struct IDeskBand {
        pub lpVtbl: *const IDeskBandVtbl,
    }

    /// IPersistStream : IPersist (objidl) — 8 slots (serves IID_IPersist too).
    #[repr(C)]
    pub struct IPersistStreamVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IPersistStream, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IPersistStream) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IPersistStream) -> ULONG,
        // IPersist
        pub GetClassID: unsafe extern "system" fn(*mut IPersistStream, *mut CLSID) -> HRESULT,
        // IPersistStream
        pub IsDirty: unsafe extern "system" fn(*mut IPersistStream) -> HRESULT,
        pub Load: unsafe extern "system" fn(*mut IPersistStream, *mut c_void) -> HRESULT,
        pub Save: unsafe extern "system" fn(*mut IPersistStream, *mut c_void, BOOL) -> HRESULT,
        pub GetSizeMax: unsafe extern "system" fn(*mut IPersistStream, *mut c_void) -> HRESULT,
    }
    #[repr(C)]
    pub struct IPersistStream {
        pub lpVtbl: *const IPersistStreamVtbl,
    }

    /// IWinEventHandler (shlobj_undoc) — IUnknown + OnWinEvent + IsWindowOwner.
    #[repr(C)]
    pub struct IWinEventHandlerVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IWinEventHandler, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IWinEventHandler) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IWinEventHandler) -> ULONG,
        pub OnWinEvent: unsafe extern "system" fn(
            *mut IWinEventHandler,
            HWND,
            UINT,
            WPARAM,
            LPARAM,
            *mut LRESULT,
        ) -> HRESULT,
        pub IsWindowOwner: unsafe extern "system" fn(*mut IWinEventHandler, HWND) -> HRESULT,
    }
    #[repr(C)]
    pub struct IWinEventHandler {
        pub lpVtbl: *const IWinEventHandlerVtbl,
    }

    /// IOleCommandTarget IMPLEMENTED vtbl (same ABI as the consumed one above,
    /// but with a self-typed `this` for building a static vtable of Rust fns).
    #[repr(C)]
    pub struct IOleCommandTargetImplVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IOleCommandTargetImpl, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IOleCommandTargetImpl) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IOleCommandTargetImpl) -> ULONG,
        pub QueryStatus: unsafe extern "system" fn(
            *mut IOleCommandTargetImpl,
            *const GUID,
            ULONG,
            *mut c_void,
            *mut c_void,
        ) -> HRESULT,
        pub Exec: unsafe extern "system" fn(
            *mut IOleCommandTargetImpl,
            *const GUID,
            DWORD,
            DWORD,
            *mut c_void,
            *mut c_void,
        ) -> HRESULT,
    }
    #[repr(C)]
    pub struct IOleCommandTargetImpl {
        pub lpVtbl: *const IOleCommandTargetImplVtbl,
    }

    // --- start-menu site (startmnusite) implemented interfaces ---

    pub static IID_IServiceProvider: IID = IID {
        Data1: 0x6D51_40C1, Data2: 0x7436, Data3: 0x11CE,
        Data4: [0x80, 0x34, 0x00, 0xAA, 0x00, 0x60, 0x09, 0xFA],
    };
    pub static IID_ITrayPriv: IID = IID {
        Data1: 0x4622_AD10, Data2: 0xFF23, Data3: 0x11D0,
        Data4: [0x8D, 0x34, 0x00, 0xA0, 0xC9, 0x0F, 0x27, 0x19],
    };
    /// `SID_SMenuPopup` — service id (== IMenuPopup's IID value).
    pub static SID_SMenuPopup: GUID = GUID {
        Data1: 0xD1E7_AFEB, Data2: 0x6A2E, Data3: 0x11D0,
        Data4: [0x8C, 0x78, 0x00, 0xC0, 0x4F, 0xD9, 0x18, 0xB4],
    };
    /// `IID_IMenuPopup` value (also externed for consumers); inlined for QI.
    pub static IID_IMenuPopup_VAL: IID = IID {
        Data1: 0xD1E7_AFEB, Data2: 0x6A2E, Data3: 0x11D0,
        Data4: [0x8C, 0x78, 0x00, 0xC0, 0x4F, 0xD9, 0x18, 0xB4],
    };

    /// IServiceProvider (servprov.idl) — IUnknown + QueryService.
    #[repr(C)]
    pub struct IServiceProviderVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IServiceProvider, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
        pub QueryService: unsafe extern "system" fn(
            *mut IServiceProvider,
            *const GUID,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
    }
    #[repr(C)]
    pub struct IServiceProvider {
        pub lpVtbl: *const IServiceProviderVtbl,
    }

    /// ITrayPriv : IOleWindow (shlobj_undoc.h) — 8 slots.
    #[repr(C)]
    pub struct ITrayPrivVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut ITrayPriv, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut ITrayPriv) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut ITrayPriv) -> ULONG,
        // IOleWindow
        pub GetWindow: unsafe extern "system" fn(*mut ITrayPriv, *mut HWND) -> HRESULT,
        pub ContextSensitiveHelp: unsafe extern "system" fn(*mut ITrayPriv, BOOL) -> HRESULT,
        // ITrayPriv
        pub Execute:
            unsafe extern "system" fn(*mut ITrayPriv, *mut c_void, *const c_void) -> HRESULT,
        pub Unknown: unsafe extern "system" fn(
            *mut ITrayPriv,
            *mut c_void,
            *mut c_void,
            *mut c_void,
            *mut c_void,
        ) -> HRESULT,
        pub AppendMenu: unsafe extern "system" fn(*mut ITrayPriv, *mut winsrv_sys::winsrv::HMENU) -> HRESULT,
    }
    #[repr(C)]
    pub struct ITrayPriv {
        pub lpVtbl: *const ITrayPrivVtbl,
    }

    // --- start-button context menu (startctxmnu): IContextMenu (impl+consume)
    //     + IShellFolder (consume). PIDL types opaque. ---

    use winsrv_sys::winsrv::{HANDLE, HMENU, INT, LPCSTR, LPCWSTR, LPSTR, POINT, UINT_PTR};

    pub type LPITEMIDLIST = *mut c_void;
    pub type LPCITEMIDLIST = *const c_void;

    pub static IID_IContextMenu: IID = IID {
        Data1: 0x0002_14E4, Data2: 0x0000, Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };
    pub static IID_IShellFolder: IID = IID {
        Data1: 0x0002_14E6, Data2: 0x0000, Data3: 0x0000,
        Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
    };

    /// IContextMenu : IUnknown (shobjidl.idl) — 6 slots. IMPLEMENTED + CONSUMED.
    #[repr(C)]
    pub struct IContextMenuVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IContextMenu, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IContextMenu) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IContextMenu) -> ULONG,
        pub QueryContextMenu:
            unsafe extern "system" fn(*mut IContextMenu, HMENU, UINT, UINT, UINT, UINT) -> HRESULT,
        pub InvokeCommand:
            unsafe extern "system" fn(*mut IContextMenu, *const CMINVOKECOMMANDINFO) -> HRESULT,
        pub GetCommandString: unsafe extern "system" fn(
            *mut IContextMenu,
            UINT_PTR,
            UINT,
            *mut UINT,
            LPSTR,
            UINT,
        ) -> HRESULT,
    }
    #[repr(C)]
    pub struct IContextMenu {
        pub lpVtbl: *const IContextMenuVtbl,
    }

    /// IShellFolder : IUnknown (shobjidl.idl) — 13-slot CONSUMED vtbl. Only
    /// BindToObject[5] + GetUIObjectOf[10] are called; the rest are ordered
    /// placeholders so those two land at the right offsets.
    #[repr(C)]
    pub struct IShellFolderVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IShellFolder, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IShellFolder) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IShellFolder) -> ULONG,
        pub ParseDisplayName: unsafe extern "system" fn(
            *mut IShellFolder,
            HWND,
            *mut c_void,
            *mut u16,
            *mut ULONG,
            *mut LPITEMIDLIST,
            *mut ULONG,
        ) -> HRESULT,
        pub EnumObjects:
            unsafe extern "system" fn(*mut IShellFolder, HWND, DWORD, *mut *mut c_void) -> HRESULT,
        pub BindToObject: unsafe extern "system" fn(
            *mut IShellFolder,
            LPCITEMIDLIST,
            *mut c_void,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
        pub BindToStorage: unsafe extern "system" fn(
            *mut IShellFolder,
            LPCITEMIDLIST,
            *mut c_void,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
        pub CompareIDs:
            unsafe extern "system" fn(*mut IShellFolder, isize, LPCITEMIDLIST, LPCITEMIDLIST) -> HRESULT,
        pub CreateViewObject:
            unsafe extern "system" fn(*mut IShellFolder, HWND, REFIID, *mut *mut c_void) -> HRESULT,
        pub GetAttributesOf:
            unsafe extern "system" fn(*mut IShellFolder, UINT, *const LPCITEMIDLIST, *mut ULONG) -> HRESULT,
        pub GetUIObjectOf: unsafe extern "system" fn(
            *mut IShellFolder,
            HWND,
            UINT,
            *const LPCITEMIDLIST,
            REFIID,
            *mut UINT,
            *mut *mut c_void,
        ) -> HRESULT,
        pub GetDisplayNameOf:
            unsafe extern "system" fn(*mut IShellFolder, LPCITEMIDLIST, DWORD, *mut c_void) -> HRESULT,
        pub SetNameOf: unsafe extern "system" fn(
            *mut IShellFolder,
            HWND,
            LPCITEMIDLIST,
            *const u16,
            DWORD,
            *mut LPITEMIDLIST,
        ) -> HRESULT,
    }
    #[repr(C)]
    pub struct IShellFolder {
        pub lpVtbl: *const IShellFolderVtbl,
    }

    /// CMINVOKECOMMANDINFO / …EX (shobjidl.idl). EX adds the wide/title/point
    /// tail; startctxmnu reads the in ptr's lpVerb and builds an EX to forward.
    #[repr(C)]
    pub struct CMINVOKECOMMANDINFO {
        pub cbSize: DWORD,
        pub fMask: DWORD,
        pub hwnd: HWND,
        pub lpVerb: LPCSTR,
        pub lpParameters: LPCSTR,
        pub lpDirectory: LPCSTR,
        pub nShow: INT,
        pub dwHotKey: DWORD,
        pub hIcon: HANDLE,
    }
    #[repr(C)]
    pub struct CMINVOKECOMMANDINFOEX {
        pub cbSize: DWORD,
        pub fMask: DWORD,
        pub hwnd: HWND,
        pub lpVerb: LPCSTR,
        pub lpParameters: LPCSTR,
        pub lpDirectory: LPCSTR,
        pub nShow: INT,
        pub dwHotKey: DWORD,
        pub hIcon: HANDLE,
        pub lpTitle: LPCSTR,
        pub lpVerbW: LPCWSTR,
        pub lpParametersW: LPCWSTR,
        pub lpDirectoryW: LPCWSTR,
        pub lpTitleW: LPCWSTR,
        pub ptInvoke: POINT,
    }

    pub const CMF_VERBSONLY: UINT = 0x0000_0002;
    pub const CMIC_MASK_UNICODE: DWORD = 0x0000_4000;
    pub const ID_SHELL_CMD_PROPERTIES: UINT = 401;
    pub const ID_SHELL_CMD_OPEN_ALL_USERS: UINT = 402;
    pub const ID_SHELL_CMD_EXPLORE_ALL_USERS: UINT = 403;
    pub const ID_SHELL_CMD_FIRST: UINT = ID_SHELL_CMD_PROPERTIES;
    pub const ID_SHELL_CMD_LAST: UINT = 413; // ID_SHELL_CMD_RESTORE_ALL

    // --- tray band site (tbsite): ITrayBandSite / IBandSiteStreamCallback
    //     (implemented) + IStream / IDeskBarClient / IPersist (consumed). ---

    use winsrv_sys::winsrv::{DWORD_PTR, LARGE_INTEGER, NMHDR, ULARGE_INTEGER};

    pub static IID_IBandSiteHelper: IID = IID {
        Data1: 0xD1E7_AFEA, Data2: 0x6A2E, Data3: 0x11D0,
        Data4: [0x8C, 0x78, 0x00, 0xC0, 0x4F, 0xD9, 0x18, 0xB4],
    };
    pub static IID_IDeskBarClient: IID = IID {
        Data1: 0xEB0F_E175, Data2: 0x1A3A, Data3: 0x11D0,
        Data4: [0x89, 0xB3, 0x00, 0xA0, 0xC9, 0x0A, 0x90, 0xAC],
    };

    /// ITrayBandSite : IUnknown (precomp.h) — carries IBandSiteStreamCallback's
    /// OnLoad/OnSave inline + the tray methods. 9 slots.
    #[repr(C)]
    pub struct ITrayBandSiteVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut ITrayBandSite, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut ITrayBandSite) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut ITrayBandSite) -> ULONG,
        pub OnLoad: unsafe extern "system" fn(
            *mut ITrayBandSite,
            *mut IStream,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
        pub OnSave:
            unsafe extern "system" fn(*mut ITrayBandSite, *mut IUnknown, *mut IStream) -> HRESULT,
        pub IsTaskBand: unsafe extern "system" fn(*mut ITrayBandSite, *mut IUnknown) -> HRESULT,
        pub ProcessMessage: unsafe extern "system" fn(
            *mut ITrayBandSite,
            HWND,
            UINT,
            WPARAM,
            LPARAM,
            *mut LRESULT,
        ) -> HRESULT,
        pub AddContextMenus: unsafe extern "system" fn(
            *mut ITrayBandSite,
            HMENU,
            UINT,
            UINT,
            UINT,
            UINT,
            *mut *mut IContextMenu,
        ) -> HRESULT,
        pub Lock: unsafe extern "system" fn(*mut ITrayBandSite, BOOL) -> HRESULT,
    }
    #[repr(C)]
    pub struct ITrayBandSite {
        pub lpVtbl: *const ITrayBandSiteVtbl,
    }

    /// IBandSiteStreamCallback : IUnknown (shlobj_undoc.h) — 5 slots.
    #[repr(C)]
    pub struct IBandSiteStreamCallbackVtbl {
        pub QueryInterface: unsafe extern "system" fn(
            *mut IBandSiteStreamCallback,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IBandSiteStreamCallback) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IBandSiteStreamCallback) -> ULONG,
        pub OnLoad: unsafe extern "system" fn(
            *mut IBandSiteStreamCallback,
            *mut IStream,
            REFIID,
            *mut *mut c_void,
        ) -> HRESULT,
        pub OnSave: unsafe extern "system" fn(
            *mut IBandSiteStreamCallback,
            *mut IUnknown,
            *mut IStream,
        ) -> HRESULT,
    }
    #[repr(C)]
    pub struct IBandSiteStreamCallback {
        pub lpVtbl: *const IBandSiteStreamCallbackVtbl,
    }

    /// IStream : ISequentialStream : IUnknown (objidlbase.idl) — 14 slots
    /// (Read/Seek/Release used; rest ordered for ABI). LARGE_INTEGER by value.
    #[repr(C)]
    pub struct IStreamVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IStream, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IStream) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IStream) -> ULONG,
        pub Read:
            unsafe extern "system" fn(*mut IStream, *mut c_void, ULONG, *mut ULONG) -> HRESULT,
        pub Write:
            unsafe extern "system" fn(*mut IStream, *const c_void, ULONG, *mut ULONG) -> HRESULT,
        pub Seek: unsafe extern "system" fn(
            *mut IStream,
            LARGE_INTEGER,
            DWORD,
            *mut ULARGE_INTEGER,
        ) -> HRESULT,
        pub SetSize: unsafe extern "system" fn(*mut IStream, ULARGE_INTEGER) -> HRESULT,
        pub CopyTo: unsafe extern "system" fn(
            *mut IStream,
            *mut IStream,
            ULARGE_INTEGER,
            *mut ULARGE_INTEGER,
            *mut ULARGE_INTEGER,
        ) -> HRESULT,
        pub Commit: unsafe extern "system" fn(*mut IStream, DWORD) -> HRESULT,
        pub Revert: unsafe extern "system" fn(*mut IStream) -> HRESULT,
        pub LockRegion:
            unsafe extern "system" fn(*mut IStream, ULARGE_INTEGER, ULARGE_INTEGER, DWORD) -> HRESULT,
        pub UnlockRegion:
            unsafe extern "system" fn(*mut IStream, ULARGE_INTEGER, ULARGE_INTEGER, DWORD) -> HRESULT,
        pub Stat: unsafe extern "system" fn(*mut IStream, *mut c_void, DWORD) -> HRESULT,
        pub Clone: unsafe extern "system" fn(*mut IStream, *mut *mut IStream) -> HRESULT,
    }
    #[repr(C)]
    pub struct IStream {
        pub lpVtbl: *const IStreamVtbl,
    }

    /// IDeskBarClient : IOleWindow : IUnknown (shlobj.h) — 9 slots.
    #[repr(C)]
    pub struct IDeskBarClientVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IDeskBarClient, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IDeskBarClient) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IDeskBarClient) -> ULONG,
        pub GetWindow: unsafe extern "system" fn(*mut IDeskBarClient, *mut HWND) -> HRESULT,
        pub ContextSensitiveHelp: unsafe extern "system" fn(*mut IDeskBarClient, BOOL) -> HRESULT,
        pub SetDeskBarSite: unsafe extern "system" fn(*mut IDeskBarClient, *mut IUnknown) -> HRESULT,
        pub SetModeDBC: unsafe extern "system" fn(*mut IDeskBarClient, DWORD) -> HRESULT,
        pub UIActivateDBC: unsafe extern "system" fn(*mut IDeskBarClient, DWORD) -> HRESULT,
        pub GetSize: unsafe extern "system" fn(*mut IDeskBarClient, DWORD, *mut RECT) -> HRESULT,
    }
    #[repr(C)]
    pub struct IDeskBarClient {
        pub lpVtbl: *const IDeskBarClientVtbl,
    }

    /// IPersist : IUnknown (objidl.idl) — 4 slots (GetClassID used).
    #[repr(C)]
    pub struct IPersistVtbl {
        pub QueryInterface:
            unsafe extern "system" fn(*mut IPersist, REFIID, *mut *mut c_void) -> HRESULT,
        pub AddRef: unsafe extern "system" fn(*mut IPersist) -> ULONG,
        pub Release: unsafe extern "system" fn(*mut IPersist) -> ULONG,
        pub GetClassID:
            unsafe extern "system" fn(*mut IPersist, *mut winsrv_sys::winsrv::CLSID) -> HRESULT,
    }
    #[repr(C)]
    pub struct IPersist {
        pub lpVtbl: *const IPersistVtbl,
    }

    /// VARIANT (oleauto) — only vt + bstrVal used; bstrVal at off 8.
    #[repr(C)]
    pub struct VARIANT {
        pub vt: u16,
        pub wReserved1: u16,
        pub wReserved2: u16,
        pub wReserved3: u16,
        pub bstrVal: *mut u16,
        pub _tail: *mut c_void,
    }
    /// BANDSITEINFO (shobjidl.idl).
    #[repr(C)]
    pub struct BANDSITEINFO {
        pub dwMask: DWORD,
        pub dwState: DWORD,
        pub dwStyle: DWORD,
    }
    /// NMMOUSE (commctrl.h).
    #[repr(C)]
    pub struct NMMOUSE {
        pub hdr: NMHDR,
        pub dwItemSpec: DWORD_PTR,
        pub dwItemData: DWORD_PTR,
        pub pt: winsrv_sys::winsrv::POINT,
        pub dwHitInfo: LPARAM,
    }

    // stream / band / rebar constants
    pub const STREAM_SEEK_SET: DWORD = 0;
    pub const STREAM_SEEK_CUR: DWORD = 1;
    pub const STGM_READ: DWORD = 0;
    pub const VT_BSTR: u16 = 8;
    pub const DBID_BANDINFOCHANGED: DWORD = 0;
    pub const DBID_DELAYINIT: DWORD = 4;
    pub const DBID_FINISHINIT: DWORD = 5;
    pub const DBID_SETWINDOWTHEME: DWORD = 6;
    pub const BSIM_STYLE: DWORD = 0x0000_0002;
    pub const BSIS_AUTOGRIPPER: DWORD = 0x0000_0000;
    pub const BSIS_NOGRIPPER: DWORD = 0x0000_0001;
    pub const BSIS_LOCKED: DWORD = 0x0000_0100;
    pub const BSSF_UNDELETEABLE: DWORD = 0x0000_1000;
    pub const RBHT_NOWHERE: UINT = 0x1;
    pub const RBHT_CLIENT: UINT = 0x3;
    pub const RBS_BANDBORDERS: DWORD = 0x400;
    pub const DBC_HIDE: DWORD = 0;
    pub const DBC_SHOW: DWORD = 1;
    pub const NM_NCHITTEST: UINT = 0xFFFF_FFF2; // NM_FIRST-14
    pub const RBN_MINMAX: UINT = 0xFFFF_FCAC; // RBN_FIRST-21
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
        0x8000_0002u32 as i32 as isize as HKEY;
    pub const HKEY_CURRENT_USER: winsrv_sys::winsrv::HKEY =
        0x8000_0001u32 as i32 as isize as HKEY;

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
        pub fn IUnknown_GetWindow(
            punk: *mut crate::com::IUnknown,
            phwnd: *mut winsrv_sys::winsrv::HWND,
        ) -> winsrv_sys::winsrv::HRESULT;
        pub fn SHRestricted(rest: u32) -> DWORD;
        /// shell32 (undocshell) — invoke a shell folder item's default verb.
        /// pShellFolder/pidl are opaque here (passed straight through).
        pub fn SHInvokeDefaultCommand(
            hwnd: winsrv_sys::winsrv::HWND,
            pShellFolder: *mut c_void,
            pidl: *const c_void,
        ) -> winsrv_sys::winsrv::HRESULT;
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
    /// shlobj.h `REST_HIDECLOCK` — tray clock policy (GetHideClock).
    pub const REST_HIDECLOCK: u32 = 0x4000_0070;
    // startmnusite AppendMenu policy gates (RESTRICTIONS enum, shlobj.h).
    pub const REST_NOCLOSE: u32 = 0x0000_0002;
    pub const REST_NOFIND: u32 = 0x0000_0080;
    pub const REST_NORECENTDOCSMENU: u32 = 0x4000_0003;
    pub const REST_FORCESTARTMENULOGOFF: u32 = 0x4000_0032;
    pub const REST_NOSMHELP: u32 = 0x4000_0043;
    pub const REST_NODISCONNECT: u32 = 0x4100_0001;
    pub const REST_NOSECURITY: u32 = 0x4100_0002;

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

    // shell32 PIDL + folder helpers (startctxmnu). stdcall → extern "system".
    use winsrv_sys::winsrv::{INT, LPCSTR, LPSTR};
    use crate::com::{IShellFolder, LPCITEMIDLIST, LPITEMIDLIST};
    extern "system" {
        pub fn SHCloneSpecialIDList(hwnd: HWND, csidl: INT, fCreate: BOOL) -> LPITEMIDLIST;
        pub fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
        pub fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
        pub fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
        pub fn ILFree(pidl: LPITEMIDLIST);
        pub fn SHGetDesktopFolder(ppshf: *mut *mut IShellFolder) -> HRESULT;
        pub fn SHGetFolderPathW(
            hwnd: HWND,
            nFolder: INT,
            hToken: HANDLE,
            dwFlags: DWORD,
            pszPath: LPWSTR,
        ) -> HRESULT;
        pub fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, pszPath: LPWSTR) -> BOOL;
        pub fn IsUserAnAdmin() -> BOOL;
        pub fn SHAnsiToUnicode(src: LPCSTR, dst: LPWSTR, iLen: INT) -> DWORD;
        pub fn SHUnicodeToAnsi(src: LPCWSTR, dst: LPSTR, iLen: INT) -> INT;
    }
    pub const CSIDL_STARTMENU: INT = 0x000b;
    pub const CSIDL_COMMON_STARTMENU: INT = 0x0016;
    pub const SHGFP_TYPE_CURRENT: DWORD = 0;
    pub const REST_NOCOMMONGROUPS: u32 = 0x0040_0000;
    pub const IDS_PROPERTIES: UINT = 720;
    pub const IDS_OPEN_ALL_USERS: UINT = 718;
    pub const IDS_EXPLORE_ALL_USERS: UINT = 719;

    // tray band site (tbsite): ole32 / oleaut32 / shlwapi. stdcall.
    extern "system" {
        pub fn OleLoadFromStream(
            pStm: *mut crate::com::IStream,
            riid: crate::com::REFIID,
            ppvObj: *mut *mut c_void,
        ) -> winsrv_sys::winsrv::HRESULT;
        pub fn SysAllocString(psz: *const u16) -> *mut u16;
        pub fn SysFreeString(bstr: *mut u16);
        pub fn SHOpenRegStreamW(
            hkey: winsrv_sys::winsrv::HKEY,
            pszSubkey: LPCWSTR,
            pszValue: LPCWSTR,
            grfMode: DWORD,
        ) -> *mut crate::com::IStream;
        pub fn SHSetWindowBits(
            hWnd: winsrv_sys::winsrv::HWND,
            iType: i32,
            dwMask: DWORD,
            dwFlags: DWORD,
        ) -> DWORD;
        pub fn IUnknown_Exec(
            punk: *mut crate::com::IUnknown,
            pguidCmdGroup: *const winsrv_sys::winsrv::GUID,
            nCmdID: DWORD,
            nCmdexecopt: DWORD,
            pvaIn: *mut c_void,
            pvaOut: *mut c_void,
        ) -> winsrv_sys::winsrv::HRESULT;
        pub fn IUnknown_SetOwner(
            punk: *mut crate::com::IUnknown,
            pUnk: *mut crate::com::IUnknown,
        ) -> winsrv_sys::winsrv::HRESULT;
    }
    /// shlobj.h RESTRICTIONS — deny rebar min/max ("moving band").
    pub const REST_NOMOVINGBAND: u32 = 0x4000_0017;

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
