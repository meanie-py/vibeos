use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use winsrv_sys::winsrv::*;

use crate::explorer::rshell::_CStartMenu_CreateInstance;
use winsrv_sys::consts::SUCCEEDED;

// startmnusite.cpp is still C++; its factory has C linkage via precomp.h.
extern "C" {
    fn CStartMenuSite_CreateInstance(
        Tray: *mut ITrayWindow,
        riid: *const IID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateStartMenu(
    pMenuPopup: *mut IMenuPopup,
    hbmBanner: HBITMAP,
    bSmallIcons: BOOL,
    bRefresh: BOOL,
) -> HRESULT {
    let mut pbb: *mut IBanneredBar = core::ptr::null_mut();
    let mut hRet: HRESULT;

    hRet = (*pMenuPopup).QueryInterface(
        &IID_IBanneredBar,
        &mut pbb as *mut *mut IBanneredBar as *mut *mut core::ffi::c_void,
    );
    if SUCCEEDED(hRet) {
        (*pbb).SetBitmap(hbmBanner);

        hRet = (*pbb).SetIconSize(if bSmallIcons != 0 { BMICON_SMALL } else { BMICON_LARGE });
    }

    if bRefresh != 0 {
        FIXME!("Refresh the Start menu with communicating with SHELL32\n");
    }

    if !pbb.is_null() {
        (*pbb).Release();
    }

    hRet
}
#[no_mangle]
pub unsafe extern "C" fn CreateStartMenu(
    Tray: *mut ITrayWindow,
    ppMenuBand: *mut *mut IMenuBand,
    hbmBanner: HBITMAP,
    bSmallIcons: BOOL,
) -> *mut IMenuPopup {
    let mut hr: HRESULT;
    let mut pMp: *mut IMenuPopup = core::ptr::null_mut();
    let mut pSms: *mut IUnknown = core::ptr::null_mut();
    let mut pMb: *mut IMenuBand = core::ptr::null_mut();
    let mut pIo: *mut IInitializeObject = core::ptr::null_mut();
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let mut pBs: *mut IBandSite = core::ptr::null_mut();
    let mut dwBandId: DWORD = 0;

    let (riid, ppv) = IID_PPV_ARG(core::ptr::addr_of!(IID_IUnknown), core::ptr::addr_of_mut!(pSms) as *mut *mut core::ffi::c_void);
    hr = CStartMenuSite_CreateInstance(Tray, riid, ppv);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    let (riid, ppv) = IID_PPV_ARG(core::ptr::addr_of!(IID_IMenuPopup), core::ptr::addr_of_mut!(pMp) as *mut *mut core::ffi::c_void);
    hr = _CStartMenu_CreateInstance(riid, ppv);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    hr = IUnknown_SetSite(pMp as *mut IUnknown, pSms);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    let (riid, ppv) = IID_PPV_ARG(core::ptr::addr_of!(IID_IInitializeObject), core::ptr::addr_of_mut!(pIo) as *mut *mut core::ffi::c_void);
    hr = ((*(*pMp).lpVtbl).QueryInterface)(pMp, riid, ppv);
    if SUCCEEDED(hr) {
        hr = ((*(*pIo).lpVtbl).Initialize)(pIo);
    } else {
        hr = S_OK;
    }

    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    hr = ((*(*pMp).lpVtbl).GetClient)(pMp, core::ptr::addr_of_mut!(pUnk));
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    let (riid, ppv) = IID_PPV_ARG(core::ptr::addr_of!(IID_IBandSite), core::ptr::addr_of_mut!(pBs) as *mut *mut core::ffi::c_void);
    hr = ((*(*pUnk).lpVtbl).QueryInterface)(pUnk, riid, ppv);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    hr = ((*(*pBs).lpVtbl).EnumBands)(pBs, 0, core::ptr::addr_of_mut!(dwBandId));
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    let (riid, ppv) = IID_PPV_ARG(core::ptr::addr_of!(IID_IMenuBand), core::ptr::addr_of_mut!(pMb) as *mut *mut core::ffi::c_void);
    hr = ((*(*pBs).lpVtbl).GetBandObject)(pBs, dwBandId, riid, ppv);
    if FAILED_UNEXPECTEDLY(hr) != 0 {
        return core::ptr::null_mut();
    }

    UpdateStartMenu(pMp, hbmBanner, bSmallIcons, 0);

    *ppMenuBand = pMb;

    pMp
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2006 - 2007 Thomas Weidenmueller <w3seek@reactos.org>
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
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include "precomp.h"
//
// HRESULT
// UpdateStartMenu(IN OUT IMenuPopup *pMenuPopup,
//                 IN HBITMAP hbmBanner  OPTIONAL,
//                 IN BOOL bSmallIcons,
//                 IN BOOL bRefresh)
// {
//     CComPtr<IBanneredBar> pbb;
//     HRESULT hRet;
//
//     hRet = pMenuPopup->QueryInterface(IID_PPV_ARG(IBanneredBar, &pbb));
//     if (SUCCEEDED(hRet))
//     {
//         hRet = pbb->SetBitmap(hbmBanner);
//
//         /* Update the icon size */
//         hRet = pbb->SetIconSize(bSmallIcons ? BMICON_SMALL : BMICON_LARGE);
//     }
//
//     if (bRefresh)
//     {
//         FIXME("Refresh the Start menu with communicating with SHELL32\n");
//     }
//
//     return hRet;
// }
//
// IMenuPopup*
// CreateStartMenu(IN ITrayWindow *Tray,
//                 OUT IMenuBand **ppMenuBand,
//                 IN HBITMAP hbmBanner OPTIONAL,
//                 IN BOOL bSmallIcons)
// {
//     HRESULT hr;
//     CComPtr<IMenuPopup> pMp;
//     CComPtr<IUnknown> pSms;
//     CComPtr<IMenuBand> pMb;
//     CComPtr<IInitializeObject> pIo;
//     CComPtr<IUnknown> pUnk;
//     CComPtr<IBandSite> pBs;
//     DWORD dwBandId = 0;
//
//     hr = CStartMenuSite_CreateInstance(Tray, IID_PPV_ARG(IUnknown, &pSms));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     hr = _CStartMenu_CreateInstance(IID_PPV_ARG(IMenuPopup, &pMp));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     /* Set the menu site so we can handle messages */
//     hr = IUnknown_SetSite(pMp, pSms);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     /* Initialize the menu object */
//     hr = pMp->QueryInterface(IID_PPV_ARG(IInitializeObject, &pIo));
//     if (SUCCEEDED(hr))
//         hr = pIo->Initialize();
//     else
//         hr = S_OK;
//
//     /* Everything is initialized now. Let's get the IMenuBand interface. */
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     hr = pMp->GetClient(&pUnk);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     hr = pUnk->QueryInterface(IID_PPV_ARG(IBandSite, &pBs));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     /* Finally we have the IBandSite interface, there's only one
//        band in it that apparently provides the IMenuBand interface */
//     hr = pBs->EnumBands(0, &dwBandId);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     hr = pBs->GetBandObject(dwBandId, IID_PPV_ARG(IMenuBand, &pMb));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return NULL;
//
//     UpdateStartMenu(pMp, hbmBanner, bSmallIcons, FALSE);
//
//     *ppMenuBand = pMb.Detach();
//
//     return pMp.Detach();
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
