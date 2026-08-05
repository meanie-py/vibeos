use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `const`, not `static`: an array of raw pointers is not Sync, and this table is only ever
// read for its elements. Each element still points at `w!`'s own `static` UTF-16 buffer, so
// the string addresses are stable -- only the ARRAY is materialized per use.
pub const ObjectPickerAttributes: [PCWSTR; 1] = [w!("ObjectSid") as PCWSTR];
unsafe extern "C" fn LengthOfStrResource(
    hInst: HINSTANCE,
    uID: u32,
) -> i32 {
    let hrSrc: HRSRC;
    let hRes: HGLOBAL;
    let mut lpName: LPWSTR;
    let mut lpStr: LPWSTR;

    if hInst == core::ptr::null_mut() {
        return -1;
    }

    lpName = MAKEINTRESOURCE((uID >> 4) + 1) as LPWSTR;

    hrSrc = FindResourceW(hInst, lpName, RT_STRING as LPWSTR);
    if hrSrc != core::ptr::null_mut() {
        hRes = LoadResource(hInst, hrSrc);
        if hRes != core::ptr::null_mut() {
            lpStr = LockResource(hRes) as LPWSTR;
            if lpStr != core::ptr::null_mut() {
                let mut x: u32;

                let uID_masked = uID & 0xF;
                x = 0;
                while x < uID_masked {
                    lpStr = lpStr.offset(((*lpStr) + 1) as isize);
                    x += 1;
                }

                return (*lpStr) as i32;
            }
        }
    }
    -1
}
unsafe extern "C" fn AllocAndLoadString(
    lpTarget: *mut LPWSTR,
    hInst: HINSTANCE,
    uID: u32,
) -> i32 {
    let mut ln: i32;

    ln = LengthOfStrResource(hInst, uID);
    ln += 1;
    if ln > 0 {
        *lpTarget = LocalAlloc(LMEM_FIXED, ((ln as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if !(*lpTarget).is_null() {
            let Ret: i32;
            Ret = LoadStringW(hInst, uID, *lpTarget, ln);
            if Ret == 0 {
                LocalFree(*lpTarget as HLOCAL);
            }
            return Ret;
        }
    }
    return 0;
}
// The `_v` twin of the variadic entry in dll/win32/aclui/rust_varargs_shim.c. `va_start` is
// the only part that has to be C; everything the function actually DOES is here, and the
// va_list arrives by pointer, which is exactly what FormatMessageW's `Arguments` wants.
#[no_mangle]
pub unsafe extern "C" fn LoadAndFormatString_v(
    hInstance: HINSTANCE,
    uID: u32,
    lpTarget: *mut LPWSTR,
    lArgs: *mut va_list,
) -> u32 {
    let mut Ret: u32 = 0;
    let mut lpFormat: LPWSTR = core::ptr::null_mut();

    if AllocAndLoadString(
        core::ptr::addr_of_mut!(lpFormat),
        hInstance,
        uID,
    ) > 0
    {
        Ret = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING,
            lpFormat as *const c_void,
            0,
            0,
            lpTarget as LPWSTR,
            0,
            lArgs,
        );

        LocalFree(lpFormat as HLOCAL);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "C" fn ListViewGetSelectedItemData(hwnd: HWND) -> LPARAM {
    let Index = ListView_GetNextItem(
        hwnd,
        -1,
        LVNI_SELECTED,
    );
    if Index != -1 {
        let mut li: LVITEM = core::mem::zeroed();

        li.mask = LVIF_PARAM;
        li.iItem = Index;
        li.iSubItem = 0;

        if ListView_GetItem(hwnd, &mut li) != 0 {
            return li.lParam;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn ListViewSelectItem(
    hwnd: HWND,
    Index: i32,
) -> i32 {
    let mut li: _LVITEMW = core::mem::zeroed();

    li.mask = LVIF_STATE;
    li.iItem = Index;
    li.iSubItem = 0;
    li.state = LVIS_SELECTED;
    li.stateMask = LVIS_SELECTED;

    ListView_SetItem(hwnd, &mut li)
}
#[no_mangle]
pub unsafe extern "C" fn InitializeObjectPicker(
    ServerName: PCWSTR,
    ObjectInfo: *mut SI_OBJECT_INFO,
    pDsObjectPicker: *mut *mut IDsObjectPicker,
) -> HRESULT {
    let mut hRet: HRESULT;

    *pDsObjectPicker = core::ptr::null_mut();

    hRet = CoCreateInstance(
        &CLSID_DsObjectPicker,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IDsObjectPicker,
        pDsObjectPicker as *mut LPVOID,
    );
    if ((hRet) >= 0) {
        let mut InitInfo: DSOP_INIT_INFO = core::mem::zeroed();
        let mut i: u32;
        static mut Scopes: [DSOP_SCOPE_INIT_INFO; 1] = [DSOP_SCOPE_INIT_INFO {
            cbSize: core::mem::size_of::<DSOP_SCOPE_INIT_INFO>() as u32,
            flType: DSOP_SCOPE_TYPE_TARGET_COMPUTER,
            flScope: DSOP_SCOPE_FLAG_DEFAULT_FILTER_USERS | DSOP_SCOPE_FLAG_DEFAULT_FILTER_GROUPS | DSOP_SCOPE_FLAG_STARTING_SCOPE,
            FilterFlags: DSOP_FILTER_FLAGS {
                Uplevel: DSOP_UPLEVEL_FILTER_FLAGS {
                    flBothModes: 0,
                    flMixedModeOnly: 0,
                    flNativeModeOnly: 0,
                },
                flDownlevel: DSOP_DOWNLEVEL_FILTER_USERS | DSOP_DOWNLEVEL_FILTER_LOCAL_GROUPS | DSOP_DOWNLEVEL_FILTER_GLOBAL_GROUPS | DSOP_DOWNLEVEL_FILTER_ALL_WELLKNOWN_SIDS,
            },
            pwzDcName: core::ptr::null_mut(),
            pwzADsPath: core::ptr::null_mut(),
            hr: S_OK,
        }];

        InitInfo.cbSize = core::mem::size_of::<DSOP_INIT_INFO>() as u32;
        InitInfo.pwzTargetComputer = ServerName;
        InitInfo.cDsScopeInfos = (core::mem::size_of_val(&Scopes) / core::mem::size_of::<DSOP_SCOPE_INIT_INFO>()) as u32;
        InitInfo.aDsScopeInfos = Scopes.as_mut_ptr();
        InitInfo.flOptions = DSOP_FLAG_MULTISELECT;
        InitInfo.cAttributesToFetch = (core::mem::size_of_val(&ObjectPickerAttributes) / core::mem::size_of::<PCWSTR>()) as u32;
        InitInfo.apwzAttributeNames = ObjectPickerAttributes.as_ptr() as *mut PCWSTR;

        i = 0;
        while i < InitInfo.cDsScopeInfos {
            if ((*ObjectInfo).dwFlags & SI_SERVER_IS_DC) != 0
                && (InitInfo.aDsScopeInfos.add(i as usize).read().flType & DSOP_SCOPE_TYPE_UPLEVEL_JOINED_DOMAIN) != 0
            {
                (*InitInfo.aDsScopeInfos.add(i as usize)).pwzDcName = InitInfo.pwzTargetComputer;
            }
            i += 1;
        }

        hRet = (**pDsObjectPicker).lpVtbl.as_ref().unwrap_unchecked().Initialize.unwrap_unchecked()(
            *pDsObjectPicker,
            &mut InitInfo,
        );

        if ((hRet) < 0) {
            (**pDsObjectPicker).lpVtbl.as_ref().unwrap_unchecked().Release.unwrap_unchecked()(*pDsObjectPicker);
        }
    }

    hRet
}
#[no_mangle]
pub unsafe extern "C" fn InvokeObjectPickerDialog(
    pDsObjectPicker: *mut IDsObjectPicker,
    hwndParent: HWND,
    SelectedSidCallback: POBJPICK_SELECTED_SID,
    Context: PVOID,
) -> HRESULT {
    let mut pdo: *mut IDataObject = core::ptr::null_mut();
    let mut hRet: HRESULT;

    hRet = ((*(*pDsObjectPicker).lpVtbl).InvokeDialog.unwrap_unchecked())(
        pDsObjectPicker,
        hwndParent,
        &mut pdo,
    );
    if hRet == S_OK {
        let mut stm: STGMEDIUM = core::mem::zeroed();
        let mut fe: FORMATETC = core::mem::zeroed();

        fe.cfFormat = RegisterClipboardFormat(CFSTR_DSOP_DS_SELECTION_LIST_BUF.as_ptr()) as u16;
        fe.ptd = core::ptr::null_mut();
        fe.dwAspect = DVASPECT_CONTENT;
        fe.lindex = -(1 as i32);
        fe.tymed = TYMED_HGLOBAL;

        hRet = ((*(*pdo).lpVtbl).GetData.unwrap_unchecked())(pdo, &mut fe, &mut stm);
        if hRet >= 0 {
            let SelectionList: PDS_SELECTION_LIST = GlobalLock(*stm.__bindgen_anon_1.hGlobal.as_ref()) as PDS_SELECTION_LIST;
            if !SelectionList.is_null() {
                let mut vSid: LPVARIANT;
                let mut pSid: PSID = core::ptr::null_mut();
                let mut i: u32;
                let mut contLoop: u32 = (TRUE) as u32;

                i = 0;
                while i < (*SelectionList).cItems && contLoop != 0 {
                    vSid = (*(*SelectionList).aDsSelection.as_ptr().add(i as usize)).pvarFetchedAttributes;

                    if !vSid.is_null() && V_VT(vSid) == ((VT_ARRAY | VT_UI1)) as u16 {
                        hRet = SafeArrayAccessData(
                            V_ARRAY(vSid),
                            &mut pSid as *mut PSID as *mut *mut core::ffi::c_void,
                        );
                        if hRet < 0 {
                            break;
                        }

                        if !pSid.is_null() {
                            contLoop = (SelectedSidCallback.unwrap_unchecked()(
                                pDsObjectPicker,
                                hwndParent,
                                pSid,
                                Context,
                            )) as u32;
                        }

                        SafeArrayUnaccessData(V_ARRAY(vSid));
                    }

                    i += 1;
                }

                GlobalUnlock(*stm.__bindgen_anon_1.hGlobal.as_ref());

                if hRet >= 0 {
                    hRet = S_OK;
                }
            } else {
                hRet = E_FAIL;
            }

            ReleaseStgMedium(&mut stm);
        }

        ((*(*pdo).lpVtbl).Release.unwrap_unchecked())(pdo);
    }

    return hRet;
}
#[no_mangle]
pub unsafe extern "C" fn FreeObjectPicker(pDsObjectPicker: *mut IDsObjectPicker) {
    ((*(*pDsObjectPicker).lpVtbl).Release.unwrap_unchecked())(pDsObjectPicker);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Access Control List Editor
//  * Copyright (C) 2004-2005 ReactOS Team
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
// /*
//  * PROJECT:         ReactOS Access Control List Editor
//  * FILE:            lib/aclui/misc.c
//  * PURPOSE:         Access Control List Editor
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *
//  * UPDATE HISTORY:
//  *      07/01/2005  Created
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// static PCWSTR ObjectPickerAttributes[] =
// {
//     L"ObjectSid",
// };
//
// static INT
// LengthOfStrResource( HINSTANCE hInst,
//                      UINT uID)
// {
//     HRSRC hrSrc;
//     HGLOBAL hRes;
//     LPWSTR lpName, lpStr;
//
//     if (hInst == NULL)
//     {
//         return -1;
//     }
//
//     /* There are always blocks of 16 strings */
//     lpName = (LPWSTR)MAKEINTRESOURCE((uID >> 4) + 1);
//
//     /* Find the string table block */
//     if ((hrSrc = FindResourceW(hInst, lpName, (LPWSTR)RT_STRING)) &&
//         (hRes = LoadResource(hInst, hrSrc)) &&
//         (lpStr = LockResource(hRes)))
//     {
//         UINT x;
//
//         /* Find the string we're looking for */
//         uID &= 0xF; /* position in the block, same as % 16 */
//         for (x = 0; x < uID; x++)
//         {
//             lpStr += (*lpStr) + 1;
//         }
//
//         /* Found the string */
//         return (int)(*lpStr);
//     }
//     return -1;
// }
//
// static INT
// AllocAndLoadString( LPWSTR *lpTarget,
//                     HINSTANCE hInst,
//                     UINT uID)
// {
//     INT ln;
//
//     ln = LengthOfStrResource(hInst,
//                              uID);
//     if (ln++ > 0)
//     {
//         (*lpTarget) = (LPWSTR)LocalAlloc(LMEM_FIXED,
//                                          ln * sizeof(WCHAR));
//         if ((*lpTarget) != NULL)
//         {
//             INT Ret;
//             if (!(Ret = LoadStringW(hInst, uID, *lpTarget, ln)))
//             {
//                 LocalFree((HLOCAL)(*lpTarget));
//             }
//             return Ret;
//         }
//     }
//     return 0;
// }
//
// DWORD
// LoadAndFormatString( HINSTANCE hInstance,
//                      UINT uID,
//                      LPWSTR *lpTarget,
//                     ...)
// {
//     DWORD Ret = 0;
//     LPWSTR lpFormat;
//     va_list lArgs;
//
//     if (AllocAndLoadString(&lpFormat,
//                            hInstance,
//                            uID) > 0)
//     {
// //         /* let's use FormatMessage to format it because it has the ability to allocate
//            memory automatically */
//         Ret = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING,
//                              lpFormat,
//                              0,
//                              0,
//                              (LPWSTR)lpTarget,
//                              0,
//                              &lArgs);
// //
//         LocalFree((HLOCAL)lpFormat);
//     }
//
//     return Ret;
// }
//
// LPARAM
// ListViewGetSelectedItemData( HWND hwnd)
// {
//     int Index;
//
//     Index = ListView_GetNextItem(hwnd,
//                                  -1,
//                                  LVNI_SELECTED);
//     if (Index != -1)
//     {
//         LVITEM li;
//
//         li.mask = LVIF_PARAM;
//         li.iItem = Index;
//         li.iSubItem = 0;
//
//         if (ListView_GetItem(hwnd,
//                              &li))
//         {
//             return li.lParam;
//         }
//     }
//
//     return 0;
// }
//
// BOOL
// ListViewSelectItem( HWND hwnd,
//                     INT Index)
// {
//     LVITEM li;
//
//     li.mask = LVIF_STATE;
//     li.iItem = Index;
//     li.iSubItem = 0;
//     li.state = LVIS_SELECTED;
//     li.stateMask = LVIS_SELECTED;
//
//     return ListView_SetItem(hwnd,
//                             &li);
// }
//
// HRESULT
// InitializeObjectPicker( PCWSTR ServerName,
//                         PSI_OBJECT_INFO ObjectInfo,
//                         IDsObjectPicker **pDsObjectPicker)
// {
//     HRESULT hRet;
//
//     *pDsObjectPicker = NULL;
//
//     hRet = CoCreateInstance(&CLSID_DsObjectPicker,
//                             NULL,
//                             CLSCTX_INPROC_SERVER,
//                             &IID_IDsObjectPicker,
//                             (LPVOID*)pDsObjectPicker);
//     if (SUCCEEDED(hRet))
//     {
//         DSOP_INIT_INFO InitInfo;
//         UINT i;
//         static DSOP_SCOPE_INIT_INFO Scopes[] =
//         {
//             {
//                 sizeof(DSOP_SCOPE_INIT_INFO),
//                 DSOP_SCOPE_TYPE_TARGET_COMPUTER,
//                 DSOP_SCOPE_FLAG_DEFAULT_FILTER_USERS | DSOP_SCOPE_FLAG_DEFAULT_FILTER_GROUPS |
//                     DSOP_SCOPE_FLAG_STARTING_SCOPE,
//                 {
//                     {
//                         0,
//                         0,
//                         0
//                     },
//                     DSOP_DOWNLEVEL_FILTER_USERS | DSOP_DOWNLEVEL_FILTER_LOCAL_GROUPS |
//                         DSOP_DOWNLEVEL_FILTER_GLOBAL_GROUPS | DSOP_DOWNLEVEL_FILTER_ALL_WELLKNOWN_SIDS
//                 },
//                 NULL,
//                 NULL,
//                 S_OK
//             },
//         };
//
//         InitInfo.cbSize = sizeof(InitInfo);
//         InitInfo.pwzTargetComputer = ServerName;
//         InitInfo.cDsScopeInfos = sizeof(Scopes) / sizeof(Scopes[0]);
//         InitInfo.aDsScopeInfos = Scopes;
//         InitInfo.flOptions = DSOP_FLAG_MULTISELECT;
//         InitInfo.cAttributesToFetch = sizeof(ObjectPickerAttributes) / sizeof(ObjectPickerAttributes[0]);
//         InitInfo.apwzAttributeNames = ObjectPickerAttributes;
//
//         for (i = 0; i < InitInfo.cDsScopeInfos; i++)
//         {
//             if ((ObjectInfo->dwFlags & SI_SERVER_IS_DC) &&
//                 (InitInfo.aDsScopeInfos[i].flType & DSOP_SCOPE_TYPE_UPLEVEL_JOINED_DOMAIN))
//             {
//                 /* only set the domain controller string if we know the target
//                    computer is a domain controller and the scope type is an
//                    up-level domain to which the target computer is joined */
//                 InitInfo.aDsScopeInfos[i].pwzDcName = InitInfo.pwzTargetComputer;
//             }
//         }
//
//         hRet = (*pDsObjectPicker)->lpVtbl->Initialize(*pDsObjectPicker,
//                                                       &InitInfo);
//
//         if (FAILED(hRet))
//         {
//             /* delete the object picker in case initialization failed! */
//             (*pDsObjectPicker)->lpVtbl->Release(*pDsObjectPicker);
//         }
//     }
//
//     return hRet;
// }
//
// HRESULT
// InvokeObjectPickerDialog( IDsObjectPicker *pDsObjectPicker,
//                           HWND hwndParent  ,
//                           POBJPICK_SELECTED_SID SelectedSidCallback,
//                           PVOID Context  )
// {
//     IDataObject *pdo = NULL;
//     HRESULT hRet;
//
//     hRet = pDsObjectPicker->lpVtbl->InvokeDialog(pDsObjectPicker,
//                                                  hwndParent,
//                                                  &pdo);
//     if (hRet == S_OK)
//     {
//         STGMEDIUM stm;
//         FORMATETC fe;
//
//         fe.cfFormat = (WORD)RegisterClipboardFormat(CFSTR_DSOP_DS_SELECTION_LIST_BUF.as_ptr());
//         fe.ptd = NULL;
//         fe.dwAspect = DVASPECT_CONTENT;
//         fe.lindex = -1;
//         fe.tymed = TYMED_HGLOBAL;
//
//         hRet = pdo->lpVtbl->GetData(pdo,
//                                     &fe,
//                                     &stm);
//         if (SUCCEEDED(hRet))
//         {
//             PDS_SELECTION_LIST SelectionList = (PDS_SELECTION_LIST)GlobalLock(*stm.__bindgen_anon_1.hGlobal.as_ref());
//             if (SelectionList != NULL)
//             {
//                 LPVARIANT vSid;
//                 PSID pSid;
//                 UINT i;
//                 BOOL contLoop = TRUE;
//
//                 for (i = 0; i < SelectionList->cItems && contLoop; i++)
//                 {
//                     vSid = SelectionList->aDsSelection[i].pvarFetchedAttributes;
//
//                     if (vSid != NULL && V_VT(vSid) == (VT_ARRAY | VT_UI1))
//                     {
//                         hRet = SafeArrayAccessData(V_ARRAY(vSid),
//                                                    (void **)&pSid);
//                         if (FAILED(hRet))
//                         {
//                             break;
//                         }
//
//                         if (pSid != NULL)
//                         {
//                             contLoop = SelectedSidCallback.unwrap_unchecked()(pDsObjectPicker,
//                                                            hwndParent,
//                                                            pSid,
//                                                            Context);
//                         }
//
//                         SafeArrayUnaccessData(V_ARRAY(vSid));
//                     }
//                 }
//
//                 GlobalUnlock(*stm.__bindgen_anon_1.hGlobal.as_ref());
//
//                 if (SUCCEEDED(hRet))
//                 {
//                     /* return S_OK instead of possible other success codes if
//                        everything went well */
//                     hRet = S_OK;
//                 }
//             }
//             else
//             {
//                 /* unable to translate the selection pointer handle, indicate
//                    failure */
//                 hRet = E_FAIL;
//             }
//
//             ReleaseStgMedium(&stm);
//         }
//
//         pdo->lpVtbl->Release(pdo);
//     }
//
//     return hRet;
// }
//
// VOID
// FreeObjectPicker( IDsObjectPicker *pDsObjectPicker)
// {
//     pDsObjectPicker->lpVtbl->Release(pDsObjectPicker);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
