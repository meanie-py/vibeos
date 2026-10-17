use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::typelib::sys::FindResourceW;
use crate::typelib::sys::FreeLibrary;
use crate::typelib::sys::LoadLibraryW;
use crate::typelib::sys::LoadResource;

extern "system" {
    pub fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    fn CreatePropertySheetPageW(lpPsp: *const PROPSHEETPAGEW) -> HPROPSHEETPAGE;
    fn GdiGetCharDimensions(hdc: HDC, lptm: *mut TEXTMETRICW, lpcy: *mut i32) -> i32;
    fn LockResource(hResData: HGLOBAL) -> LPVOID;
    fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
    fn PropertySheetW(lppsph: *mut PROPSHEETHEADERW) -> isize;
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_ole as *mut __wine_debug_channel };

static mut __wine_dbch_ole: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"ole\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
    }
};

// winuser.h:1767 `#define WM_INITDIALOG 272`, winuser.h:1637 `#define WM_DESTROY 2`.
// NOT pub: the prelude glob-re-exports this module, and a pub const here would collide
// with a sibling Tier-D module declaring the same name (ambiguous-glob E0659).
const WM_INITDIALOG: u32 = 272;
const WM_DESTROY: u32 = 2;
// prsht.h:40 `#define PSH_PROPTITLE 1`; prsht.h:23/26 `PSP_DLGINDIRECT 1` / `PSP_USETITLE 8`.
const PSH_PROPTITLE: DWORD = 1;
const PSP_DLGINDIRECT: DWORD = 1;
const PSP_USETITLE: DWORD = 8;

#[repr(C)]
pub struct PropertyPageSite {
    pub IPropertyPageSite_iface: IPropertyPageSite,
    pub lcid: LCID,
    pub ref_: LONG,
}
pub type PPropertyPageSite = *mut PropertyPageSite;

#[repr(C)]
struct DialogTemplate {
    template: DLGTEMPLATE,
    menu: u16,
    class: u16,
    title: u16,
}
unsafe extern "C" fn impl_from_IPropertyPageSite(
    iface: *mut IPropertyPageSite,
) -> *mut PropertyPageSite {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(PropertyPageSite, IPropertyPageSite_iface) as ULONG_PTR,
    ) as *mut PropertyPageSite;
}
stdmethod! {
fn property_sheet_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> INT_PTR {
    let mut property_page = GetWindowLongPtrW(hwnd, DWLP_USER as i32) as *mut IPropertyPage;

    match msg {
        WM_INITDIALOG => {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };

            property_page = (*(lparam as LPPROPSHEETPAGEW)).lParam as *mut IPropertyPage;

            GetClientRect(hwnd, core::ptr::addr_of_mut!(rect));
            IPropertyPage_Activate(property_page, hwnd, core::ptr::addr_of_mut!(rect) as LPCRECT, TRUE);
            IPropertyPage_Show(property_page, SW_SHOW as u32);

            SetWindowLongPtrW(hwnd, DWLP_USER as i32, property_page as LONG_PTR);
            return FALSE as INT_PTR;
        }
        WM_DESTROY => {
            IPropertyPage_Show(property_page, SW_HIDE as u32);
            IPropertyPage_Deactivate(property_page);
            return FALSE as INT_PTR;
        }
        _ => return FALSE as INT_PTR,
    }
}
}
stdmethod! {
fn PropertyPageSite_QueryInterface(
    iface: *mut IPropertyPageSite,
    riid: *const IID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepropframe.c\0"
                as *const u8 as *const i8,
            b"PropertyPageSite_QueryInterface\0" as *const u8 as *const i8,
            77i32,
            b"(%p riid: %s)\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(riid as *const _GUID),
        );
    }
    
    if IsEqualGUID(
        core::ptr::addr_of!(IID_IUnknown) as *const _GUID,
        riid as *const _GUID,
    )
        || IsEqualGUID(
            core::ptr::addr_of!(IID_IPropertyPageSite) as *const _GUID,
            riid as *const _GUID,
        )
    {
        *ppv = iface as *mut core::ffi::c_void;
    } else {
        *ppv = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    
    IUnknown_AddRef(*ppv as *mut IUnknown);
    S_OK
}
}
stdmethod! {
fn PropertyPageSite_AddRef(iface: *mut IPropertyPageSite) -> ULONG {
    let this: *mut PropertyPageSite = impl_from_IPropertyPageSite(iface);
    let ref_0: LONG = InterlockedIncrement(core::ptr::addr_of_mut!((*this).ref_));

    TRACE!("(%p) ref=%d\n", this, ref_0);
    return ref_0 as ULONG;
}
}
stdmethod! {
fn PropertyPageSite_Release(iface: *mut IPropertyPageSite) -> ULONG {
    let this: *mut PropertyPageSite = impl_from_IPropertyPageSite(iface);
    let ref_count: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*this).ref_));

    TRACE!(
        b"(%p) ref=%d\n\0".as_ptr() as *const i8,
        this,
        ref_count,
    );
    
    if ref_count == 0 {
        HeapFree(GetProcessHeap(), 0, this as LPVOID);
    }
    
    ref_count as ULONG
}
}
stdmethod! {
fn PropertyPageSite_OnStatusChange(
    iface: *mut IPropertyPageSite,
    dwFlags: DWORD,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepropframe.c\0"
                as *const u8 as *const i8,
            b"PropertyPageSite_OnStatusChange\0" as *const u8 as *const i8,
            114i32,
            b"(%p, %x)\n\0" as *const u8 as *const i8,
            iface,
            dwFlags,
        );
    }
    return S_OK;
}
}
stdmethod! {
fn PropertyPageSite_GetLocaleID(
    iface: *mut IPropertyPageSite,
    pLocaleID: *mut LCID,
) -> HRESULT {
    let this: *mut PropertyPageSite = impl_from_IPropertyPageSite(iface);
    
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const i8, iface, pLocaleID);
    
    *pLocaleID = (*this).lcid;
    S_OK
}
}
stdmethod! {
fn PropertyPageSite_GetPageContainer(
    iface: *mut IPropertyPageSite,
    ppUnk: *mut *mut IUnknown,
) -> HRESULT {
    FIXME!(b"(%p, %p)\n\0".as_ptr() as *const i8, iface, ppUnk);
    E_NOTIMPL
}
}
stdmethod! {
fn PropertyPageSite_TranslateAccelerator(
    iface: *mut IPropertyPageSite,
    pMsg: *mut MSG,
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepropframe.c\0"
                as *const u8 as *const i8,
            b"PropertyPageSite_TranslateAccelerator\0" as *const u8 as *const i8,
            138 as i32,
            b"(%p, %p)\n\0" as *const u8 as *const i8,
            iface,
            pMsg,
        );
    }
    return E_NOTIMPL;
}
}

#[no_mangle]
pub unsafe extern "system" fn OleCreatePropertyFrameIndirect(lpParams: *mut OCPFIPARAMS) -> HRESULT {
    static comctlW: [u16; 13] = [
        'c' as u16, 'o' as u16, 'm' as u16, 'c' as u16, 't' as u16, 'l' as u16,
        '3' as u16, '2' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
    ];

    let mut property_sheet: PROPSHEETHEADERW = core::mem::zeroed();
    let mut property_sheet_page: PROPSHEETPAGEW = core::mem::zeroed();
    let dialogs: *mut DialogTemplate;
    let property_page: *mut *mut IPropertyPage;
    let mut property_page_site: *mut PropertyPageSite;
    let mut res: HRESULT;
    let mut i: u32;
    let hcomctl: HMODULE;
    let mut property_sheet_dialog_find: HRSRC = core::ptr::null_mut();
    let mut property_sheet_dialog_load: HGLOBAL = core::ptr::null_mut();
    let mut property_sheet_dialog_data: *mut u16 = core::ptr::null_mut();
    let hdc: HDC;
    let mut font_desc: LOGFONTW = core::mem::zeroed();
    let mut hfont: HFONT;
    let mut font_width: i32 = 4;
    let mut font_height: i32 = 8;

    if lpParams.is_null() {
        return E_POINTER;
    }

    TRACE!(
        b"(%d %p %d %d %s %d %p %d %p %d %d)\n\0".as_ptr(),
        (*lpParams).cbStructSize,
        (*lpParams).hWndOwner,
        (*lpParams).x,
        (*lpParams).y,
        debugstr_w((*lpParams).lpszCaption),
        (*lpParams).cObjects,
        (*lpParams).lplpUnk,
        (*lpParams).cPages,
        (*lpParams).lpPages,
        (*lpParams).lcid,
        (*lpParams).dispidInitialProperty,
    );

    if (*lpParams).lpPages.is_null() {
        return E_POINTER;
    }

    if (*lpParams).cbStructSize != core::mem::size_of::<OCPFIPARAMS>() as u32 {
        WARN!(b"incorrect structure size\n\0".as_ptr());
        return E_INVALIDARG;
    }

    if (*lpParams).dispidInitialProperty != 0 {
        FIXME!(b"dispidInitialProperty not yet implemented\n\0".as_ptr());
    }

    hdc = GetDC(core::ptr::null_mut());
    hcomctl = (LoadLibraryW(comctlW.as_ptr())).cast();
    if !hcomctl.is_null() {
        property_sheet_dialog_find = (FindResourceW((hcomctl).cast(), MAKEINTRESOURCEW(1006), RT_DIALOG as LPWSTR)).cast();
    }
    if !property_sheet_dialog_find.is_null() {
        property_sheet_dialog_load = LoadResource((hcomctl).cast(), (property_sheet_dialog_find).cast());
    }
    if !property_sheet_dialog_load.is_null() {
        property_sheet_dialog_data = LockResource(property_sheet_dialog_load) as *mut u16;
    }

    if !property_sheet_dialog_data.is_null() {
        if *property_sheet_dialog_data.offset(1) == 0xffff {
            ERR!(b"Expected DLGTEMPLATE structure\n\0".as_ptr());
            FreeLibrary((hcomctl).cast());
            return E_OUTOFMEMORY;
        }

        property_sheet_dialog_data = property_sheet_dialog_data.offset(
            (core::mem::size_of::<DLGTEMPLATE>() / core::mem::size_of::<u16>()) as isize,
        );
        property_sheet_dialog_data = property_sheet_dialog_data.offset((lstrlenW(property_sheet_dialog_data) + 1) as isize);
        property_sheet_dialog_data = property_sheet_dialog_data.offset((lstrlenW(property_sheet_dialog_data) + 1) as isize);
        property_sheet_dialog_data = property_sheet_dialog_data.offset((lstrlenW(property_sheet_dialog_data) + 1) as isize);

        core::ptr::write_bytes(&mut font_desc as *mut LOGFONTW, 0, 1);
        font_desc.lfHeight = -MulDiv(
            *property_sheet_dialog_data.offset(0) as i32,
            GetDeviceCaps(hdc, LOGPIXELSY),
            72,
        );
        font_desc.lfCharSet = DEFAULT_CHARSET as u8;
        core::ptr::copy_nonoverlapping(
            property_sheet_dialog_data.offset(1),
            font_desc.lfFaceName.as_mut_ptr(),
            (lstrlenW(property_sheet_dialog_data.offset(1)) + 1) as usize,
        );
        hfont = CreateFontIndirectW(&font_desc);

        if !hfont.is_null() {
            hfont = SelectObject(hdc, hfont as HGDIOBJ) as HFONT;
            font_width = GdiGetCharDimensions(hdc, core::ptr::null_mut(), &mut font_height);
            SelectObject(hdc, hfont as HGDIOBJ);
        }
    }
    if !hcomctl.is_null() {
        FreeLibrary((hcomctl).cast());
    }
    ReleaseDC(core::ptr::null_mut(), hdc);

    core::ptr::write_bytes(&mut property_sheet as *mut PROPSHEETHEADERW, 0, 1);
    property_sheet.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
    if !(*lpParams).lpszCaption.is_null() {
        property_sheet.dwFlags = PSH_PROPTITLE;
        property_sheet.pszCaption = (*lpParams).lpszCaption;
    }

    // `u3` in the C (NONAMELESSUNION) is bindgen's `__bindgen_anon_3` union.
    property_sheet.__bindgen_anon_3.phpage = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (((*lpParams).cPages as usize) * core::mem::size_of::<HPROPSHEETPAGE>()) as SIZE_T,
    ) as *mut HPROPSHEETPAGE;
    property_page = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (((*lpParams).cPages as usize) * core::mem::size_of::<*mut IPropertyPage>()) as SIZE_T,
    ) as *mut *mut IPropertyPage;
    dialogs = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (((*lpParams).cPages as usize) * core::mem::size_of::<DialogTemplate>()) as SIZE_T,
    ) as *mut DialogTemplate;
    if property_sheet.__bindgen_anon_3.phpage.is_null()
        || property_page.is_null()
        || dialogs.is_null()
    {
        HeapFree(
            GetProcessHeap(),
            0,
            property_sheet.__bindgen_anon_3.phpage as LPVOID,
        );
        HeapFree(GetProcessHeap(), 0, property_page as LPVOID);
        HeapFree(GetProcessHeap(), 0, dialogs as LPVOID);
        return E_OUTOFMEMORY;
    }

    core::ptr::write_bytes(&mut property_sheet_page as *mut PROPSHEETPAGEW, 0, 1);
    property_sheet_page.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    property_sheet_page.dwFlags = PSP_DLGINDIRECT | PSP_USETITLE;
    property_sheet_page.pfnDlgProc = Some(property_sheet_proc);

    i = 0;
    while i < (*lpParams).cPages {
        let mut page_info: PROPPAGEINFO = core::mem::zeroed();

        res = CoCreateInstance(
            (*lpParams).lpPages.offset(i as isize),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IPropertyPage,
            property_page.offset(i as isize) as *mut LPVOID,
        );
        if !FAILED(res) {
            property_page_site = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<PropertyPageSite>() as SIZE_T,
            ) as *mut PropertyPageSite;
            if !property_page_site.is_null() {
                (*property_page_site).IPropertyPageSite_iface.lpVtbl =
                    core::ptr::addr_of!(PropertyPageSiteVtbl) as *mut IPropertyPageSiteVtbl;
                (*property_page_site).ref_ = 1;
                (*property_page_site).lcid = (*lpParams).lcid;

                res = IPropertyPage_SetPageSite(
                    *property_page.offset(i as isize),
                    &mut (*property_page_site).IPropertyPageSite_iface,
                );
                IPropertyPageSite_Release(&mut (*property_page_site).IPropertyPageSite_iface);
                if !FAILED(res) {
                    res = IPropertyPage_SetObjects(
                        *property_page.offset(i as isize),
                        (*lpParams).cObjects,
                        (*lpParams).lplpUnk as *mut *mut IUnknown,
                    );
                    if FAILED(res) {
                        WARN!(b"SetObjects() failed, hr %#x.\n\0".as_ptr(), res);
                    }

                    res = IPropertyPage_GetPageInfo(*property_page.offset(i as isize), &mut page_info);
                    if !FAILED(res) {
                        (*dialogs.offset(i as isize)).template.cx =
                            MulDiv(page_info.size.cx, 4, font_width) as i16;
                        (*dialogs.offset(i as isize)).template.cy =
                            MulDiv(page_info.size.cy, 8, font_height) as i16;

                        // C: `property_sheet_page.u.pResource` (NONAMELESSUNION) — bindgen
                        // emits this union as a struct + __BindgenUnionField accessors.
                        *property_sheet_page.__bindgen_anon_1.pResource.as_mut() =
                            core::ptr::addr_of!((*dialogs.offset(i as isize)).template);
                        property_sheet_page.lParam = *property_page.offset(i as isize) as LPARAM;
                        property_sheet_page.pszTitle = page_info.pszTitle;

                        let idx = property_sheet.nPages;
                        property_sheet.nPages = property_sheet.nPages.wrapping_add(1);
                        *property_sheet.__bindgen_anon_3.phpage.offset(idx as isize) =
                            CreatePropertySheetPageW(&property_sheet_page);
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }

    PropertySheetW(&mut property_sheet);

    i = 0;
    while i < (*lpParams).cPages {
        if !(*property_page.offset(i as isize)).is_null() {
            IPropertyPage_Release(*property_page.offset(i as isize));
        }
        i = i.wrapping_add(1);
    }

    HeapFree(GetProcessHeap(), 0, dialogs as LPVOID);
    HeapFree(GetProcessHeap(), 0, property_page as LPVOID);
    HeapFree(
        GetProcessHeap(),
        0,
        property_sheet.__bindgen_anon_3.phpage as LPVOID,
    );
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn OleCreatePropertyFrame(
    hwndOwner: HWND,
    x: UINT,
    y: UINT,
    lpszCaption: LPCOLESTR,
    cObjects: ULONG,
    ppUnk: *mut LPUNKNOWN,
    cPages: ULONG,
    pPageClsID: LPCLSID,
    lcid: LCID,
    _dwReserved: DWORD,
    _pvReserved: LPVOID,
) -> HRESULT {
    let mut ocpf = OCPFIPARAMS {
        cbStructSize: core::mem::size_of::<OCPFIPARAMS>() as ULONG,
        hWndOwner: hwndOwner,
        x: x as i32,
        y: y as i32,
        lpszCaption: lpszCaption,
        cObjects: cObjects,
        lplpUnk: ppUnk,
        cPages: cPages,
        lpPages: pPageClsID as *mut CLSID,
        lcid: lcid,
        dispidInitialProperty: 0,
    };

    OleCreatePropertyFrameIndirect(&mut ocpf)
}

// COM vtable tables — generated from olepropframe.c by vtbl_gen.py
/// olepropframe.c:142 `static const IPropertyPageSiteVtbl PropertyPageSiteVtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static PropertyPageSiteVtbl: IPropertyPageSiteVtbl = IPropertyPageSiteVtbl {
    QueryInterface: Some(PropertyPageSite_QueryInterface),
    AddRef: Some(PropertyPageSite_AddRef),
    Release: Some(PropertyPageSite_Release),
    OnStatusChange: Some(PropertyPageSite_OnStatusChange),
    GetLocaleID: Some(PropertyPageSite_GetLocaleID),
    GetPageContainer: Some(PropertyPageSite_GetPageContainer),
    TranslateAcceleratorA: Some(PropertyPageSite_TranslateAccelerator),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 1999 Corel Corporation
//  * Sean Langley
//  * Copyright 2010  Geoffrey Hausheer
//  * Copyright 2010 Piotr Caban for CodeWeavers
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
// #include <stdarg.h>
//
// #define COBJMACROS
// #define NONAMELESSUNION
//
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "ole2.h"
// #include "olectl.h"
// #include "oledlg.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(ole);
//
// typedef struct {
//     IPropertyPageSite IPropertyPageSite_iface;
//     LCID lcid;
//     LONG ref;
// } PropertyPageSite;
//
// static inline PropertyPageSite *impl_from_IPropertyPageSite(IPropertyPageSite *iface)
// {
//     return CONTAINING_RECORD(iface, PropertyPageSite, IPropertyPageSite_iface);
// }
//
// static INT_PTR __stdcall property_sheet_proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam)
// {
//     IPropertyPage *property_page = (IPropertyPage*)GetWindowLongPtrW(hwnd, DWLP_USER);
//
//     switch(msg) {
//     case WM_INITDIALOG: {
//         RECT rect;
//
//         property_page = (IPropertyPage*)((LPPROPSHEETPAGEW)lparam)->lParam;
//
//         GetClientRect(hwnd, &rect);
//         IPropertyPage_Activate(property_page, hwnd, &rect, TRUE);
//         IPropertyPage_Show(property_page, SW_SHOW);
//
//         SetWindowLongPtrW(hwnd, DWLP_USER, (LONG_PTR)property_page);
//         return FALSE;
//     }
//     case WM_DESTROY:
//         IPropertyPage_Show(property_page, SW_HIDE);
//         IPropertyPage_Deactivate(property_page);
//         return FALSE;
//     default:
//         return FALSE;
//     }
// }
//
// static HRESULT WINAPI PropertyPageSite_QueryInterface(IPropertyPageSite*  iface,
//         REFIID  riid, void**  ppv)
// {
//     TRACE("(%p riid: %s)\n",iface, debugstr_guid(riid));
//
//     if(IsEqualGUID(&IID_IUnknown, riid)
//             || IsEqualGUID(&IID_IPropertyPageSite, riid))
//         *ppv = iface;
//     else {
//         *ppv = NULL;
//         return E_NOINTERFACE;
//     }
//
//     IUnknown_AddRef((IUnknown*)*ppv);
//     return S_OK;
// }
//
// static ULONG WINAPI PropertyPageSite_AddRef(IPropertyPageSite* iface)
// {
//     PropertyPageSite *this = impl_from_IPropertyPageSite(iface);
//     LONG ref = InterlockedIncrement(&this->ref);
//
//     TRACE("(%p) ref=%d\n", this, ref);
//     return ref;
// }
//
// static ULONG WINAPI PropertyPageSite_Release(IPropertyPageSite* iface)
// {
//     PropertyPageSite *this = impl_from_IPropertyPageSite(iface);
//     LONG ref = InterlockedDecrement(&this->ref);
//
//     TRACE("(%p) ref=%d\n", this, ref);
//     if(!ref)
//         HeapFree(GetProcessHeap(), 0, this);
//     return ref;
// }
//
// static HRESULT WINAPI PropertyPageSite_OnStatusChange(
//         IPropertyPageSite *iface, DWORD dwFlags)
// {
//     TRACE("(%p, %x)\n", iface, dwFlags);
//     return S_OK;
// }
//
// static HRESULT WINAPI PropertyPageSite_GetLocaleID(
//         IPropertyPageSite *iface, LCID *pLocaleID)
// {
//     PropertyPageSite *this = impl_from_IPropertyPageSite(iface);
//
//     TRACE("(%p, %p)\n", iface, pLocaleID);
//     *pLocaleID = this->lcid;
//     return S_OK;
// }
//
// static HRESULT WINAPI PropertyPageSite_GetPageContainer(
//         IPropertyPageSite* iface, IUnknown** ppUnk)
// {
//     FIXME("(%p, %p)\n", iface, ppUnk);
//     return E_NOTIMPL;
// }
//
// static HRESULT WINAPI PropertyPageSite_TranslateAccelerator(
//         IPropertyPageSite* iface, MSG *pMsg)
// {
//     FIXME("(%p, %p)\n", iface, pMsg);
//     return E_NOTIMPL;
// }
//
// static IPropertyPageSiteVtbl PropertyPageSiteVtbl = {
//     PropertyPageSite_QueryInterface,
//     PropertyPageSite_AddRef,
//     PropertyPageSite_Release,
//     PropertyPageSite_OnStatusChange,
//     PropertyPageSite_GetLocaleID,
//     PropertyPageSite_GetPageContainer,
//     PropertyPageSite_TranslateAccelerator
// };
//
// /***********************************************************************
//  * OleCreatePropertyFrameIndirect (OLEAUT32.416)
//  */
// HRESULT WINAPI OleCreatePropertyFrameIndirect(LPOCPFIPARAMS lpParams)
// {
//     static const WCHAR comctlW[] = { 'c','o','m','c','t','l','3','2','.','d','l','l',0 };
//
//     PROPSHEETHEADERW property_sheet;
//     PROPSHEETPAGEW property_sheet_page;
//     struct {
//         DLGTEMPLATE template;
//         WORD menu;
//         WORD class;
//         WORD title;
//     } *dialogs;
//     IPropertyPage **property_page;
//     PropertyPageSite *property_page_site;
//     HRESULT res;
//     ULONG i;
//     HMODULE hcomctl;
//     HRSRC property_sheet_dialog_find = NULL;
//     HGLOBAL property_sheet_dialog_load = NULL;
//     WCHAR *property_sheet_dialog_data = NULL;
//     HDC hdc;
//     LOGFONTW font_desc;
//     HFONT hfont;
//     LONG font_width = 4, font_height = 8;
//
//     if(!lpParams)
//         return E_POINTER;
//
//     TRACE("(%d %p %d %d %s %d %p %d %p %d %d)\n", lpParams->cbStructSize,
//             lpParams->hWndOwner, lpParams->x, lpParams->y,
//             debugstr_w(lpParams->lpszCaption), lpParams->cObjects,
//             lpParams->lplpUnk, lpParams->cPages, lpParams->lpPages,
//             lpParams->lcid, lpParams->dispidInitialProperty);
//
//     if(!lpParams->lpPages)
//         return E_POINTER;
//
//     if(lpParams->cbStructSize != sizeof(OCPFIPARAMS)) {
//         WARN("incorrect structure size\n");
//         return E_INVALIDARG;
//     }
//
//     if(lpParams->dispidInitialProperty)
//         FIXME("dispidInitialProperty not yet implemented\n");
//
//     hdc = GetDC(NULL);
//     hcomctl = LoadLibraryW(comctlW);
//     if(hcomctl)
//         property_sheet_dialog_find = FindResourceW(hcomctl,
//                 MAKEINTRESOURCEW(1006 /*IDD_PROPSHEET*/), (LPWSTR)RT_DIALOG);
//     if(property_sheet_dialog_find)
//         property_sheet_dialog_load = LoadResource(hcomctl, property_sheet_dialog_find);
//     if(property_sheet_dialog_load)
//         property_sheet_dialog_data = LockResource(property_sheet_dialog_load);
//
//     if(property_sheet_dialog_data) {
//         if(property_sheet_dialog_data[1] == 0xffff) {
//             ERR("Expected DLGTEMPLATE structure\n");
//             FreeLibrary(hcomctl);
//             return E_OUTOFMEMORY;
//         }
//
//         property_sheet_dialog_data += sizeof(DLGTEMPLATE)/sizeof(WCHAR);
//         /* Skip menu, class and title */
//         property_sheet_dialog_data += lstrlenW(property_sheet_dialog_data)+1;
//         property_sheet_dialog_data += lstrlenW(property_sheet_dialog_data)+1;
//         property_sheet_dialog_data += lstrlenW(property_sheet_dialog_data)+1;
//
//         memset(&font_desc, 0, sizeof(LOGFONTW));
//         /* Calculate logical height */
//         font_desc.lfHeight = -MulDiv(property_sheet_dialog_data[0],
//                 GetDeviceCaps(hdc, LOGPIXELSY), 72);
//         font_desc.lfCharSet = DEFAULT_CHARSET;
//         memcpy(font_desc.lfFaceName, property_sheet_dialog_data+1,
//                 sizeof(WCHAR)*(lstrlenW(property_sheet_dialog_data+1)+1));
//         hfont = CreateFontIndirectW(&font_desc);
//
//         if(hfont) {
//             hfont = SelectObject(hdc, hfont);
//             font_width = GdiGetCharDimensions(hdc, NULL, &font_height);
//             SelectObject(hdc, hfont);
//         }
//     }
//     if(hcomctl)
//         FreeLibrary(hcomctl);
//     ReleaseDC(NULL, hdc);
//
//     memset(&property_sheet, 0, sizeof(property_sheet));
//     property_sheet.dwSize = sizeof(property_sheet);
//     if(lpParams->lpszCaption) {
//         property_sheet.dwFlags = PSH_PROPTITLE;
//         property_sheet.pszCaption = lpParams->lpszCaption;
//     }
//
//     property_sheet.u3.phpage = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//             lpParams->cPages*sizeof(HPROPSHEETPAGE));
//     property_page = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//             lpParams->cPages*sizeof(IPropertyPage*));
//     dialogs = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//             lpParams->cPages*sizeof(*dialogs));
//     if(!property_sheet.u3.phpage || !property_page || !dialogs) {
//         HeapFree(GetProcessHeap(), 0, property_sheet.u3.phpage);
//         HeapFree(GetProcessHeap(), 0, property_page);
//         HeapFree(GetProcessHeap(), 0, dialogs);
//         return E_OUTOFMEMORY;
//     }
//
//     memset(&property_sheet_page, 0, sizeof(PROPSHEETPAGEW));
//     property_sheet_page.dwSize = sizeof(PROPSHEETPAGEW);
//     property_sheet_page.dwFlags = PSP_DLGINDIRECT|PSP_USETITLE;
//     property_sheet_page.pfnDlgProc = property_sheet_proc;
//
//     for(i=0; i<lpParams->cPages; i++) {
//         PROPPAGEINFO page_info;
//
//         res = CoCreateInstance(&lpParams->lpPages[i], NULL, CLSCTX_INPROC_SERVER,
//                 &IID_IPropertyPage, (void**)&property_page[i]);
//         if(FAILED(res))
//             continue;
//
//         property_page_site = HeapAlloc(GetProcessHeap(), 0, sizeof(PropertyPageSite));
//         if(!property_page_site)
//             continue;
//         property_page_site->IPropertyPageSite_iface.lpVtbl = &PropertyPageSiteVtbl;
//         property_page_site->ref = 1;
//         property_page_site->lcid = lpParams->lcid;
//
//         res = IPropertyPage_SetPageSite(property_page[i],
//                 &property_page_site->IPropertyPageSite_iface);
//         IPropertyPageSite_Release(&property_page_site->IPropertyPageSite_iface);
//         if(FAILED(res))
//             continue;
//
//         res = IPropertyPage_SetObjects(property_page[i],
//                 lpParams->cObjects, lpParams->lplpUnk);
//         if(FAILED(res))
//             WARN("SetObjects() failed, hr %#x.\n", res);
//
//         res = IPropertyPage_GetPageInfo(property_page[i], &page_info);
//         if(FAILED(res))
//             continue;
//
//         dialogs[i].template.cx = MulDiv(page_info.size.cx, 4, font_width);
//         dialogs[i].template.cy = MulDiv(page_info.size.cy, 8, font_height);
//
//         property_sheet_page.u.pResource = &dialogs[i].template;
//         property_sheet_page.lParam = (LPARAM)property_page[i];
//         property_sheet_page.pszTitle = page_info.pszTitle;
//
//         property_sheet.u3.phpage[property_sheet.nPages++] =
//             CreatePropertySheetPageW(&property_sheet_page);
//     }
//
//     PropertySheetW(&property_sheet);
//
//     for(i=0; i<lpParams->cPages; i++) {
//         if(property_page[i])
//             IPropertyPage_Release(property_page[i]);
//     }
//
//     HeapFree(GetProcessHeap(), 0, dialogs);
//     HeapFree(GetProcessHeap(), 0, property_page);
//     HeapFree(GetProcessHeap(), 0, property_sheet.u3.phpage);
//     return S_OK;
// }
//
// /***********************************************************************
//  * OleCreatePropertyFrame (OLEAUT32.417)
//  */
// HRESULT WINAPI OleCreatePropertyFrame(
//         HWND hwndOwner, UINT x, UINT y, LPCOLESTR lpszCaption, ULONG cObjects,
//         LPUNKNOWN* ppUnk, ULONG cPages, LPCLSID pPageClsID, LCID lcid,
//         DWORD dwReserved, LPVOID pvReserved)
// {
//     OCPFIPARAMS ocpf;
//
//     ocpf.cbStructSize =  sizeof(OCPFIPARAMS);
//     ocpf.hWndOwner    = hwndOwner;
//     ocpf.x            = x;
//     ocpf.y            = y;
//     ocpf.lpszCaption  = lpszCaption;
//     ocpf.cObjects     = cObjects;
//     ocpf.lplpUnk      = ppUnk;
//     ocpf.cPages       = cPages;
//     ocpf.lpPages      = pPageClsID;
//     ocpf.lcid         = lcid;
//     ocpf.dispidInitialProperty = 0;
//
//     return OleCreatePropertyFrameIndirect(&ocpf);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
