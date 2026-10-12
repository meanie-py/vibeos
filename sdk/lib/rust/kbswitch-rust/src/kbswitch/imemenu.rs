use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_pMenuList: PIMEMENUNODE =
    ::core::ptr::null::<tagIMEMENUNODE>() as *mut tagIMEMENUNODE;

#[no_mangle]
pub static mut g_nNextMenuID: INT = 0 as INT;

// WINE_DEFAULT_DEBUG_CHANNEL(internat)
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "internat";

unsafe extern "C" fn AddImeMenuNode(pMenu: PIMEMENUNODE) {
    if g_pMenuList.is_null() {
        g_pMenuList = pMenu;
        return;
    }

    (*pMenu).m_pNext = g_pMenuList;
    g_pMenuList = pMenu;
}

unsafe extern "C" fn AllocateImeMenu(mut itemCount: DWORD) -> PIMEMENUNODE {
    let mut cbMenu: SIZE_T = (::core::mem::size_of::<IMEMENUNODE>() as SIZE_T).wrapping_add(
        (itemCount.wrapping_sub(1 as DWORD) as SIZE_T)
            .wrapping_mul(::core::mem::size_of::<IMEMENUITEM>() as SIZE_T),
    );
    let mut pMenu: PIMEMENUNODE = LocalAlloc(LPTR as UINT, cbMenu) as PIMEMENUNODE;
    if pMenu.is_null() {
        return core::ptr::null_mut();
    }
    (*pMenu).m_nItems = itemCount as INT;
    AddImeMenuNode(pMenu);
    return pMenu;
}

unsafe extern "C" fn GetImeMenuItem(
    mut hIMC: HIMC,
    mut lpImeParentMenu: PIMEMENUITEMINFO,
    mut bRightMenu: BOOL,
    mut pItem: PIMEMENUITEM,
) {
    ZeroMemory!(
        pItem as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<IMEMENUITEM>() as usize,
    );
    (*pItem).m_Info = *lpImeParentMenu;

    if (*lpImeParentMenu).fType & IMFT_SUBMENU as UINT != 0 {
        (*pItem).m_pSubMenu =
            CreateImeMenu(hIMC, lpImeParentMenu, bRightMenu) as *mut tagIMEMENUNODE;
    }

    (*pItem).m_nRealID = (*pItem).m_Info.wID;
    let fresh31 = g_nNextMenuID;
    g_nNextMenuID = g_nNextMenuID + 1;
    (*pItem).m_Info.wID = (ID_STARTIMEMENU + (fresh31) as u32) as UINT;
}

#[no_mangle]
pub unsafe extern "C" fn CreateImeMenu(hIMC: HIMC, lpImeParentMenu: PIMEMENUITEMINFO, bRightMenu: BOOL) -> PIMEMENUNODE {
    let dwFlags: DWORD = (if bRightMenu != 0 {
        (IGIMIF_RIGHTMENU) as DWORD
    } else {
        (0) as DWORD
    }) as DWORD;
    let dwTypes: DWORD = ((IGIMII_CMODE) as DWORD
        | (IGIMII_SMODE) as DWORD
        | (IGIMII_CONFIGURE) as DWORD
        | (IGIMII_TOOLS) as DWORD
        | (IGIMII_HELP) as DWORD
        | (IGIMII_OTHER) as DWORD) as DWORD;
    let mut itemCount: DWORD = ImmGetImeMenuItems(
        hIMC,
        dwFlags,
        dwTypes,
        lpImeParentMenu as LPIMEMENUITEMINFOW,
        core::ptr::null_mut::<tagIMEMENUITEMINFOW>(),
        (0) as DWORD,
    );
    if itemCount == 0 {
        return core::ptr::null_mut::<tagIMEMENUNODE>();
    }

    let mut pMenu: PIMEMENUNODE = AllocateImeMenu(itemCount);
    if pMenu.is_null() {
        return core::ptr::null_mut::<tagIMEMENUNODE>();
    }

    let mut cbItems: DWORD = (core::mem::size_of::<IMEMENUITEMINFO>() as DWORD)
        .wrapping_mul(itemCount);
    let mut pImeMenuItems: PIMEMENUITEMINFO = LocalAlloc(
        (LPTR) as UINT,
        cbItems as SIZE_T,
    ) as PIMEMENUITEMINFO;
    if pImeMenuItems.is_null() {
        LocalFree(pMenu as HLOCAL);
        return core::ptr::null_mut::<tagIMEMENUNODE>();
    }

    itemCount = ImmGetImeMenuItems(
        hIMC,
        dwFlags,
        dwTypes,
        lpImeParentMenu as LPIMEMENUITEMINFOW,
        pImeMenuItems as LPIMEMENUITEMINFOW,
        cbItems,
    );
    if itemCount == 0 {
        LocalFree(pImeMenuItems as HLOCAL);
        LocalFree(pMenu as HLOCAL);
        return core::ptr::null_mut::<tagIMEMENUNODE>();
    }

    let mut pItems: PIMEMENUITEM = ((*pMenu).m_Items).as_mut_ptr();
    let mut iItem: DWORD = (0) as DWORD;
    while iItem < itemCount {
        GetImeMenuItem(
            hIMC,
            pImeMenuItems.add(iItem as usize) as PIMEMENUITEMINFO,
            bRightMenu,
            pItems.add(iItem as usize) as PIMEMENUITEM,
        );
        iItem = iItem.wrapping_add(1);
    }

    LocalFree(pImeMenuItems as HLOCAL);
    return pMenu;
}

unsafe extern "C" fn FillImeMenuItem(mut pItemInfo: LPMENUITEMINFO, mut pItem: *const IMEMENUITEM) -> BOOL {
    ZeroMemory!(
        pItemInfo as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFO>(),
    );
    (*pItemInfo).cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
    (*pItemInfo).fMask = (MIIM_ID | MIIM_STATE | MIIM_DATA) as UINT;
    (*pItemInfo).wID = (*pItem).m_Info.wID;
    (*pItemInfo).fState = (*pItem).m_Info.fState;
    (*pItemInfo).dwItemData = (*pItem).m_Info.dwItemData as ULONG_PTR;

    if (*pItem).m_Info.fType != 0 {
        (*pItemInfo).fMask |= MIIM_FTYPE as UINT;
        (*pItemInfo).fType = 0 as UINT;
        if (*pItem).m_Info.fType & IMFT_RADIOCHECK as UINT != 0 {
            (*pItemInfo).fType |= MFT_RADIOCHECK as UINT;
        }
        if (*pItem).m_Info.fType & IMFT_SEPARATOR as UINT != 0 {
            (*pItemInfo).fType |= MFT_SEPARATOR as UINT;
        }
    }

    if (*pItem).m_Info.fType & IMFT_SUBMENU as UINT != 0 {
        (*pItemInfo).fMask |= MIIM_SUBMENU as UINT;
        (*pItemInfo).hSubMenu = CreatePopupMenu();
        if MakeImeMenu((*pItemInfo).hSubMenu, (*pItem).m_pSubMenu) == 0 {
            DestroyMenu((*pItemInfo).hSubMenu);
            (*pItemInfo).hSubMenu = core::ptr::null_mut();
            return (FALSE) as BOOL;
        }
    }

    if !(*pItem).m_Info.hbmpChecked.is_null() && !(*pItem).m_Info.hbmpUnchecked.is_null() {
        (*pItemInfo).fMask |= MIIM_CHECKMARKS as UINT;
        (*pItemInfo).hbmpChecked = (*pItem).m_Info.hbmpChecked;
        (*pItemInfo).hbmpUnchecked = (*pItem).m_Info.hbmpUnchecked;
    }

    if !(*pItem).m_Info.hbmpItem.is_null() {
        (*pItemInfo).fMask |= MIIM_BITMAP as UINT;
        (*pItemInfo).hbmpItem = (*pItem).m_Info.hbmpItem;
    }

    let mut szString: PCTSTR = core::ptr::addr_of!((*pItem).m_Info.szString) as PCTSTR;
    if !szString.is_null() && *szString.offset(0 as isize) != 0 {
        (*pItemInfo).fMask |= MIIM_STRING as UINT;
        (*pItemInfo).dwTypeData = (szString as PTSTR).cast();
        (*pItemInfo).cch = lstrlen(szString) as UINT;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn MakeImeMenu(hMenu: HMENU, pMenu: *const IMEMENUNODE) -> BOOL {
    if pMenu.is_null() || (*pMenu).m_nItems == 0 {
        return (FALSE) as BOOL;
    }

    let mut iItem: INT = 0;
    while iItem < (*pMenu).m_nItems {
        let mut mi: MENUITEMINFO = core::mem::zeroed();
        mi.cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
        if FillImeMenuItem(
            (&raw mut mi).cast(),
            (&raw const (*pMenu).m_Items as *const IMEMENUITEM).offset(iItem as isize)
                as *const IMEMENUITEM,
        ) == 0
        {
            ERR!("FillImeMenuItem failed\n");
            return (FALSE) as BOOL;
        }
        if InsertMenuItem(hMenu, iItem as UINT, (TRUE as i32), &raw mut mi) == 0
        {
            ERR!("InsertMenuItem failed\n");
            return (FALSE) as BOOL;
        }
        iItem += 1;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn MenuFromImeMenu(pMenu: *const IMEMENUNODE) -> HMENU {
    let mut hMenu: HMENU = CreatePopupMenu();
    if pMenu.is_null() {
        return hMenu;
    }
    if MakeImeMenu(hMenu, pMenu) == 0 {
        DestroyMenu(hMenu);
        return core::ptr::null_mut();
    }
    return hMenu;
}
#[no_mangle]
pub unsafe extern "C" fn GetRealImeMenuID(pMenu: *const IMEMENUNODE, nFakeID: INT) -> INT {
    if pMenu.is_null() || (*pMenu).m_nItems == 0 || nFakeID < (ID_STARTIMEMENU as i32) {
        return 0;
    }

    let mut iItem: INT = 0;
    while iItem < (*pMenu).m_nItems {
        let pItem: *const IMEMENUITEM = (core::ptr::addr_of!((*pMenu).m_Items) as *const IMEMENUITEM)
            .offset(iItem as isize);
        if (*pItem).m_Info.wID as INT == nFakeID {
            return (*pItem).m_nRealID as INT;
        }

        if !(*pItem).m_pSubMenu.is_null() {
            let nRealID: INT = GetRealImeMenuID((*pItem).m_pSubMenu, nFakeID);
            if nRealID != 0 {
                return nRealID;
            }
        }
        iItem += 1;
    }

    return 0;
}

unsafe extern "C" fn FreeMenuNode(mut pMenuNode: PIMEMENUNODE) -> BOOL {
    if pMenuNode.is_null() {
        return (FALSE) as BOOL;
    }
    let mut iItem: INT = 0 as INT;
    while iItem < (*pMenuNode).m_nItems {
        let mut pItem: PIMEMENUITEM = (&raw mut (*pMenuNode).m_Items as *mut IMEMENUITEM)
            .offset(iItem as isize) as PIMEMENUITEM;
        if !(*pItem).m_Info.hbmpChecked.is_null() {
            DeleteObject((*pItem).m_Info.hbmpChecked as HGDIOBJ);
        }
        if !(*pItem).m_Info.hbmpUnchecked.is_null() {
            DeleteObject((*pItem).m_Info.hbmpUnchecked as HGDIOBJ);
        }
        if !(*pItem).m_Info.hbmpItem.is_null() {
            DeleteObject((*pItem).m_Info.hbmpItem as HGDIOBJ);
        }
        iItem += 1;
    }
    LocalFree(pMenuNode as HLOCAL);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CleanupImeMenus() {
    if g_pMenuList.is_null() {
        return;
    }

    let mut pNext: PIMEMENUNODE = core::ptr::null_mut();
    let mut pNode: PIMEMENUNODE = g_pMenuList;
    while !pNode.is_null() {
        pNext = (*pNode).m_pNext;
        FreeMenuNode(pNode);
        pNode = pNext;
    }

    g_pMenuList = core::ptr::null_mut();
    g_nNextMenuID = 0;
}
