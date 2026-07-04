//! tbsite.cpp — `CTrayBandSite`, the tray band site COM object (the "outer"
//! object that wraps the shell's RebarBandSite and hosts the internal task
//! band). It cannot use ATL because ReactOS' ATL lacks full aggregation
//! support, so the whole IUnknown/QI is hand-rolled — which makes it a clean
//! mirror of the multi-vtable C-COM shape (see taskband.rs for the template).
//!
//! The C++ class `CTrayBandSite : ITrayBandSite, IBandSite,
//! IBandSiteStreamCallback` exposes only THREE QI-reachable vtables:
//!   * ITrayBandSite            — handed out by the factory (first base, off 0);
//!                                also carries the IBandSiteStreamCallback
//!                                OnLoad/OnSave slots inline (combined iface).
//!   * IBandSite                — QI(IID_IBandSite).
//!   * IBandSiteStreamCallback  — QI(IID_IUnknown) and QI(IID_IBandSiteHelper)
//!                                (the shell queries the undocumented helper IID
//!                                while (de)serialising the band stream).
//! QI(IID_IWinEventHandler) is explicitly refused (E_NOINTERFACE); anything
//! else is forwarded to the inner RebarBandSite (`m_Inner`).
//!
//! Everything else the class does — the IBandSite forwards, the tray-specific
//! ProcessMessage/AddContextMenus/Lock, and the private virtuals (HasTaskBand,
//! Update, BroadcastOleCommandExec, FinishInit, Show, Load*, _Init) — is
//! internal glue that delegates to `m_Inner`/`m_BandSite`/`m_TaskBand`.
//!
//! Object base recovery is CONTAINING_RECORD from each interface vtbl field,
//! exactly like taskband.rs.

use core::ffi::c_void;
use core::mem::{offset_of, size_of, zeroed};
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::server::{interlocked_decrement, interlocked_increment};
use winsrv_sys::consts::*;
use winsrv_sys::winsrv::*;
// IStream exists in BOTH explorer_sys::com and winsrv bindgen — the explicit
// import disambiguates the two globs in favour of our hand-rolled COM one.
use explorer_sys::com::IStream;

// CLSID_ITaskBand lives in taskband.rs (this crate) as a #[no_mangle] pub
// static; reference it directly to intercept the task-band CLSID in the stream.
use crate::explorer::taskband::CLSID_ITaskBand;
// IsSameObject (util.rs) + the RebarBandSite/BandSiteMenu factories (rshell.rs)
// are already-translated siblings in this crate — bind them directly.
use crate::explorer::rshell::{_CBandSite_CreateInstance, _CBandSiteMenu_CreateInstance};
use crate::explorer::util::IsSameObject;

#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}

#[inline]
unsafe fn guid_eq(a: *const GUID, b: *const GUID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

// =====================================================================
// Object layout — one vtbl ptr per QI-reachable interface, then the
// refcount, then the CComPtr members + plain fields.
// =====================================================================

#[repr(C)]
pub struct CTrayBandSite {
    pub lpVtbl_TrayBandSite: *const ITrayBandSiteVtbl, // off 0 (factory hands this out)
    pub lpVtbl_BandSite: *const IBandSiteVtbl,
    pub lpVtbl_BandSiteStreamCallback: *const IBandSiteStreamCallbackVtbl,
    pub ref_: LONG,
    pub m_Tray: *mut ITrayWindow,           // CComPtr
    pub m_Inner: *mut IUnknown,             // CComPtr (the RebarBandSite)
    pub m_BandSite: *mut IBandSite,         // CComPtr
    pub m_TaskBand: *mut IDeskBand,         // CComPtr
    pub m_WindowEventHandler: *mut IWinEventHandler, // CComPtr
    pub m_ContextMenu: *mut IContextMenu,   // CComPtr
    pub m_Rebar: HWND,
    pub dwFlags: DWORD, // union { DWORD dwFlags; DWORD Locked:1; } — Locked is bit 0
}

// ---- object base recovery (CONTAINING_RECORD from each interface field) ----

macro_rules! from_iface {
    ($fn:ident, $iface:ty, $field:ident) => {
        #[inline]
        unsafe fn $fn(this: *mut $iface) -> *mut CTrayBandSite {
            (this as usize - offset_of!(CTrayBandSite, $field)) as *mut CTrayBandSite
        }
    };
}
from_iface!(from_tbs, ITrayBandSite, lpVtbl_TrayBandSite);
from_iface!(from_bs, IBandSite, lpVtbl_BandSite);
from_iface!(from_bssc, IBandSiteStreamCallback, lpVtbl_BandSiteStreamCallback);

// =====================================================================
// Shared IUnknown (the hand-rolled QueryInterface / AddRef / Release).
// =====================================================================

unsafe fn tbsite_qi(this: *mut CTrayBandSite, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IBandSiteHelper) {
        // return IBandSiteStreamCallback's IUnknown
        *ppv = core::ptr::addr_of_mut!((*this).lpVtbl_BandSiteStreamCallback) as *mut c_void;
    } else if guid_eq(riid, &IID_IBandSite) {
        *ppv = core::ptr::addr_of_mut!((*this).lpVtbl_BandSite) as *mut c_void;
    } else if guid_eq(riid, &IID_IWinEventHandler) {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    } else if !(*this).m_Inner.is_null() {
        return (*(*this).m_Inner).QueryInterface(riid, ppv);
    } else {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    tbsite_addref(this);
    S_OK
}

unsafe fn tbsite_addref(this: *mut CTrayBandSite) -> ULONG {
    interlocked_increment(&mut (*this).ref_) as ULONG
}

unsafe fn tbsite_release(this: *mut CTrayBandSite) -> ULONG {
    let ret = interlocked_decrement(&mut (*this).ref_) as ULONG;
    if ret == 0 {
        // ~CTrayBandSite + CComPtr member destruction (order irrelevant).
        if !(*this).m_ContextMenu.is_null() {
            ((*(*(*this).m_ContextMenu).lpVtbl).Release)((*this).m_ContextMenu);
        }
        if !(*this).m_WindowEventHandler.is_null() {
            ((*(*(*this).m_WindowEventHandler).lpVtbl).Release)((*this).m_WindowEventHandler);
        }
        if !(*this).m_TaskBand.is_null() {
            ((*(*(*this).m_TaskBand).lpVtbl).Release)((*this).m_TaskBand);
        }
        if !(*this).m_BandSite.is_null() {
            ((*(*(*this).m_BandSite).lpVtbl).Release)((*this).m_BandSite);
        }
        if !(*this).m_Inner.is_null() {
            (*(*this).m_Inner).Release();
        }
        if !(*this).m_Tray.is_null() {
            ((*(*(*this).m_Tray).lpVtbl).Release)((*this).m_Tray);
        }
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    ret
}

// ---- per-interface IUnknown trampolines (3 ifaces × QI/AddRef/Release) ----

macro_rules! iunknown_slots {
    ($qi:ident, $ar:ident, $rel:ident, $iface:ty, $recover:ident) => {
        unsafe extern "system" fn $qi(
            this: *mut $iface,
            riid: REFIID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            tbsite_qi($recover(this), riid, ppv)
        }
        unsafe extern "system" fn $ar(this: *mut $iface) -> ULONG {
            tbsite_addref($recover(this))
        }
        unsafe extern "system" fn $rel(this: *mut $iface) -> ULONG {
            tbsite_release($recover(this))
        }
    };
}
iunknown_slots!(tbs_QI, tbs_AddRef, tbs_Release, ITrayBandSite, from_tbs);
iunknown_slots!(bs_QI, bs_AddRef, bs_Release, IBandSite, from_bs);
iunknown_slots!(bssc_QI, bssc_AddRef, bssc_Release, IBandSiteStreamCallback, from_bssc);

// =====================================================================
// IBandSiteStreamCallback::OnLoad / OnSave — shared impls (these slots
// appear in BOTH ITrayBandSite and IBandSiteStreamCallback, so a single
// C++ method override drives two vtable entries).
// =====================================================================

unsafe fn tbsite_OnLoad(
    obj: *mut CTrayBandSite,
    pStm: *mut IStream,
    riid: REFIID,
    pvObj: *mut *mut c_void,
) -> HRESULT {
    // NOTE: riid always points to IID_IUnknown here — the shell hasn't read
    // anything yet, so we read the CLSID from the stream ourselves and
    // intercept the (internal) task band before the shell tries to
    // CoCreateInstance it (which would fail).
    let mut li_pos_zero: LARGE_INTEGER = zeroed(); // ZeroMemory(&liPosZero, ...)
    let mut li_current: ULARGE_INTEGER = zeroed();
    let mut clsid: CLSID = zeroed();
    let mut ul_read: ULONG = 0;

    // Remember the current position so we can rewind if it's not our band.
    let mut h_ret = ((*(*pStm).lpVtbl).Seek)(pStm, li_pos_zero, STREAM_SEEK_CUR, &mut li_current);

    if SUCCEEDED(h_ret) {
        h_ret = ((*(*pStm).lpVtbl).Read)(
            pStm,
            &mut clsid as *mut CLSID as *mut c_void,
            size_of::<CLSID>() as ULONG,
            &mut ul_read,
        );

        if SUCCEEDED(h_ret) && ul_read == size_of::<CLSID>() as ULONG {
            if guid_eq(&clsid, &CLSID_ITaskBand) {
                // ASSERT(m_TaskBand != NULL);
                // It's the task band — create it from our cached instance.
                h_ret = ((*(*(*obj).m_TaskBand).lpVtbl).QueryInterface)(
                    (*obj).m_TaskBand,
                    riid,
                    pvObj,
                );
                return h_ret;
            }
        }
    }

    // Reset the position and let the shell do the rest.
    li_pos_zero.QuadPart = li_current.QuadPart as LONGLONG; // *(LARGE_INTEGER*)&liCurrent
    h_ret = ((*(*pStm).lpVtbl).Seek)(
        pStm,
        li_pos_zero,
        STREAM_SEEK_SET,
        core::ptr::null_mut(),
    );
    if SUCCEEDED(h_ret) {
        h_ret = OleLoadFromStream(pStm, riid, pvObj);
    }

    h_ret
}

unsafe fn tbsite_OnSave(
    _obj: *mut CTrayBandSite,
    _pUnk: *mut IUnknown,
    _pStm: *mut IStream,
) -> HRESULT {
    // FIXME: Implement
    E_NOTIMPL
}

// ITrayBandSite forwarders for the shared OnLoad/OnSave
unsafe extern "system" fn tbs_OnLoad(
    this: *mut ITrayBandSite,
    pStm: *mut IStream,
    riid: REFIID,
    pvObj: *mut *mut c_void,
) -> HRESULT {
    tbsite_OnLoad(from_tbs(this), pStm, riid, pvObj)
}
unsafe extern "system" fn tbs_OnSave(
    this: *mut ITrayBandSite,
    pUnk: *mut IUnknown,
    pStm: *mut IStream,
) -> HRESULT {
    tbsite_OnSave(from_tbs(this), pUnk, pStm)
}

// IBandSiteStreamCallback forwarders for the shared OnLoad/OnSave
unsafe extern "system" fn bssc_OnLoad(
    this: *mut IBandSiteStreamCallback,
    pStm: *mut IStream,
    riid: REFIID,
    pvObj: *mut *mut c_void,
) -> HRESULT {
    tbsite_OnLoad(from_bssc(this), pStm, riid, pvObj)
}
unsafe extern "system" fn bssc_OnSave(
    this: *mut IBandSiteStreamCallback,
    pUnk: *mut IUnknown,
    pStm: *mut IStream,
) -> HRESULT {
    tbsite_OnSave(from_bssc(this), pUnk, pStm)
}

// =====================================================================
// ITrayBandSite-specific methods (IsTaskBand / ProcessMessage /
// AddContextMenus / Lock).
// =====================================================================

unsafe extern "system" fn tbs_IsTaskBand(this: *mut ITrayBandSite, punk: *mut IUnknown) -> HRESULT {
    let obj = from_tbs(this);
    IsSameObject((*obj).m_BandSite as *mut IUnknown, punk)
}

unsafe extern "system" fn tbs_ProcessMessage(
    this: *mut ITrayBandSite,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    plResult: *mut LRESULT,
) -> HRESULT {
    let obj = from_tbs(this);

    // ASSERT(m_Rebar != NULL);

    // Custom task band behavior
    if uMsg == WM_NOTIFY {
        let nmh = lParam as *const NMHDR;

        if (*nmh).hwndFrom == (*obj).m_Rebar {
            match (*nmh).code {
                NM_NCHITTEST => {
                    let nmm = lParam as *const NMMOUSE;

                    if (*nmm).dwHitInfo == RBHT_CLIENT as LPARAM
                        || (*nmm).dwHitInfo == RBHT_NOWHERE as LPARAM
                        || (*nmm).dwItemSpec == usize::MAX as DWORD_PTR
                    {
                        // Make the rebar appear transparent so the user can
                        // drag the tray window.
                        *plResult = HTTRANSPARENT as LRESULT;
                    }
                    return S_OK;
                }
                RBN_MINMAX => {
                    // Deny if an Administrator disabled this "feature".
                    *plResult = (SHRestricted(REST_NOMOVINGBAND) != 0) as LRESULT;
                    return S_OK;
                }
                _ => {}
            }
        }
    }

    // Forward to the shell's IWinEventHandler for the default behavior.
    if (*obj).m_WindowEventHandler.is_null() {
        return E_FAIL;
    }

    ((*(*(*obj).m_WindowEventHandler).lpVtbl).OnWinEvent)(
        (*obj).m_WindowEventHandler,
        hWnd,
        uMsg,
        wParam,
        lParam,
        plResult,
    )
}

unsafe extern "system" fn tbs_AddContextMenus(
    this: *mut ITrayBandSite,
    hmenu: HMENU,
    indexMenu: UINT,
    idCmdFirst: UINT,
    idCmdLast: UINT,
    uFlags: UINT,
    ppcm: *mut *mut IContextMenu,
) -> HRESULT {
    let obj = from_tbs(this);
    let mut h_ret;

    if (*obj).m_ContextMenu.is_null() {
        // Cache the context menu so we don't CoCreateInstance every time.
        h_ret = _CBandSiteMenu_CreateInstance(
            &IID_IContextMenu,
            &mut (*obj).m_ContextMenu as *mut *mut IContextMenu as *mut *mut c_void,
        );
        if FAILED_UNEXPECTEDLY(h_ret) != 0 {
            return h_ret;
        }

        h_ret = IUnknown_SetOwner(
            (*obj).m_ContextMenu as *mut IUnknown,
            core::ptr::addr_of_mut!((*obj).lpVtbl_BandSite) as *mut IUnknown, // (IBandSite*)this
        );
        if FAILED_UNEXPECTEDLY(h_ret) != 0 {
            return h_ret;
        }
    }

    if !ppcm.is_null() {
        *ppcm = (*obj).m_ContextMenu;
        ((*(**ppcm).lpVtbl).AddRef)(*ppcm);
    }

    // Add the menu items.
    h_ret = ((*(*(*obj).m_ContextMenu).lpVtbl).QueryContextMenu)(
        (*obj).m_ContextMenu,
        hmenu,
        indexMenu,
        idCmdFirst,
        idCmdLast,
        uFlags,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return h_ret;
    }

    S_OK
}

unsafe extern "system" fn tbs_Lock(this: *mut ITrayBandSite, bLock: BOOL) -> HRESULT {
    let obj = from_tbs(this);

    // Locked bitfield == bit 0 of dwFlags.
    let b_prev_locked: BOOL = ((*obj).dwFlags & 1) as BOOL;

    // ASSERT(m_BandSite != NULL);

    if b_prev_locked != bLock {
        // Locked = bLock (single-bit store)
        if bLock != 0 {
            (*obj).dwFlags |= 1;
        } else {
            (*obj).dwFlags &= !1;
        }
        let locked = (*obj).dwFlags & 1;

        let mut bsi: BANDSITEINFO = zeroed();
        bsi.dwMask = BSIM_STYLE;
        bsi.dwStyle = if locked != 0 {
            BSIS_LOCKED | BSIS_NOGRIPPER
        } else {
            BSIS_AUTOGRIPPER
        };

        let mut h_ret = ((*(*(*obj).m_BandSite).lpVtbl).SetBandSiteInfo)(
            (*obj).m_BandSite,
            &bsi as *const BANDSITEINFO as *const c_void,
        );
        if SUCCEEDED(h_ret) {
            h_ret = tbsite_Update(obj);
        }

        return h_ret;
    }

    S_FALSE
}

// =====================================================================
// IBandSite forwards — every slot delegates to m_BandSite (the inner
// RebarBandSite), with two of them adding tray-specific behavior.
// =====================================================================

unsafe extern "system" fn bs_AddBand(this: *mut IBandSite, punk: *mut IUnknown) -> HRESULT {
    let obj = from_bs(this);

    // Send DBID_DELAYINIT to initialize the band being added.
    // FIXME: Should be delayed.
    IUnknown_Exec(
        punk,
        &IID_IDeskBand as *const IID as *const GUID,
        DBID_DELAYINIT,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let hr = ((*(*(*obj).m_BandSite).lpVtbl).AddBand)((*obj).m_BandSite, punk);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return hr;
    }

    let mut v_theme_name: VARIANT = zeroed();
    v_theme_name.vt = VT_BSTR;
    v_theme_name.bstrVal = SysAllocString(winsrv_sys::w!("TaskBar").as_ptr());
    IUnknown_Exec(
        punk,
        &IID_IDeskBand as *const IID as *const GUID,
        DBID_SETWINDOWTHEME,
        0,
        &mut v_theme_name as *mut VARIANT as *mut c_void,
        core::ptr::null_mut(),
    );

    SysFreeString(v_theme_name.bstrVal);

    S_OK
}

unsafe extern "system" fn bs_EnumBands(
    this: *mut IBandSite,
    uBand: UINT,
    pdwBandID: *mut DWORD,
) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).EnumBands)((*obj).m_BandSite, uBand, pdwBandID)
}

unsafe extern "system" fn bs_QueryBand(
    this: *mut IBandSite,
    dwBandID: DWORD,
    ppstb: *mut *mut c_void,
    pdwState: *mut DWORD,
    pszName: LPWSTR,
    cchName: i32,
) -> HRESULT {
    let obj = from_bs(this);
    let mut pstb: *mut IDeskBand = core::ptr::null_mut();

    let mut h_ret = ((*(*(*obj).m_BandSite).lpVtbl).QueryBand)(
        (*obj).m_BandSite,
        dwBandID,
        &mut pstb as *mut *mut IDeskBand as *mut *mut c_void,
        pdwState,
        pszName,
        cchName,
    );

    if SUCCEEDED(h_ret) {
        h_ret = IsSameObject(pstb as *mut IUnknown, (*obj).m_TaskBand as *mut IUnknown);
        if h_ret == S_OK {
            // The task bar band shouldn't be deletable.
            if !pdwState.is_null() {
                *pdwState |= BSSF_UNDELETEABLE;
            }
        } else if !SUCCEEDED(h_ret) {
            ((*(*pstb).lpVtbl).Release)(pstb);
            pstb = core::ptr::null_mut();
        }

        if !ppstb.is_null() {
            *ppstb = pstb as *mut c_void;
        }
    } else if !ppstb.is_null() {
        *ppstb = core::ptr::null_mut();
    }

    h_ret
}

unsafe extern "system" fn bs_SetBandState(
    this: *mut IBandSite,
    dwBandID: DWORD,
    dwMask: DWORD,
    dwState: DWORD,
) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).SetBandState)((*obj).m_BandSite, dwBandID, dwMask, dwState)
}

unsafe extern "system" fn bs_RemoveBand(this: *mut IBandSite, dwBandID: DWORD) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).RemoveBand)((*obj).m_BandSite, dwBandID)
}

unsafe extern "system" fn bs_GetBandObject(
    this: *mut IBandSite,
    dwBandID: DWORD,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).GetBandObject)((*obj).m_BandSite, dwBandID, riid, ppv)
}

unsafe extern "system" fn bs_SetBandSiteInfo(
    this: *mut IBandSite,
    pbsinfo: *const c_void,
) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).SetBandSiteInfo)((*obj).m_BandSite, pbsinfo)
}

unsafe extern "system" fn bs_GetBandSiteInfo(
    this: *mut IBandSite,
    pbsinfo: *mut c_void,
) -> HRESULT {
    let obj = from_bs(this);
    ((*(*(*obj).m_BandSite).lpVtbl).GetBandSiteInfo)((*obj).m_BandSite, pbsinfo)
}

// =====================================================================
// Private virtual helpers (never reached through a QI'd pointer — plain
// internal glue).
// =====================================================================

unsafe fn tbsite_HasTaskBand(obj: *mut CTrayBandSite) -> BOOL {
    let mut p_band: *mut IPersist = core::ptr::null_mut();
    let mut band_clsid: CLSID = zeroed();
    let mut dw_band_id: DWORD = 0;
    let mut u_band: UINT = 0;

    // Enumerate all bands.
    while SUCCEEDED(((*(*(*obj).m_BandSite).lpVtbl).EnumBands)(
        (*obj).m_BandSite,
        u_band,
        &mut dw_band_id,
    )) {
        if !p_band.is_null() {
            ((*(*p_band).lpVtbl).Release)(p_band);
            p_band = core::ptr::null_mut();
        }
        if dw_band_id != 0
            && SUCCEEDED(((*(*(*obj).m_BandSite).lpVtbl).GetBandObject)(
                (*obj).m_BandSite,
                dw_band_id,
                &IID_IPersist,
                &mut p_band as *mut *mut IPersist as *mut *mut c_void,
            ))
        {
            if SUCCEEDED(((*(*p_band).lpVtbl).GetClassID)(p_band, &mut band_clsid)) {
                if guid_eq(&band_clsid, &CLSID_ITaskBand) {
                    ((*(*p_band).lpVtbl).Release)(p_band);
                    return TRUE as BOOL;
                }
            }
        }
        u_band += 1;
    }

    if !p_band.is_null() {
        ((*(*p_band).lpVtbl).Release)(p_band);
    }

    FALSE as BOOL
}

unsafe fn tbsite_Update(obj: *mut CTrayBandSite) -> HRESULT {
    IUnknown_Exec(
        (*obj).m_Inner,
        &IID_IDeskBand as *const IID as *const GUID,
        DBID_BANDINFOCHANGED,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

unsafe fn tbsite_BroadcastOleCommandExec(
    obj: *mut CTrayBandSite,
    pguidCmdGroup: *const GUID,
    nCmdID: DWORD,
    nCmdExecOpt: DWORD,
    pvaIn: *mut c_void,
    pvaOut: *mut c_void,
) {
    let mut dw_band_id: DWORD = 0;
    let mut u_band: UINT = 0;

    // Enumerate all bands.
    while SUCCEEDED(((*(*(*obj).m_BandSite).lpVtbl).EnumBands)(
        (*obj).m_BandSite,
        u_band,
        &mut dw_band_id,
    )) {
        let mut p_oct: *mut IOleCommandTarget = core::ptr::null_mut();
        if SUCCEEDED(((*(*(*obj).m_BandSite).lpVtbl).GetBandObject)(
            (*obj).m_BandSite,
            dw_band_id,
            &IID_IOleCommandTarget,
            &mut p_oct as *mut *mut IOleCommandTarget as *mut *mut c_void,
        )) {
            // Execute the command.
            ((*(*p_oct).lpVtbl).Exec)(p_oct, pguidCmdGroup, nCmdID, nCmdExecOpt, pvaIn, pvaOut);
            ((*(*p_oct).lpVtbl).Release)(p_oct);
        }

        u_band += 1;
    }
}

unsafe fn tbsite_FinishInit(obj: *mut CTrayBandSite) -> HRESULT {
    // Broadcast the DBID_FINISHINIT command.
    tbsite_BroadcastOleCommandExec(
        obj,
        &IID_IDeskBand as *const IID as *const GUID,
        DBID_FINISHINIT,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    S_OK
}

unsafe fn tbsite_Show(obj: *mut CTrayBandSite, bShow: BOOL) -> HRESULT {
    let mut p_dbc: *mut IDeskBarClient = core::ptr::null_mut();

    let mut h_ret = ((*(*(*obj).m_BandSite).lpVtbl).QueryInterface)(
        (*obj).m_BandSite,
        &IID_IDeskBarClient,
        &mut p_dbc as *mut *mut IDeskBarClient as *mut *mut c_void,
    );
    if SUCCEEDED(h_ret) {
        h_ret = ((*(*p_dbc).lpVtbl).UIActivateDBC)(
            p_dbc,
            if bShow != 0 { DBC_SHOW } else { DBC_HIDE },
        );
        ((*(*p_dbc).lpVtbl).Release)(p_dbc);
    }

    h_ret
}

unsafe fn tbsite_LoadFromStream(obj: *mut CTrayBandSite, pStm: *mut IStream) -> HRESULT {
    let mut p_pstm: *mut IPersistStream = core::ptr::null_mut();

    // ASSERT(m_BandSite != NULL);

    // Implement the undocumented IBandSiteStreamCallback so the shell queries
    // us and we can custom-load the (internal) task band by its CLSID.
    let mut h_ret = ((*(*(*obj).m_BandSite).lpVtbl).QueryInterface)(
        (*obj).m_BandSite,
        &IID_IPersistStream,
        &mut p_pstm as *mut *mut IPersistStream as *mut *mut c_void,
    );
    if SUCCEEDED(h_ret) {
        h_ret = ((*(*p_pstm).lpVtbl).Load)(p_pstm, pStm as *mut c_void);
        ((*(*p_pstm).lpVtbl).Release)(p_pstm);
    }

    h_ret
}

unsafe fn tbsite_GetUserBandsStream(_obj: *mut CTrayBandSite, grfMode: DWORD) -> *mut IStream {
    let mut hk_streams: HKEY = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();

    if RegCreateKeyW(hkExplorer, winsrv_sys::w!("Streams").as_ptr(), &mut hk_streams)
        == ERROR_SUCCESS as LSTATUS
    {
        stream = SHOpenRegStreamW(
            hk_streams,
            winsrv_sys::w!("Desktop").as_ptr(),
            winsrv_sys::w!("TaskbarWinXP").as_ptr(),
            grfMode,
        );

        RegCloseKey(hk_streams);
    }

    stream
}

unsafe fn tbsite_GetDefaultBandsStream(_obj: *mut CTrayBandSite, grfMode: DWORD) -> *mut IStream {
    let mut hk_streams: HKEY = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();

    if RegCreateKeyW(
        HKEY_LOCAL_MACHINE,
        winsrv_sys::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Streams").as_ptr(),
        &mut hk_streams,
    ) == ERROR_SUCCESS as LSTATUS
    {
        stream = SHOpenRegStreamW(
            hk_streams,
            winsrv_sys::w!("Desktop").as_ptr(),
            winsrv_sys::w!("Default Taskbar").as_ptr(),
            grfMode,
        );

        RegCloseKey(hk_streams);
    }

    stream
}

unsafe fn tbsite_Load(obj: *mut CTrayBandSite) -> HRESULT {
    let mut h_ret;

    // Try to load the user's settings.
    let mut p_stm = tbsite_GetUserBandsStream(obj, STGM_READ);
    if !p_stm.is_null() {
        h_ret = tbsite_LoadFromStream(obj, p_stm);
        ((*(*p_stm).lpVtbl).Release)(p_stm);
    } else {
        h_ret = E_FAIL;
    }

    // If the user's settings couldn't be loaded, fall back to defaults
    // (e.g. first logon).
    if !SUCCEEDED(h_ret) {
        p_stm = tbsite_GetDefaultBandsStream(obj, STGM_READ);
        if !p_stm.is_null() {
            h_ret = tbsite_LoadFromStream(obj, p_stm);
            ((*(*p_stm).lpVtbl).Release)(p_stm);
        } else {
            h_ret = E_FAIL;
        }
    }

    h_ret
}

unsafe fn tbsite_Init(
    obj: *mut CTrayBandSite,
    tray: *mut ITrayWindow,
    pTaskBand: *mut IDeskBand,
) -> HRESULT {
    let mut p_dbc: *mut IDeskBarClient = core::ptr::null_mut();

    // m_Tray = tray (CComPtr AddRef)
    if !tray.is_null() {
        ((*(*tray).lpVtbl).AddRef)(tray);
    }
    (*obj).m_Tray = tray;
    // m_TaskBand = pTaskBand (CComPtr AddRef)
    if !pTaskBand.is_null() {
        ((*(*pTaskBand).lpVtbl).AddRef)(pTaskBand);
    }
    (*obj).m_TaskBand = pTaskBand;

    // Create the RebarBandSite (aggregated inner object).
    let mut h_ret = _CBandSite_CreateInstance(
        core::ptr::addr_of_mut!((*obj).lpVtbl_BandSite) as *mut IUnknown, // (IBandSite*)this
        &IID_IUnknown,
        &mut (*obj).m_Inner as *mut *mut IUnknown as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return h_ret;
    }

    h_ret = (*(*obj).m_Inner).QueryInterface(
        &IID_IBandSite,
        &mut (*obj).m_BandSite as *mut *mut IBandSite as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return h_ret;
    }

    h_ret = (*(*obj).m_Inner).QueryInterface(
        &IID_IWinEventHandler,
        &mut (*obj).m_WindowEventHandler as *mut *mut IWinEventHandler as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return h_ret;
    }

    h_ret = (*(*obj).m_Inner).QueryInterface(
        &IID_IDeskBarClient,
        &mut p_dbc as *mut *mut IDeskBarClient as *mut *mut c_void,
    );
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        return h_ret;
    }

    // Create the rebar in the tray.
    h_ret = ((*(*p_dbc).lpVtbl).SetDeskBarSite)(p_dbc, tray as *mut IUnknown);
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        ((*(*p_dbc).lpVtbl).Release)(p_dbc);
        return h_ret;
    }

    h_ret = ((*(*p_dbc).lpVtbl).GetWindow)(p_dbc, &mut (*obj).m_Rebar);
    if FAILED_UNEXPECTEDLY(h_ret) != 0 {
        ((*(*p_dbc).lpVtbl).Release)(p_dbc);
        return h_ret;
    }

    // SetWindowStyle(m_Rebar, RBS_BANDBORDERS, 0)
    SHSetWindowBits((*obj).m_Rebar, GWL_STYLE, RBS_BANDBORDERS, 0);

    // Set the Desk Bar mode to the current one.
    // FIXME: track the mode/update when the tray docks to another edge.
    let mut dw_mode: DWORD = 0;
    if ((*(*(*obj).m_Tray).lpVtbl).IsHorizontal)((*obj).m_Tray) == 0 {
        dw_mode = DBIF_VIEWMODE_VERTICAL;
    }

    ((*(*p_dbc).lpVtbl).SetModeDBC)(p_dbc, dw_mode);

    // Load the saved state of the task band site.
    // FIXME: should delay-load shell extensions (DBID_DELAYINIT).
    tbsite_Load(obj);

    // Add the task bar band if it wasn't added while loading.
    if tbsite_HasTaskBand(obj) == 0 {
        h_ret =
            ((*(*(*obj).m_BandSite).lpVtbl).AddBand)((*obj).m_BandSite, (*obj).m_TaskBand as *mut IUnknown);
        if FAILED_UNEXPECTEDLY(h_ret) != 0 {
            ((*(*p_dbc).lpVtbl).Release)(p_dbc);
            return h_ret;
        }
    }

    // Should we send this after showing it?
    tbsite_Update(obj);

    // FIXME: When should we send this?
    tbsite_FinishInit(obj);

    // Activate the band site.
    tbsite_Show(obj, TRUE as BOOL);

    // Release the local IDeskBarClient CComPtr.
    ((*(*p_dbc).lpVtbl).Release)(p_dbc);

    S_OK
}

// =====================================================================
// Static vtables.
// =====================================================================

static vt_TrayBandSite: ITrayBandSiteVtbl = ITrayBandSiteVtbl {
    QueryInterface: tbs_QI,
    AddRef: tbs_AddRef,
    Release: tbs_Release,
    OnLoad: tbs_OnLoad,
    OnSave: tbs_OnSave,
    IsTaskBand: tbs_IsTaskBand,
    ProcessMessage: tbs_ProcessMessage,
    AddContextMenus: tbs_AddContextMenus,
    Lock: tbs_Lock,
};

static vt_BandSite: IBandSiteVtbl = IBandSiteVtbl {
    QueryInterface: bs_QI,
    AddRef: bs_AddRef,
    Release: bs_Release,
    AddBand: bs_AddBand,
    EnumBands: bs_EnumBands,
    QueryBand: bs_QueryBand,
    SetBandState: bs_SetBandState,
    RemoveBand: bs_RemoveBand,
    GetBandObject: bs_GetBandObject,
    SetBandSiteInfo: bs_SetBandSiteInfo,
    GetBandSiteInfo: bs_GetBandSiteInfo,
};

static vt_BandSiteStreamCallback: IBandSiteStreamCallbackVtbl = IBandSiteStreamCallbackVtbl {
    QueryInterface: bssc_QI,
    AddRef: bssc_AddRef,
    Release: bssc_Release,
    OnLoad: bssc_OnLoad,
    OnSave: bssc_OnSave,
};

// =====================================================================
// Construction + factory.
// =====================================================================

unsafe fn CTrayBandSite_ctor(this: *mut CTrayBandSite) {
    (*this).lpVtbl_TrayBandSite = &vt_TrayBandSite;
    (*this).lpVtbl_BandSite = &vt_BandSite;
    (*this).lpVtbl_BandSiteStreamCallback = &vt_BandSiteStreamCallback;
    (*this).ref_ = 0; // m_RefCount(0)
    (*this).m_Tray = core::ptr::null_mut();
    (*this).m_Inner = core::ptr::null_mut();
    (*this).m_BandSite = core::ptr::null_mut();
    (*this).m_TaskBand = core::ptr::null_mut();
    (*this).m_WindowEventHandler = core::ptr::null_mut();
    (*this).m_ContextMenu = core::ptr::null_mut();
    (*this).m_Rebar = core::ptr::null_mut(); // m_Rebar(NULL)
    (*this).dwFlags = 0;
}

/// `CTrayBandSite_CreateInstance` — the free factory (cdecl). The precomp.h
/// declaration must be given C linkage so the C++ callers bind this symbol.
#[no_mangle]
pub unsafe extern "C" fn CTrayBandSite_CreateInstance(
    tray: *mut ITrayWindow,
    pTaskBand: *mut IDeskBand,
    pBandSite: *mut *mut ITrayBandSite,
) -> HRESULT {
    let tb = HeapAlloc(hProcessHeap, 0, size_of::<CTrayBandSite>() as SIZE_T) as *mut CTrayBandSite;
    if tb.is_null() {
        return E_FAIL;
    }
    CTrayBandSite_ctor(tb);

    tbsite_addref(tb);

    let hr = tbsite_Init(tb, tray, pTaskBand);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        tbsite_release(tb);
        return hr;
    }

    // *pBandSite = tb (implicit upcast to ITrayBandSite* = first base, off 0).
    *pBandSite = core::ptr::addr_of_mut!((*tb).lpVtbl_TrayBandSite) as *mut ITrayBandSite;

    S_OK
}
