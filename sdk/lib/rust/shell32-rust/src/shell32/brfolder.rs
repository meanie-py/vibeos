//! FLIPPED: this TU is wired into the crate and shipping; the C++ original is
//! commented out of dll/win32/shell32/CMakeLists.txt.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_731D7EC48EEF: [WCHAR; 3] = [0x0020, 0x0009, 0x0000];

use core::ffi::c_void;
use crate::prelude::*;
use crate::shellcom::{IContextMenu, IContextMenuVtbl};
use crate::shell32::sdk_consts::*;

// ---- file-scope preamble (translated) ----
pub const SHV_CHANGE_NOTIFY: u32 = WM_USER + 0x1111;

pub const SHGDN_TREE: u32 = SHGDN_NORMAL | SHGDN_INFOLDER;

#[repr(C)]
pub struct BrFolder {
    pub lpBrowseInfo: LPBROWSEINFOW,
    pub hWnd: HWND,
    pub hwndTreeView: HWND,
    pub pidlRet: PIDLIST_ABSOLUTE,
    pub layout: *mut LAYOUT_DATA,
    pub szMin: SIZE,
    pub hChangeNotify: ULONG,
    pub pContextMenu: *mut IContextMenu,
}

#[repr(C)]
pub struct BrItemData {
    pub lpsfParent: CComPtr<IShellFolder>,
    pub pidlChild: PCIDLIST_RELATIVE,
    pub pidlFull: CComHeapPtr<ITEMIDLIST_ABSOLUTE>,
}

pub static mut g_layout_info: [LAYOUT_INFO; 7] = [
    LAYOUT_INFO { m_nCtrlID: IDC_BROWSE_FOR_FOLDER_TITLE, m_uEdges: BF_TOP | BF_LEFT | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDC_BROWSE_FOR_FOLDER_STATUS, m_uEdges: BF_TOP | BF_LEFT | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDC_BROWSE_FOR_FOLDER_TREEVIEW, m_uEdges: BF_TOP | BF_BOTTOM | BF_LEFT | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT, m_uEdges: BF_TOP | BF_LEFT | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDC_BROWSE_FOR_FOLDER_NEW_FOLDER, m_uEdges: BF_BOTTOM | BF_LEFT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDOK, m_uEdges: BF_BOTTOM | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
    LAYOUT_INFO { m_nCtrlID: IDCANCEL, m_uEdges: BF_BOTTOM | BF_RIGHT, m_hwndCtrl: core::ptr::null_mut(),
        m_margin1: SIZE { cx: 0, cy: 0 }, m_margin2: SIZE { cx: 0, cy: 0 } },
];

pub const SUPPORTED_FLAGS: u32 = BIF_STATUSTEXT | BIF_BROWSEFORCOMPUTER | BIF_RETURNFSANCESTORS |
                                  BIF_RETURNONLYFSDIRS | BIF_NONEWFOLDERBUTTON | BIF_NEWDIALOGSTYLE |
                                  BIF_BROWSEINCLUDEFILES;
unsafe extern "C" fn ILCloneToDepth(pidlSrc: LPCITEMIDLIST, depth: UINT) -> LPITEMIDLIST {
    let mut cb: usize = 0;
    let mut pidl = pidlSrc;
    let mut remaining = depth;
    while !pidl.is_null() && remaining > 0 {
        cb += (*pidl).mkid.cb as usize;
        pidl = ILGetNext(pidl);
        remaining -= 1;
    }
    let pidlOut: LPITEMIDLIST = SHAlloc((cb + core::mem::size_of::<WORD>()) as SIZE_T) as LPITEMIDLIST;
    if !pidlOut.is_null() {
        CopyMemory(pidlOut as *mut _, pidlSrc as *const _, cb);
        ZeroMemory((pidlOut as *mut u8).add(cb), core::mem::size_of::<WORD>());
    }
    pidlOut
}
unsafe extern "C" fn BrowseFlagsToSHCONTF(ulFlags: u32) -> u32 {
    SHCONTF_FOLDERS | if (ulFlags & BIF_BROWSEINCLUDEFILES) != 0 {
        SHCONTF_NONFOLDERS
    } else {
        0
    }
}
unsafe extern "C" fn BrFolder_Callback(lpBrowseInfo: *mut BROWSEINFOW, hWnd: HWND, uMsg: UINT, lParam: LPARAM) {
    if let Some(callback) = (*lpBrowseInfo).lpfn {
        callback(hWnd, uMsg, lParam, (*lpBrowseInfo).lParam);
    }
}
unsafe extern "C" fn BrFolder_GetItemData(info: *mut BrFolder, hItem: HTREEITEM) -> *mut BrItemData {
    let mut item: TVITEMW = core::mem::zeroed();
    item.mask = TVIF_HANDLE | TVIF_PARAM;
    item.hItem = hItem;
    if TreeView_GetItem((*info).hwndTreeView, &mut item) == 0 {
        return core::ptr::null_mut();
    }
    item.lParam as *mut BrItemData
}
unsafe extern "C" fn BrFolder_GetItemAttributes(info: *mut BrFolder, hItem: HTREEITEM, Query: SFGAOF) -> SFGAOF {
    let item = BrFolder_GetItemData(info, hItem);
    let mut Att = Query;
    let hr = if !item.is_null() {
        let sf = (*item).lpsfParent.Get();
        ((*(*sf).lpVtbl).GetAttributesOf)(
            sf,
            1,
            (core::ptr::addr_of!((*item).pidlChild)).cast(),
            &mut Att,
        )
    } else {
        E_FAIL
    };
    if hr >= 0 {
        Query & Att
    } else {
        0
    }
}
unsafe extern "C" fn BrFolder_GetChildrenEnum(
    info: *mut BrFolder,
    pItemData: *mut BrItemData,
    ppEnum: *mut *mut IEnumIDList,
) -> i32 {
    if pItemData.is_null() {
        return E_FAIL;
    }

    let mut pEnumIL: *mut IEnumIDList = core::ptr::null_mut();
    let pidlRef: PCUITEMID_CHILD = (*pItemData).pidlChild;
    let mut attrs: u32 = SFGAO_HASSUBFOLDER | SFGAO_FOLDER;
    let lpsf: *mut IShellFolder = (*pItemData).lpsfParent.Get();
    let mut hr: i32 = ((*(*lpsf).lpVtbl).GetAttributesOf)(lpsf, 1, core::ptr::addr_of!(pidlRef).cast(), &mut attrs);
    if FAILED_UNEXPECTEDLY!(hr) || (attrs & SFGAO_FOLDER) == 0 {
        return E_FAIL;
    }

    let mut psfChild: *mut IShellFolder = core::ptr::null_mut();
    if _ILIsDesktop((*pItemData).pidlFull.m_pData) != 0 {
        hr = SHGetDesktopFolder(&mut psfChild);
    } else {
        hr = ((*(*lpsf).lpVtbl).BindToObject)(lpsf, (pidlRef).cast(), core::ptr::null_mut(), core::ptr::addr_of!(IID_IShellFolder).cast(), &mut psfChild as *mut *mut IShellFolder as *mut *mut core::ffi::c_void);
    }
    if FAILED_UNEXPECTEDLY!(hr) {
        if !psfChild.is_null() {
            ((*(*psfChild).lpVtbl).Release)(psfChild);
        }
        return E_FAIL;
    }

    let flags: u32 = BrowseFlagsToSHCONTF((*(*info).lpBrowseInfo).ulFlags);
    hr = ((*(*psfChild).lpVtbl).EnumObjects)(psfChild, ((*info).hWnd).cast(), flags, &mut pEnumIL);
    if hr == S_OK {
        if !pEnumIL.is_null() && (((*(*pEnumIL).lpVtbl).Skip)(pEnumIL, 1) != S_OK || ((*(*pEnumIL).lpVtbl).Reset)(pEnumIL) < 0) {
            ((*(*pEnumIL).lpVtbl).Release)(pEnumIL);
            pEnumIL = core::ptr::null_mut();
        }
    }

    if pEnumIL.is_null() || (attrs & SFGAO_HASSUBFOLDER) == 0 {
        if !psfChild.is_null() {
            ((*(*psfChild).lpVtbl).Release)(psfChild);
        }
        return E_FAIL;
    }

    if !ppEnum.is_null() {
        *ppEnum = pEnumIL;
    } else if !pEnumIL.is_null() {
        ((*(*pEnumIL).lpVtbl).Release)(pEnumIL);
    }

    if !psfChild.is_null() {
        ((*(*psfChild).lpVtbl).Release)(psfChild);
    }

    S_OK
}
unsafe extern "C" fn BrFolder_InitTreeView(info: *mut BrFolder) {
    let mut hImageList: HIMAGELIST = core::ptr::null_mut();
    let hr: HRESULT;
    let hItem: HTREEITEM;

    Shell_GetImageLists(core::ptr::null_mut(), &mut hImageList);
    if !hImageList.is_null() {
        TreeView_SetImageList((*info).hwndTreeView, hImageList, 0);
    }

    let pidlRoot: PCIDLIST_ABSOLUTE = (*(*info).lpBrowseInfo).pidlRoot;
    let mut pidlParent = CComHeapPtr::<ITEMIDLIST_ABSOLUTE>::from_raw(ILClone(pidlRoot).cast());
    ILRemoveLastID(pidlParent.m_pData);
    let pidlChild: PCIDLIST_RELATIVE = ILFindLastID(pidlRoot);

    let mut lpsfParent = CComPtr::<IShellFolder>::new();
    hr = SHBindToObject(core::ptr::null_mut(), pidlParent.m_pData, (core::ptr::addr_of!(IID_IShellFolder)).cast(), core::ptr::addr_of_mut!(lpsfParent).cast());
    if FAILED_UNEXPECTEDLY!(hr) {
        return;
    }

    TreeView_DeleteItem((*info).hwndTreeView, TVI_ROOT);
    hItem = BrFolder_InsertItem(info, lpsfParent.p, pidlChild, pidlParent.m_pData, TVI_ROOT);
    TreeView_Expand((*info).hwndTreeView, hItem, TVE_EXPAND);
}
unsafe extern "C" fn BrFolder_GetIconPair(
    item: *mut BrItemData,
    tvi: *mut TVITEMW,
) -> HRESULT {
    let mut open: i32 = -( 1 as i32 );
    let normal = SHMapPIDLToSystemImageListIndex(
        ((*item).lpsfParent.Get()),
        (*item).pidlChild,
        &mut open,
    );
    if normal >= 0 as i32 {
        (*tvi).iImage = normal;
        (*tvi).iSelectedImage = if open >= 0 as i32 { open } else { normal };
        return S_OK;
    }
    let index = Shell_GetCachedImageIndexW(
        swShell32Name.as_ptr(),
        (IDI_SHELL_FOLDER - 1) as i32,
        0,
    );
    (*tvi).iImage = index;
    (*tvi).iSelectedImage = index;
    S_FALSE
}
/// The `(BrFolder*, HTREEITEM, ...)` OVERLOAD of `BrFolder_GetName`. C++ has two; Rust has no
/// overloading, so the translation kept only the `(IShellFolder*, PIDL, ...)` one and left its
/// callers silently pointing at the wrong signature. Resolve the item, then delegate.
unsafe extern "C" fn BrFolder_GetNameOfItem(
    info: *mut BrFolder,
    hItem: HTREEITEM,
    Flags: u32,
    Buffer: *mut u16,
) -> i32 {
    let item = BrFolder_GetItemData(info, hItem);
    if item.is_null() {
        return FALSE;
    }
    BrFolder_GetName(
        (*item).lpsfParent.Get(),
        (*item).pidlChild,
        Flags,
        Buffer,
    )
}
unsafe extern "C" fn BrFolder_GetName(
    lpsf: *mut IShellFolder,
    pidlChild: PCIDLIST_RELATIVE,
    dwFlags: u32,
    lpFriendlyName: *mut u16,
) -> i32 {
    let mut bSuccess: i32 = FALSE;
    let mut str: STRRET = core::mem::zeroed();
    if !FAILED_UNEXPECTEDLY!(((*(*lpsf).lpVtbl).GetDisplayNameOf)(lpsf, pidlChild.cast(), dwFlags, core::ptr::addr_of_mut!(str).cast())) {
        bSuccess = StrRetToStrNW(lpFriendlyName, (MAX_PATH) as u32, (&raw mut str).cast(), (pidlChild).cast());
    }
    bSuccess
}
unsafe extern "C" fn BrFolder_IsTreeItemInEnum(
    info: *mut BrFolder,
    hItem: HTREEITEM,
    pEnum: *mut IEnumIDList,
) -> BOOL {
    let pItemData = BrFolder_GetItemData(info, hItem);
    if pItemData.is_null() {
        return FALSE;
    }

    ((*(*pEnum).lpVtbl).Reset)(pEnum);

    let mut pidlTemp: *mut ITEMIDLIST_RELATIVE = core::ptr::null_mut();
    loop {
        let hr = ((*(*pEnum).lpVtbl).Next)(
            pEnum,
            1,
            core::ptr::addr_of_mut!(pidlTemp).cast(),
            core::ptr::null_mut(),
        );
        if hr != S_OK {
            break;
        }
        if ILIsEqual(pidlTemp, (*pItemData).pidlChild) != 0 {
            return TRUE;
        }
        ILFree(pidlTemp);
    }

    FALSE
}
unsafe extern "C" fn BrFolder_TreeItemHasThisChild(
    info: *mut BrFolder,
    hItem: HTREEITEM,
    pidlChild: PITEMID_CHILD,
) -> i32 {
    let mut hItem = hItem;
    hItem = TreeView_GetChild((*info).hwndTreeView, hItem);
    while !hItem.is_null() {
        let pItemData = BrFolder_GetItemData(info, hItem);
        if ILIsEqual((*pItemData).pidlChild, pidlChild) != 0 {
            return TRUE;
        }
        hItem = TreeView_GetNextSibling((*info).hwndTreeView, hItem);
    }
    FALSE
}
unsafe extern "C" fn BrFolder_FindTreeItemOfAbsoluteItem(
    info: *mut BrFolder,
    pidl: PCIDLIST_ABSOLUTE,
    mut hItem: HTREEITEM,
) -> HTREEITEM {
    if hItem.is_null() {
        hItem = TreeView_GetRoot((*info).hwndTreeView);
    }
    while !pidl.is_null() && !hItem.is_null() {
        let pItemData = BrFolder_GetItemData(info, hItem);
        let pidlFull = (*pItemData).pidlFull.as_ptr();
        if ILIsEqual(pidlFull, pidl) != 0 {
            return hItem;
        }
        let hChild = TreeView_GetChild((*info).hwndTreeView, hItem);
        let mut temp: HTREEITEM = core::ptr::null_mut();
        if !hChild.is_null() {
            temp = BrFolder_FindTreeItemOfAbsoluteItem(info, pidl, hChild);
        }
        if !hChild.is_null() && !temp.is_null() {
            return temp;
        }
        hItem = TreeView_GetNextSibling((*info).hwndTreeView, hItem);
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn BrFolder_UpdateItemEx(
    info: *mut BrFolder,
    hItem: HTREEITEM,
    pidlFull: PCIDLIST_ABSOLUTE,
    flags: u32,
) -> BOOL {
    let _ = TVIF_TEXT | TVIF_IMAGE | TVIF_CHILDREN;
    let mut item = TVITEMW {
        mask: TVIF_HANDLE | flags,
        hItem,
        ..core::mem::zeroed()
    };
    let data = BrFolder_GetItemData(info, hItem);
    if data.is_null() {
        return FALSE;
    }

    if !pidlFull.is_null() {
        let pidlFull_clone = ILClone(pidlFull);
        if pidlFull_clone.is_null() {
            return FALSE;
        }
        let pidlFull_mut = pidlFull_clone as PIDLIST_ABSOLUTE;
        (*data).pidlFull.Free();
        (*data).pidlFull.Attach(pidlFull_mut);
        (*data).pidlChild = ILFindLastID((*data).pidlFull.as_ptr());
    }

    let mut name: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if (flags & TVIF_TEXT) != 0 {
        item.pszText = name.as_mut_ptr();
        if BrFolder_GetName(
            (*data).lpsfParent.Get(),
            (*data).pidlChild,
            SHGDN_TREE,
            name.as_mut_ptr(),
        ) == 0
        {
            return FALSE;
        }
    }

    if (flags & TVIF_IMAGE) != 0 {
        item.mask |= TVIF_SELECTEDIMAGE;
        BrFolder_GetIconPair(data, core::ptr::addr_of_mut!(item));
    }

    item.cChildren = 0;
    let mut pEnum: CComPtr<IEnumIDList> = CComPtr::new();
    if (flags & TVIF_CHILDREN) != 0
        && BrFolder_GetChildrenEnum(info, data, pEnum.GetAddressOf()) == S_OK
    {
        let mut pidlTemp: CComHeapPtr<ITEMIDLIST_RELATIVE> = CComHeapPtr::new();
        let pe = pEnum.Get();
        if ((*(*pe).lpVtbl).Next)(
            pe,
            1,
            (&mut pidlTemp as *mut _ as *mut *mut ITEMIDLIST_RELATIVE).cast(),
            core::ptr::null_mut(),
        ) == S_OK
        {
            item.cChildren += 1;
        }
    }

    TreeView_SetItem((*info).hwndTreeView, &item)
}
unsafe extern "C" fn BrFolder_UpdateItem(info: *mut BrFolder, hItem: HTREEITEM) {
    BrFolder_UpdateItemEx(info, hItem, core::ptr::null_mut(), TVIF_IMAGE | TVIF_CHILDREN);
}
unsafe extern "C" fn BrFolder_InsertItem(
    info: *mut BrFolder,
    lpsf: *mut IShellFolder,
    pidlChild: PCUITEMID_CHILD,
    pidlParent: PCIDLIST_ABSOLUTE,
    hParent: HTREEITEM,
) -> HTREEITEM {
    if (BrowseFlagsToSHCONTF((*(*info).lpBrowseInfo).ulFlags) & SHCONTF_NONFOLDERS) == 0 {
        if (SHGetAttributes(lpsf, pidlChild, SFGAO_STREAM) & SFGAO_STREAM) != 0 {
            return core::ptr::null_mut();
        }
    }

    let mut szName: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    if BrFolder_GetName(lpsf, pidlChild, SHGDN_TREE, szName.as_mut_ptr()) == 0 {
        return core::ptr::null_mut();
    }

    // C++ `new BrItemData()` value-initialises: the PODs zero and the two smart pointers
    // default-construct to null. `atl.rs` documents both as zeroable and transparent, so a
    // zeroed block IS that object.
    let pItemData: *mut BrItemData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<BrItemData>() as SIZE_T,
    ) as *mut BrItemData;
    if pItemData.is_null() {
        return core::ptr::null_mut();
    }
    let mut item: TVITEMW = unsafe { core::mem::zeroed() };
    item.mask = TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_PARAM | TVIF_CHILDREN;
    item.pszText = szName.as_mut_ptr();
    item.cchTextMax = _countof!(szName) as i32;
    item.lParam = pItemData as LPARAM;

    let pidlFull = if !pidlParent.is_null() {
        ILCombine(pidlParent, pidlChild)
    } else {
        ILClone(pidlChild)
    };

    (*pItemData).lpsfParent = CComPtr { p: lpsf };
    ((*pItemData).pidlFull).Attach(pidlFull);
    (*pItemData).pidlChild = ILFindLastID((*pItemData).pidlFull.as_ptr());
    BrFolder_GetIconPair(pItemData, core::ptr::addr_of_mut!(item));

    if BrFolder_GetChildrenEnum(info, pItemData, core::ptr::null_mut()) == S_OK {
        item.cChildren = 1;
    }

    let mut tvins: TVINSERTSTRUCTW = unsafe { core::mem::zeroed() };
    tvins.hParent = hParent;
    // bindgen renders the TVINSERTSTRUCT tail as an anonymous UNION of itemex/item.
    *tvins.__bindgen_anon_1.item.as_mut() = item;
    TreeView_InsertItem((*info).hwndTreeView, &raw const tvins)
}
unsafe extern "C" fn BrFolder_Expand(
    info: *mut BrFolder,
    lpsf: *mut IShellFolder,
    pidlFull: PCIDLIST_ABSOLUTE,
    hParent: HTREEITEM,
) {
    let pItemData = BrFolder_GetItemData(info, hParent);
    let mut pEnum: CComPtr<IEnumIDList> = core::mem::zeroed();
    let hr: HRESULT = BrFolder_GetChildrenEnum(info, pItemData, pEnum.GetAddressOf());
    if hr < 0 {
        return;
    }

    SetCapture((*info).hWnd);
    SetCursor(LoadCursorW(
        core::ptr::null_mut(),
        IDC_WAIT as LPWSTR,
    ));

    let mut pidlTemp: CComHeapPtr<ITEMIDLIST_RELATIVE> = core::mem::zeroed();
    let mut ulFetched: ULONG = 0;
    while S_OK
        == ((*(**pEnum).lpVtbl).Next)(
            *pEnum,
            1,
            core::ptr::addr_of_mut!(pidlTemp).cast(),
            &mut ulFetched,
        )
    {
        BrFolder_InsertItem(info, lpsf, pidlTemp.as_ptr(), pidlFull, hParent);
        pidlTemp.Free();
    }

    ReleaseCapture();
    SetCursor(LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    ));
}
unsafe extern "C" fn PIDLIsType(pidl: LPCITEMIDLIST, type_: PIDLTYPE) -> BOOL {
    let data = _ILGetDataPointer((pidl).cast()) as *mut PIDLDATA;
    if data.is_null() {
        return FALSE;
    }
    if (*data).type_ == type_ {
        return 1;
    }
    0
}
unsafe extern "C" fn BrFolder_CheckValidSelection(info: *mut BrFolder, pItemData: *mut BrItemData) {
    let lpBrowseInfo = (*info).lpBrowseInfo;
    let pidlChild = (*pItemData).pidlChild;
    let mut dwAttributes: u32;
    let mut hr: HRESULT;

    let mut bEnabled = TRUE;
    if ((*lpBrowseInfo).ulFlags & BIF_BROWSEFORCOMPUTER) != 0 && PIDLIsType(pidlChild, (PT_COMP) as u8) == 0 {
        bEnabled = FALSE;
    }

    if ((*lpBrowseInfo).ulFlags & BIF_RETURNFSANCESTORS) != 0 {
        dwAttributes = SFGAO_FILESYSANCESTOR | SFGAO_FILESYSTEM;
        let sf = (*pItemData).lpsfParent.Get();
        hr = ((*(*sf).lpVtbl).GetAttributesOf)(sf, 1, core::ptr::addr_of!(pidlChild).cast(), &mut dwAttributes as *mut u32);
        if hr < 0 || (dwAttributes & (SFGAO_FILESYSANCESTOR | SFGAO_FILESYSTEM)) == 0 {
            bEnabled = FALSE;
        }
    }

    dwAttributes = SFGAO_FOLDER | SFGAO_FILESYSTEM;
    let sf = (*pItemData).lpsfParent.Get();
    hr = ((*(*sf).lpVtbl).GetAttributesOf)(sf, 1, core::ptr::addr_of!(pidlChild).cast(), &mut dwAttributes as *mut u32);
    if FAILED_UNEXPECTEDLY!(hr) || ((dwAttributes & (SFGAO_FOLDER | SFGAO_FILESYSTEM)) != (SFGAO_FOLDER | SFGAO_FILESYSTEM)) {
        if ((*lpBrowseInfo).ulFlags & BIF_RETURNONLYFSDIRS) != 0 {
            bEnabled = FALSE;
        }
        EnableWindow(GetDlgItem((*info).hWnd, (IDC_BROWSE_FOR_FOLDER_NEW_FOLDER) as i32), FALSE);
    } else {
        EnableWindow(GetDlgItem((*info).hWnd, (IDC_BROWSE_FOR_FOLDER_NEW_FOLDER) as i32), TRUE);
    }

    SendMessageW((*info).hWnd, BFFM_ENABLEOK, 0, (bEnabled) as LPARAM);
}
unsafe extern "C" fn BrFolder_Treeview_Delete(
    info: *mut BrFolder,
    pnmtv: *mut NMTREEVIEWW,
) -> LRESULT {
    let pItemData = (*pnmtv).itemOld.lParam as *mut BrItemData;
    if !pItemData.is_null() {
        // Destructor for CComPtr<IShellFolder> lpsfParent – call Release
        let lpsf = (*pItemData).lpsfParent.Get();
        if !lpsf.is_null() {
            ((*(*lpsf).lpVtbl).Release)(lpsf);
        }
        // Destructor for CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFull – free the PIDL
        let pidl = (*pItemData).pidlFull.as_ptr();
        if !pidl.is_null() {
            ILFree(pidl as PIDLIST_ABSOLUTE);
        }
        // Free the BrItemData memory (operator delete -> free)
        free(pItemData as *mut core::ffi::c_void);
    }
    return 0;
}
unsafe extern "C" fn BrFolder_Treeview_Expand(info: *mut BrFolder, pnmtv: *mut NMTREEVIEWW) -> LRESULT {
    let pItemData = (*pnmtv).itemNew.lParam as *mut BrItemData;

    if ((*pnmtv).itemNew.state & TVIS_EXPANDEDONCE) != 0 {
        return 0;
    }

    let mut hr = S_OK;
    let mut lpsf2: *mut IShellFolder = core::ptr::null_mut();
    if _ILIsEmpty((*pItemData).pidlChild) == 0 {
        let sfp = (*pItemData).lpsfParent.Get();
        hr = ((*(*sfp).lpVtbl).BindToObject)(
            sfp,
            (*pItemData).pidlChild.cast(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder).cast(),
            &mut lpsf2 as *mut *mut IShellFolder as *mut *mut core::ffi::c_void,
        );
    } else {
        lpsf2 = (*pItemData).lpsfParent.Get();
        if !lpsf2.is_null() {
            ((*(*lpsf2).lpVtbl).AddRef)(lpsf2);
        }
    }

    let hItem = (*pnmtv).itemNew.hItem;
    if !FAILED_UNEXPECTEDLY!(hr) {
        BrFolder_Expand(info, lpsf2, (*pItemData).pidlFull.as_ptr(), hItem);
    }

    if _ILIsMyComputer(((*pItemData).pidlChild).cast()) == 0 {
        TreeView_SortChildren((*info).hwndTreeView, hItem, FALSE);
    }

    if !lpsf2.is_null() {
        ((*(*lpsf2).lpVtbl).Release)(lpsf2);
    }

    0
}
unsafe extern "C" fn BrFolder_Treeview_Changed(info: *mut BrFolder, pnmtv: *mut NMTREEVIEWW) -> HRESULT {
    let pItemData = (*pnmtv).itemNew.lParam as *mut BrItemData;

    ILFree((*info).pidlRet);
    (*info).pidlRet = ILClone((*pItemData).pidlFull.as_ptr());

    let mut szName: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    if BrFolder_GetName(
        (*pItemData).lpsfParent.Get(),
        (*pItemData).pidlChild,
        SHGDN_NORMAL | SHGDN_INFOLDER,
        szName.as_mut_ptr(),
    ) != 0
    {
        SetDlgItemTextW((*info).hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT) as i32, szName.as_mut_ptr());
    }

    BrFolder_Callback(
        (*info).lpBrowseInfo,
        (*info).hWnd,
        BFFM_SELCHANGED,
        (*info).pidlRet as isize as LPARAM,
    );
    BrFolder_CheckValidSelection(info, pItemData);

    S_OK
}
unsafe extern "C" fn BrFolder_Treeview_Rename(info: *mut BrFolder, pnmtv: *mut NMTVDISPINFOW) -> LRESULT {
    if (*pnmtv).item.pszText.is_null() {
        return (FALSE) as LRESULT;
    }

    let hItem = TreeView_GetSelection((*info).hwndTreeView);
    let data = BrFolder_GetItemData(info, hItem);

    let mut newChild: PITEMID_CHILD = core::ptr::null_mut();
    let hr = ((*(*(*data).lpsfParent.p).lpVtbl).SetNameOf)((*data).lpsfParent.p,
        (*info).hWnd.cast(),
        (*data).pidlChild.cast(),
        (*pnmtv).item.pszText,
        SHGDN_NORMAL | SHGDN_INFOLDER,
        core::ptr::addr_of_mut!(newChild).cast(),
    );
    if hr < 0 {
        return (FALSE) as LRESULT;
    }

    let mut newFull: LPITEMIDLIST = core::ptr::null_mut();
    let hr = SHILClone((*data).pidlFull.as_ptr(), &mut newFull);
    if hr >= 0 {
        ILRemoveLastID(newFull);
        let hr = SHILAppend(newChild, &mut newFull);
        if hr >= 0 {
            (*data).pidlFull.Free();
            (*data).pidlFull.Attach(newFull);
            (*data).pidlChild = ILFindLastID((*data).pidlFull.as_ptr());
        }
        newChild = core::ptr::null_mut();
    }
    ILFree(newChild);

    // C's `!` on a BOOL is LOGICAL negation; Rust's `!` on an i32 is BITWISE (!0 == -1), which
    // compiles clean and is always truthy. Spell the comparison out.
    let AllowTreeTextChange: BOOL =
        (BrFolder_UpdateItemEx(info, hItem, core::ptr::null_mut(), TVIF_TEXT) == 0) as BOOL;

    let mut nmtv: NMTREEVIEWW = core::mem::zeroed();
    nmtv.itemNew.hItem = hItem;
    nmtv.itemNew.mask = TVIF_PARAM;
    nmtv.itemNew.lParam = data as LPARAM;
    BrFolder_Treeview_Changed(info, &mut nmtv);
    (AllowTreeTextChange) as LRESULT
}

unsafe extern "C" fn BrFolder_Rename(info: *mut BrFolder, hItem: HTREEITEM) -> HRESULT {
    TreeView_SelectItem((*info).hwndTreeView, hItem);
    TreeView_EditLabel((*info).hwndTreeView, hItem);
    S_OK
}
unsafe extern "C" fn BrFolder_Delete(info: *mut BrFolder, hItem: HTREEITEM) {
    let data = BrFolder_GetItemData(info, hItem);
    if data.is_null() || BrFolder_GetItemAttributes(info, hItem, SFGAO_CANDELETE) == 0 {
        return;
    }
    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let sf = (*data).lpsfParent.Get();
    let hr = ((*(*sf).lpVtbl).GetUIObjectOf)(
        sf,
        ((*info).hWnd).cast(),
        1,
        &raw const (*data).pidlChild as *const _,
        core::ptr::addr_of!(IID_IContextMenu).cast(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(pCM).cast(),
    );
    if hr < 0 {
        if !pCM.is_null() {
            ((*(*pCM).lpVtbl).Release)(pCM);
        }
        return;
    }
    let mut fCMIC: u32 = 0;
    if GetKeyState((VK_SHIFT) as i32) < 0 {
        fCMIC |= CMIC_MASK_SHIFT_DOWN;
    }
    if GetKeyState((VK_CONTROL) as i32) < 0 {
        fCMIC |= CMIC_MASK_CONTROL_DOWN;
    }
    SHInvokeCommandOnContextMenu(
        (*info).hWnd,
        core::ptr::null_mut(),
        pCM.cast(),
        fCMIC,
        b"delete\0".as_ptr() as *const i8,
    );
    if !pCM.is_null() {
        ((*(*pCM).lpVtbl).Release)(pCM);
    }
}
unsafe extern "C" fn BrFolder_Treeview_Keydown(info: *mut BrFolder, keydown: *mut NMTVKEYDOWN) -> LRESULT {
    if ((*(*info).lpBrowseInfo).ulFlags & BIF_USENEWUI) == 0 {
        return 0;
    }
    let hItem = TreeView_GetSelection((*info).hwndTreeView);
    // `wVKey` is a WORD; the VK_* island constants are u32. Widen the scrutinee, not the arms.
    match (*keydown).wVKey as u32 {
        VK_F2 => {
            BrFolder_Rename(info, hItem);
        }
        VK_DELETE => {
            BrFolder_Delete(info, hItem);
        }
        0x52 /* b'R' */ => {
            if GetKeyState((VK_CONTROL) as i32) < 0 {
                BrFolder_Refresh(info);
            }
        }
        VK_F5 => {
            BrFolder_Refresh(info);
        }
        _ => {}
    }
    0
}
unsafe extern "C" fn BrFolder_OnNotify(
    info: *mut BrFolder,
    CtlID: u32,
    lpnmh: *mut NMHDR,
) -> LRESULT {
    let pnmtv: *mut NMTREEVIEWW = lpnmh as *mut NMTREEVIEWW;

    if (*pnmtv).hdr.idFrom != (IDC_BROWSE_FOR_FOLDER_TREEVIEW) as UINT_PTR {
        return 0;
    }

    // `hdr.code` is UINT; the TVN_* island constants are i32 (they are TVN_FIRST-relative, i.e.
    // negative). Cast the SCRUTINEE once — a cast is not legal in a match pattern.
    match (*pnmtv).hdr.code as i32 {
        TVN_DELETEITEMA | TVN_DELETEITEMW => {
            return BrFolder_Treeview_Delete(info, pnmtv);
        }
        TVN_ITEMEXPANDINGA | TVN_ITEMEXPANDINGW => {
            return BrFolder_Treeview_Expand(info, pnmtv);
        }
        TVN_SELCHANGEDA | TVN_SELCHANGEDW => {
            return (BrFolder_Treeview_Changed(info, pnmtv)) as LRESULT;
        }
        TVN_BEGINLABELEDITA | TVN_BEGINLABELEDITW => {
            let hWndEdit = TreeView_GetEditControl((*lpnmh).hwndFrom);
            let tvdi: &mut NMTVDISPINFO = &mut *(lpnmh as *mut NMTVDISPINFO);
            let att = BrFolder_GetItemAttributes(info, tvdi.item.hItem, SFGAO_CANRENAME);
            if att & SFGAO_CANRENAME == 0 {
                MessageBeep(0xffffffff);
                return (S_FALSE) as LRESULT;
            }
            let mut Name: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
            if BrFolder_GetNameOfItem(
                info,
                tvdi.item.hItem,
                SHGDN_INFOLDER | SHGDN_FOREDITING,
                Name.as_mut_ptr(),
            ) != 0
            {
                SetWindowTextW(hWndEdit, Name.as_ptr());
            }
            return (S_OK) as LRESULT;
        }
        TVN_ENDLABELEDITW => {
            return BrFolder_Treeview_Rename(info, pnmtv as *mut NMTVDISPINFOW);
        }
        TVN_KEYDOWN => {
            return BrFolder_Treeview_Keydown(info, pnmtv as *mut NMTVKEYDOWN);
        }
        _ => {
            WARN!("unhandled ({})\n", (*pnmtv).hdr.code);
        }
    }

    return 0;
}
unsafe extern "C" fn BrFolder_OnInitDialog(hWnd: HWND, info: *mut BrFolder) -> BOOL {
    let mut pidlDesktop: CComHeapPtr<ITEMIDLIST_ABSOLUTE> = CComHeapPtr::new();
    let mut ntreg: SHChangeNotifyEntry = core::mem::zeroed();
    let lpBrowseInfo: *mut BROWSEINFOW = (*info).lpBrowseInfo;

    (*info).hWnd = hWnd;
    SetWindowLongPtrW(hWnd, DWLP_USER, info as LONG_PTR);

    if ((*lpBrowseInfo).ulFlags & BIF_NEWDIALOGSTYLE) != 0 {
        FIXME!("flags BIF_NEWDIALOGSTYLE partially implemented\n");
    }

    if ((*lpBrowseInfo).ulFlags & !SUPPORTED_FLAGS) != 0 {
        FIXME!("flags %x not implemented\n", (*lpBrowseInfo).ulFlags & !SUPPORTED_FLAGS);
    }

    (*info).layout = core::ptr::null_mut();
    if ((*lpBrowseInfo).ulFlags & BIF_USENEWUI) != 0 {
        let mut rcWnd: RECT = core::mem::zeroed();

        if (((*lpBrowseInfo).ulFlags & BIF_NEWDIALOGSTYLE) != 0) &&
           (((*lpBrowseInfo).ulFlags & BIF_EDITBOX) == 0)
        {
            let mut rcEdit: RECT = core::mem::zeroed();
            let mut rcTreeView: RECT = core::mem::zeroed();
            GetWindowRect(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT) as i32), &mut rcEdit);
            GetWindowRect(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_TREEVIEW) as i32), &mut rcTreeView);
            let cy = rcTreeView.top - rcEdit.top;
            MapWindowPoints(core::ptr::null_mut(), hWnd, &mut rcTreeView as *mut RECT as *mut POINT, ((core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>()) as i32) as u32);
            rcTreeView.top -= cy;
            MoveWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_TREEVIEW) as i32),
                       rcTreeView.left, rcTreeView.top,
                       rcTreeView.right - rcTreeView.left,
                       rcTreeView.bottom - rcTreeView.top, TRUE);
        }

        if ((*lpBrowseInfo).ulFlags & BIF_NEWDIALOGSTYLE) != 0 {
            (*info).layout = LayoutInit(hWnd, g_layout_info.as_ptr(), (_countof!(g_layout_info)) as i32);
        }

        GetWindowRect(hWnd, &mut rcWnd);
        (*info).szMin.cx = rcWnd.right - rcWnd.left;
        (*info).szMin.cy = rcWnd.bottom - rcWnd.top;
    }

    if !(*lpBrowseInfo).lpszTitle.is_null() {
        SetDlgItemTextW(hWnd, (IDC_BROWSE_FOR_FOLDER_TITLE) as i32, (*lpBrowseInfo).lpszTitle);
    } else {
        ShowWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_TITLE) as i32), (SW_HIDE) as i32);
    }

    if (((*lpBrowseInfo).ulFlags & BIF_STATUSTEXT) == 0) || (((*lpBrowseInfo).ulFlags & BIF_USENEWUI) != 0) {
        ShowWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_STATUS) as i32), (SW_HIDE) as i32);
    }

    if (((*lpBrowseInfo).ulFlags & BIF_NONEWFOLDERBUTTON) != 0) ||
       (((*lpBrowseInfo).ulFlags & BIF_NEWDIALOGSTYLE) == 0)
    {
        ShowWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_NEW_FOLDER) as i32), (SW_HIDE) as i32);
    }

    if ((*lpBrowseInfo).ulFlags & BIF_EDITBOX) == 0 {
        ShowWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER) as i32), (SW_HIDE) as i32);
        ShowWindow(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT) as i32), (SW_HIDE) as i32);
    }

    (*info).hwndTreeView = GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_TREEVIEW) as i32);
    if !(*info).hwndTreeView.is_null() {
        BrFolder_InitTreeView(info);
    } else {
        ERR!("treeview control missing!\n");
    }

    SHGetFolderLocation(core::ptr::null_mut(), (CSIDL_DESKTOP) as i32, core::ptr::null_mut(), 0, pidlDesktop.put());

    ntreg.pidl = *pidlDesktop;
    ntreg.fRecursive = TRUE;
    (*info).hChangeNotify = SHChangeNotifyRegister(hWnd,
                                                 (SHCNRF_ShellLevel | SHCNRF_NewDelivery) as i32,
                                                 (SHCNE_ALLEVENTS) as i32,
                                                 SHV_CHANGE_NOTIFY, 1, &mut ntreg);

    if (*lpBrowseInfo).pidlRoot.is_null() {
        let csidl = if ((*lpBrowseInfo).ulFlags & BIF_NEWDIALOGSTYLE) != 0 { CSIDL_PERSONAL } else { CSIDL_DRIVES };
        let pidl = SHCloneSpecialIDList(core::ptr::null_mut(), (csidl) as i32, TRUE);
        if !pidl.is_null() {
            SendMessageW((*info).hWnd, BFFM_SETSELECTION, FALSE as WPARAM, pidl as LPARAM);
            if csidl == CSIDL_DRIVES {
                SendMessageW((*info).hWnd, BFFM_SETEXPANDED, FALSE as WPARAM, pidl as LPARAM);
            }
            ILFree(pidl);
        }
    } else {
        (*info).pidlRet = ILClone((*lpBrowseInfo).pidlRoot);
    }

    BrFolder_Callback((*info).lpBrowseInfo, hWnd, BFFM_INITIALIZED, 0);

    SHAutoComplete(GetDlgItem(hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT) as i32),
                   (SHACF_FILESYS_ONLY | SHACF_URLHISTORY | SHACF_FILESYSTEM));
    TRUE
}
unsafe extern "C" fn BrFolder_NewFolder(info: *mut BrFolder) -> HRESULT {
    let mut desktop: CComPtr<IShellFolder> = CComPtr::new();
    let mut cur: CComPtr<IShellFolder> = CComPtr::new();
    let mut wszNewFolder: [u16; 25] = [0; 25];
    let mut path: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut name: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];

    let mut hr = SHGetDesktopFolder(desktop.GetAddressOf());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    if !(*info).pidlRet.is_null() {
        let dsk = desktop.Get();
        hr = ((*(*dsk).lpVtbl).BindToObject)(
            dsk,
            (*info).pidlRet.cast(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder).cast(),
            (cur.GetAddressOf()).cast(),
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        hr = SHGetPathFromIDListW((*info).pidlRet, path.as_mut_ptr());
    } else {
        cur.Attach(desktop.Detach());
        hr = SHGetFolderPathW(
            core::ptr::null_mut(),
            (CSIDL_DESKTOPDIRECTORY) as i32,
            core::ptr::null_mut(),
            SHGFP_TYPE_CURRENT,
            path.as_mut_ptr(),
        );
    }

    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = E_FAIL;
    if LoadStringW(
        shell32_hInstance,
        IDS_NEWFOLDER,
        wszNewFolder.as_mut_ptr(),
        (_countof!(wszNewFolder)) as i32,
    ) == 0
    {
        return hr;
    }

    if PathYetAnotherMakeUniqueName(
        name.as_mut_ptr(),
        path.as_ptr(),
        core::ptr::null_mut(),
        wszNewFolder.as_ptr(),
    ) == 0
    {
        return hr;
    }

    let mut len = lstrlenW(path.as_ptr());
    if len < (MAX_PATH) as i32 && *name.as_ptr().offset(len as isize) == 0x005Cu16 {
        len += 1;
    }

    if CreateDirectoryW(name.as_ptr(), core::ptr::null_mut()) == 0 {
        return hr;
    }

    let hParent = TreeView_GetSelection((*info).hwndTreeView);
    if hParent.is_null() {
        return hr;
    }

    TreeView_Expand((*info).hwndTreeView, hParent, TVE_EXPAND);

    let mut item: TVITEMW = core::mem::zeroed();
    item.mask = TVIF_PARAM | TVIF_STATE;
    item.hItem = hParent;
    TreeView_GetItem((*info).hwndTreeView, &mut item);
    let item_data = item.lParam as *mut BrItemData;
    if item_data.is_null() {
        return hr;
    }

    if (item.state & TVIS_EXPANDEDONCE) == 0 {
        item.mask = TVIF_STATE;
        item.state = TVIS_EXPANDEDONCE;
        item.stateMask = TVIS_EXPANDEDONCE;
        TreeView_SetItem((*info).hwndTreeView, &mut item);
    }

    let mut pidlNew: CComHeapPtr<ITEMIDLIST_RELATIVE> = CComHeapPtr::new();
    let curp = cur.Get();
    hr = ((*(*curp).lpVtbl).ParseDisplayName)(
        curp,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        name.as_mut_ptr().offset(len as isize),
        core::ptr::null_mut(),
        (pidlNew.GetAddressOf()).cast(),
        core::ptr::null_mut(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let hAdded = BrFolder_InsertItem(info, cur.Get(), pidlNew.Get(), (*item_data).pidlFull.Get(), hParent);
    TreeView_SortChildren((*info).hwndTreeView, hParent, FALSE);
    BrFolder_Rename(info, hAdded)
}
unsafe extern "C" fn BrFolder_OnOK(info: *mut BrFolder) {
    let mut szPath: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    GetDlgItemTextW((*info).hWnd, (IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT) as i32, szPath.as_mut_ptr(), (_countof!(szPath)) as i32);
    StrTrimW(szPath.as_mut_ptr(), CPP_COM_WSTR_731D7EC48EEF.as_ptr());

    if PathIsRelativeW(szPath.as_mut_ptr()) == 0 && PathIsDirectoryW(szPath.as_mut_ptr()) != 0 {
        (*info).pidlRet = ILCreateFromPathW(szPath.as_mut_ptr());
    } else {
        (*info).pidlRet = ILClone((*info).pidlRet);
    }

    if (*info).pidlRet.is_null() {
        (*info).pidlRet = (_ILCreateDesktop()).cast();
    }

    let lpBrowseInfo = (*info).lpBrowseInfo;
    if (*lpBrowseInfo).pszDisplayName.is_null() {
        return;
    }

    let mut fileInfo: SHFILEINFOW = core::mem::zeroed();
    (*lpBrowseInfo).pszDisplayName.add(0).write(UNICODE_NULL);
    if SHGetFileInfoW(
        (*info).pidlRet as *const u16,
        0,
        &raw mut fileInfo,
        (core::mem::size_of::<SHFILEINFOW>()) as u32,
        SHGFI_PIDL | SHGFI_DISPLAYNAME,
    ) != 0
    {
        crate::island::lstrcpynW((*lpBrowseInfo).pszDisplayName, core::ptr::addr_of!(fileInfo.szDisplayName).cast(), (MAX_PATH) as i32);
    }
}
unsafe extern "C" fn BrFolder_OnCommand(info: *mut BrFolder, id: u32) {
    match id {
        IDOK => {
            BrFolder_OnOK(info);
            EndDialog((*info).hWnd, (IDOK) as INT_PTR);
        }
        IDCANCEL => {
            EndDialog((*info).hWnd, (IDCANCEL) as INT_PTR);
        }
        IDC_BROWSE_FOR_FOLDER_NEW_FOLDER => {
            BrFolder_NewFolder(info);
        }
        _ => {}
    }
}
unsafe extern "C" fn GetTreeViewItemContextMenuPos(hWnd: HWND, hItem: HTREEITEM, ppt: *mut POINT) {
    let mut rc: RECT = core::mem::zeroed();
    if TreeView_GetItemRect(hWnd, hItem, &mut rc, TRUE) != 0 {
        (*ppt).x = (rc.left + rc.right) / 2;
        (*ppt).y = (rc.top + rc.bottom) / 2;
    }
    ClientToScreen(hWnd, ppt);
}
unsafe extern "C" fn BrFolder_OnContextMenu(info: *mut BrFolder, lParam: LPARAM) {
    // C anonymous enum: successive values, so ID_FIRSTCMD is IDC_TOGGLE + 1.
    const IDC_TOGGLE: UINT = 1;
    const ID_FIRSTCMD: UINT = 2;
    const ID_LASTCMD: UINT = 0xffff;
    let mut hSelected = TreeView_GetSelection((*info).hwndTreeView);
    let mut ici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
    ici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32;
    ici.fMask = CMIC_MASK_PTINVOKE;
    ici.hwnd = (*info).hWnd;
    ici.nShow = (SW_SHOW) as i32;
    ici.ptInvoke.x = GET_X_LPARAM(lParam);
    ici.ptInvoke.y = GET_Y_LPARAM(lParam);
    if lParam as i32 == -1 {
        GetTreeViewItemContextMenuPos((*info).hwndTreeView, hSelected, core::ptr::addr_of_mut!(ici.ptInvoke));
    } else {
        let mut hti: TVHITTESTINFO = core::mem::zeroed();
        hti.pt = ici.ptInvoke;
        ScreenToClient((*info).hwndTreeView, &mut hti.pt);
        hSelected = TreeView_HitTest((*info).hwndTreeView, &mut hti);
    }
    let item = BrFolder_GetItemData(info, hSelected);
    if item.is_null() {
        return;
    }

    let mut tvi: TV_ITEM = core::mem::zeroed();
    tvi.mask = TVIF_STATE | TVIF_CHILDREN;
    tvi.stateMask = TVIS_EXPANDED;
    tvi.hItem = hSelected;
    TreeView_GetItem((*info).hwndTreeView, &mut tvi);

    let mut pcm: *mut IContextMenu = core::ptr::null_mut();
    let isf = (*item).lpsfParent.Get();
    let hr = ((*(*isf).lpVtbl).GetUIObjectOf)(
        isf,
        (*info).hWnd.cast(),
        1,
        core::ptr::addr_of!((*item).pidlChild).cast(),
        core::ptr::addr_of!(IID_IContextMenu).cast(),
        core::ptr::null_mut(),
        &mut pcm as *mut *mut IContextMenu as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return;
    }

    let hMenu = CreatePopupMenu();
    if hMenu.is_null() {
        return;
    }
    (*info).pContextMenu = pcm;
    let cmf = (if GetKeyState((VK_SHIFT) as i32) < 0 {
        CMF_EXTENDEDVERBS
    } else {
        0
    }) | CMF_CANRENAME;
    let hr = ((*(*pcm).lpVtbl).QueryContextMenu)(
        pcm,
        hMenu.cast(),
        0,
        ID_FIRSTCMD,
        ID_LASTCMD,
        CMF_EXPLORE | cmf,
    );
    if hr > 0 {
        _InsertMenuItemW(hMenu, 0, TRUE, 0, MFT_SEPARATOR, core::ptr::null_mut(), 0);
    }
    _InsertMenuItemW(
        hMenu,
        0,
        TRUE,
        IDC_TOGGLE,
        MFT_STRING,
        MAKEINTRESOURCEW(if (tvi.state & TVIS_EXPANDED) != 0 { IDS_COLLAPSE } else { IDS_EXPAND }),
        MFS_DEFAULT | if tvi.cChildren != 0 { 0 } else { MFS_GRAYED },
    );

    let cmd = TrackPopupMenuEx(hMenu, TPM_RETURNCMD, ici.ptInvoke.x, ici.ptInvoke.y, (*info).hWnd, core::ptr::null_mut());
    ici.lpVerb = MAKEINTRESOURCEA((cmd - (ID_FIRSTCMD) as i32) as u32);
    if cmd == (IDC_TOGGLE) as i32 {
        TreeView_SelectItem((*info).hwndTreeView, hSelected);
        TreeView_Expand((*info).hwndTreeView, hSelected, TVE_TOGGLE);
    } else if cmd != 0 && GetDfmCmd((pcm).cast(), ici.lpVerb) == DFM_CMD_RENAME {
        BrFolder_Rename(info, hSelected);
    } else if cmd != 0 {
        if GetKeyState((VK_SHIFT) as i32) < 0 {
            ici.fMask |= CMIC_MASK_SHIFT_DOWN;
        }
        if GetKeyState((VK_CONTROL) as i32) < 0 {
            ici.fMask |= CMIC_MASK_CONTROL_DOWN;
        }
        ((*(*pcm).lpVtbl).InvokeCommand)(pcm, core::ptr::addr_of!(ici) as *mut c_void);
    }
    (*info).pContextMenu = core::ptr::null_mut();
    DestroyMenu(hMenu);
}
unsafe extern "C" fn BrFolder_ExpandToPidl(info: *mut BrFolder, pidlSelection: LPITEMIDLIST, phItem: *mut HTREEITEM) -> i32 {
    let pidlCurrent: LPITEMIDLIST = pidlSelection;
    if _ILIsDesktop(pidlSelection) != 0 {
        if !phItem.is_null() {
            *phItem = TVI_ROOT;
        }
        return TRUE;
    }

    let mut item: TVITEMEXW = core::mem::zeroed();
    item.mask = TVIF_PARAM;
    item.hItem = TreeView_GetRoot((*info).hwndTreeView);
    if !item.hItem.is_null() {
        item.hItem = TreeView_GetChild((*info).hwndTreeView, item.hItem);
    }

    let mut depth: u32 = _ILGetDepth((*(*info).lpBrowseInfo).pidlRoot);
    let mut pidlCurrent = pidlCurrent;
    while !item.hItem.is_null() && !pidlCurrent.is_null() {
        depth = depth.wrapping_add(1);
        let pidlNeedle: LPITEMIDLIST = ILCloneToDepth(pidlSelection, depth);
        if _ILIsEmpty(pidlNeedle) != 0 {
            ILFree(pidlNeedle);
            item.hItem = core::ptr::null_mut();
            break;
        }
        loop {
            TreeView_GetItem((*info).hwndTreeView, &mut item);
            let pItemData: *const BrItemData = item.lParam as *const BrItemData;
            if ILIsEqual((*pItemData).pidlFull.m_pData as LPCITEMIDLIST, pidlNeedle) != 0 {
                let done: i32 = if _ILGetDepth(pidlSelection) == _ILGetDepth(pidlNeedle) { 1 } else { 0 };
                if done != 0 {
                    pidlCurrent = core::ptr::null_mut();
                } else {
                    TreeView_Expand((*info).hwndTreeView, item.hItem, TVE_EXPAND);
                    item.hItem = TreeView_GetChild((*info).hwndTreeView, item.hItem);
                }
                break;
            } else {
                item.hItem = TreeView_GetNextSibling((*info).hwndTreeView, item.hItem);
                if item.hItem.is_null() {
                    break;
                }
            }
        }
        ILFree(pidlNeedle);
    }

    if !phItem.is_null() {
        *phItem = item.hItem;
    }

    if _ILIsEmpty(pidlCurrent) != 0 && !item.hItem.is_null() { 1 } else { 0 }
}
unsafe extern "C" fn BrFolder_ExpandToString(info: *mut BrFolder, pszString: PWSTR, phItem: *mut HTREEITEM) -> i32 {
    let mut pidlSelection: *mut ITEMIDLIST_ABSOLUTE = core::ptr::null_mut();
    let hr = SHParseDisplayName(pszString, core::ptr::null_mut(), &mut pidlSelection, 0, core::ptr::null_mut());
    let ok = ((hr) >= 0) && BrFolder_ExpandToPidl(info, pidlSelection, phItem) != 0;
    if !pidlSelection.is_null() {
        CoTaskMemFree(pidlSelection as *mut core::ffi::c_void);
    }
    ok as i32
}
unsafe extern "C" fn BrFolder_OnSetExpanded(info: *mut BrFolder, pidlSelection: LPITEMIDLIST, pszString: PWSTR) -> i32 {
    let mut hItem: HTREEITEM = core::mem::zeroed();
    let ret: i32 = if !pszString.is_null() {
        BrFolder_ExpandToString(info, pszString, &mut hItem)
    } else {
        BrFolder_ExpandToPidl(info, pidlSelection, &mut hItem)
    };
    if ret != 0 {
        TreeView_Expand((*info).hwndTreeView, hItem, TVE_EXPAND);
    }
    ret
}
unsafe extern "C" fn BrFolder_OnSetSelectionPidl(info: *mut BrFolder, pidlSelection: LPITEMIDLIST) -> i32 {
    if pidlSelection.is_null() {
        return FALSE;
    }
    let mut hItem: HTREEITEM = core::ptr::null_mut();
    let ret = BrFolder_ExpandToPidl(info, pidlSelection, core::ptr::addr_of_mut!(hItem));
    if ret != 0 {
        TreeView_SelectItem((*info).hwndTreeView, hItem);
    }
    ret
}
unsafe extern "C" fn BrFolder_OnSetSelectionW(info: *mut BrFolder, pszSelection: PWSTR) -> BOOL {
    unsafe {
        if pszSelection.is_null() {
            return FALSE;
        }
        let mut hItem: HTREEITEM = core::ptr::null_mut();
        let ret: BOOL = BrFolder_ExpandToString(info, pszSelection, &mut hItem);
        if ret != 0 {
            TreeView_SelectItem((*info).hwndTreeView, hItem);
        }
        ret
    }
}
unsafe extern "C" fn BrFolder_OnSetSelectionA(info: *mut BrFolder, psz_selection_a: LPSTR) -> BOOL {
    if psz_selection_a.is_null() {
        return FALSE;
    }

    let mut psz_selection_w: PWSTR = core::ptr::null_mut();
    __SHCloneStrAtoW(
        core::ptr::addr_of_mut!(psz_selection_w),
        psz_selection_a,
    );
    if psz_selection_w.is_null() {
        return FALSE;
    }

    let ret = BrFolder_OnSetSelectionW(info, psz_selection_w);

    CoTaskMemFree(psz_selection_w as *mut core::ffi::c_void);

    ret
}
unsafe extern "C" fn BrFolder_OnDestroy(info: *mut BrFolder) {
    if !(*info).layout.is_null() {
        LayoutDestroy((*info).layout);
        (*info).layout = core::ptr::null_mut();
    }
    SHChangeNotifyDeregister((*info).hChangeNotify);
}
unsafe extern "C" fn BrFolder_RefreshRecurse(
    info: *mut BrFolder,
    hTarget: HTREEITEM,
) {
    let mut pEnum: CComPtr<IEnumIDList> = CComPtr::null();
    let pItemData = BrFolder_GetItemData(info, hTarget);
    let hrEnum = BrFolder_GetChildrenEnum(info, pItemData, pEnum.GetAddressOf());

    if hrEnum >= 0 {
        let mut pidlTemp: CComHeapPtr<ITEMIDLIST_RELATIVE> = core::mem::zeroed();
        while S_OK
            == ((*(**pEnum).lpVtbl).Next)(
                *pEnum,
                1,
                core::ptr::addr_of_mut!(pidlTemp).cast(),
                core::ptr::null_mut(),
            )
        {
            if BrFolder_TreeItemHasThisChild(info, hTarget, pidlTemp.Get()) == 0 {
                BrFolder_InsertItem(
                    info,
                    (*pItemData).lpsfParent.Get(),
                    pidlTemp.Get(),
                    (*pItemData).pidlFull.Get(),
                    hTarget,
                );
            }
            pidlTemp.Free();
        }
    }

    let mut hItem;
    let mut hNextItem;
    hItem = TreeView_GetChild((*info).hwndTreeView, hTarget);
    while !hItem.is_null() {
        hNextItem = TreeView_GetNextSibling((*info).hwndTreeView, hItem);
        if hrEnum < 0 || BrFolder_IsTreeItemInEnum(info, hItem, pEnum.Get()) == 0 {
            TreeView_DeleteItem((*info).hwndTreeView, hItem);
            hNextItem = TreeView_GetChild((*info).hwndTreeView, hTarget);
            hItem = hNextItem;
            continue;
        }
        BrFolder_UpdateItem(info, hItem);
        BrFolder_RefreshRecurse(info, hItem);
        hItem = hNextItem;
    }

    if hrEnum >= 0 {
        TreeView_SortChildren((*info).hwndTreeView, hTarget, FALSE);
    }

    pEnum.Release();
}
unsafe extern "C" fn BrFolder_Refresh(info: *mut BrFolder) {
    let hRoot = TreeView_GetRoot((*info).hwndTreeView);

    SendMessageW(
        (*info).hwndTreeView,
        WM_SETREDRAW,
        FALSE as WPARAM,
        0,
    );

    BrFolder_RefreshRecurse(info, hRoot);

    SendMessageW(
        (*info).hwndTreeView,
        WM_SETREDRAW,
        TRUE as WPARAM,
        0,
    );
    InvalidateRect((*info).hwndTreeView, core::ptr::null_mut(), TRUE);
}
unsafe extern "C" fn BrFolder_OnChangeEx(
    info: *mut BrFolder,
    pidl1: PCIDLIST_ABSOLUTE,
    pidl2: PCIDLIST_ABSOLUTE,
    event: i32,
) {
    // `event` is a C `int`; the SHCNE_* island constants are u32. C's usual arithmetic
    // conversions did this implicitly — do it once, on the scrutinee.
    match event as u32 {
        SHCNE_RENAMEFOLDER |
        SHCNE_RENAMEITEM |
        SHCNE_UPDATEITEM => {
            let UpdateFlags: u32 = if event == (SHCNE_UPDATEITEM) as i32 {
                TVIF_IMAGE | TVIF_CHILDREN
            } else {
                TVIF_TEXT
            };
            let mut hTI = BrFolder_FindTreeItemOfAbsoluteItem(info, pidl1, core::ptr::null_mut());
            if !hTI.is_null() {
                if BrFolder_UpdateItemEx(info, hTI, pidl2, UpdateFlags) != (0u8) as i32 {
                    hTI = TreeView_GetParent((*info).hwndTreeView, hTI);
                    if !hTI.is_null() {
                        TreeView_SortChildren((*info).hwndTreeView, hTI, FALSE);
                        return;
                    }
                }
            }
        }
        _ => {}
    }

    match event as u32 {
        SHCNE_DRIVEADD |
        SHCNE_MKDIR |
        SHCNE_CREATE |
        SHCNE_DRIVEREMOVED |
        SHCNE_RMDIR |
        SHCNE_DELETE |
        SHCNE_RENAMEFOLDER |
        SHCNE_RENAMEITEM |
        SHCNE_UPDATEDIR |
        SHCNE_UPDATEITEM => {
            // FIXME: Avoid full refresh and optimize for speed. Use pidl0 and pidl1
            BrFolder_Refresh(info);
        }
        _ => {}
    }
}
unsafe extern "C" fn BrFolder_OnChange(info: *mut BrFolder, wParam: WPARAM, lParam: LPARAM) {
    let hChange = wParam as HANDLE;
    let dwProcID = lParam as DWORD;

    let mut ppidl: *mut PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut event: i32 = 0;

    let hLock = SHChangeNotification_Lock(hChange, dwProcID, &mut ppidl, &mut event);
    if hLock.is_null() {
        return;
    }

    BrFolder_OnChangeEx(
        info,
        *ppidl.offset(0),
        *ppidl.offset(1),
        event & (!SHCNE_INTERRUPT) as i32,
    );

    SHChangeNotification_Unlock(hLock);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn BrFolderDlgProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    if uMsg == WM_INITDIALOG {
        return (BrFolder_OnInitDialog(hWnd, lParam as *mut BrFolder)) as INT_PTR;
    }

    let info = GetWindowLongPtrW(hWnd, DWLP_USER) as *mut BrFolder;
    if info.is_null() {
        return 0;
    }

    if !(*info).pContextMenu.is_null() && SHELL_IsContextMenuMsg(uMsg) != 0 {
        let mut result: LRESULT = 0;
        if SHForwardContextMenuMsg(
            (*info).pContextMenu.cast(),
            uMsg,
            wParam,
            lParam,
            &mut result,
            TRUE,
        ) == S_OK
        {
            SetWindowLongPtr(hWnd, DWLP_MSGRESULT as i32, result);
            return (TRUE) as INT_PTR;
        }
    }

    match uMsg {
        WM_NOTIFY => {
            SetWindowLongPtr(
                hWnd,
                DWLP_MSGRESULT as i32,
                (BrFolder_OnNotify(info, wParam as u32, lParam as *mut NMHDR)) as LONG_PTR,
            );
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            BrFolder_OnCommand(info, wParam);
        }
        WM_CONTEXTMENU => {
            if (*info).hwndTreeView == wParam as HWND {
                BrFolder_OnContextMenu(&mut (*info), lParam);
            }
        }
        WM_GETMINMAXINFO => {
            let minmax = lParam as *mut MINMAXINFO;
            (*minmax).ptMinTrackSize.x = (*info).szMin.cx;
            (*minmax).ptMinTrackSize.y = (*info).szMin.cy;
        }
        WM_SIZE => {
            if !(*info).layout.is_null() {
                LayoutUpdate(
                    hWnd,
                    (*info).layout,
                    (g_layout_info.as_ptr()).cast(),
                    _countof!(g_layout_info),
                );
            }
        }
        BFFM_SETSTATUSTEXTA => {
            SetDlgItemTextA(hWnd, IDC_BROWSE_FOR_FOLDER_STATUS as i32, lParam as LPSTR);
        }
        BFFM_SETSTATUSTEXTW => {
            SetDlgItemTextW(hWnd, IDC_BROWSE_FOR_FOLDER_STATUS as i32, lParam as LPWSTR);
        }
        BFFM_ENABLEOK => {
            EnableWindow(
                GetDlgItem(hWnd, IDOK as i32),
                (lParam != 0) as i32,
            );
        }
        BFFM_SETOKTEXT => {
            SetDlgItemTextW(hWnd, IDOK as i32, lParam as LPWSTR);
        }
        BFFM_SETSELECTIONA => {
            if wParam != 0 {
                return (BrFolder_OnSetSelectionA(info, lParam as LPSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetSelectionPidl(info, lParam as LPITEMIDLIST)) as INT_PTR;
            }
        }
        BFFM_SETSELECTIONW => {
            if wParam != 0 {
                return (BrFolder_OnSetSelectionW(info, lParam as LPWSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetSelectionPidl(info, lParam as LPITEMIDLIST)) as INT_PTR;
            }
        }
        BFFM_SETEXPANDED => {
            if wParam != 0 {
                return (BrFolder_OnSetExpanded(info, core::ptr::null_mut(), lParam as LPWSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetExpanded(info, lParam as LPITEMIDLIST, core::ptr::null_mut())) as INT_PTR;
            }
        }
        SHV_CHANGE_NOTIFY => {
            BrFolder_OnChange(info, wParam, lParam);
        }
        WM_DESTROY => {
            BrFolder_OnDestroy(info);
        }
        _ => {}
    }

    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn BrFolderDlgProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    if uMsg == WM_INITDIALOG {
        return (BrFolder_OnInitDialog(hWnd, lParam as *mut BrFolder)) as INT_PTR;
    }

    let info = GetWindowLongPtrW(hWnd, DWLP_USER) as *mut BrFolder;
    if info.is_null() {
        return 0;
    }

    if !(*info).pContextMenu.is_null() && SHELL_IsContextMenuMsg(uMsg) != 0 {
        let mut result: LRESULT = 0;
        if SHForwardContextMenuMsg(
            (*info).pContextMenu.cast(),
            uMsg,
            wParam,
            lParam,
            &mut result,
            TRUE,
        ) == S_OK
        {
            SetWindowLongPtr(hWnd, (DWLP_MSGRESULT) as i32, (result) as LONG_PTR);
            return (TRUE) as INT_PTR;
        }
    }

    match uMsg {
        WM_NOTIFY => {
            SetWindowLongPtr(
                hWnd,
                (DWLP_MSGRESULT) as i32,
                (BrFolder_OnNotify(info, wParam as u32, lParam as *mut NMHDR)) as LONG_PTR,
            );
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            BrFolder_OnCommand(info, (wParam) as u32);
        }
        WM_CONTEXTMENU => {
            if (*info).hwndTreeView == wParam as HWND {
                BrFolder_OnContextMenu(&mut (*info), lParam);
            }
        }
        WM_GETMINMAXINFO => {
            let minmax = lParam as *mut MINMAXINFO;
            (*minmax).ptMinTrackSize.x = (*info).szMin.cx;
            (*minmax).ptMinTrackSize.y = (*info).szMin.cy;
        }
        WM_SIZE => {
            if !(*info).layout.is_null() {
                LayoutUpdate(
                    hWnd,
                    (*info).layout,
                    (g_layout_info.as_ptr()).cast(),
                    _countof!(g_layout_info),
                );
            }
        }
        BFFM_SETSTATUSTEXTA => {
            SetDlgItemTextA(hWnd, (IDC_BROWSE_FOR_FOLDER_STATUS) as i32, lParam as LPSTR);
        }
        BFFM_SETSTATUSTEXTW => {
            SetDlgItemTextW(hWnd, (IDC_BROWSE_FOR_FOLDER_STATUS) as i32, lParam as LPWSTR);
        }
        BFFM_ENABLEOK => {
            EnableWindow(
                GetDlgItem(hWnd, (IDOK) as i32),
                (lParam != 0) as i32,
            );
        }
        BFFM_SETOKTEXT => {
            SetDlgItemTextW(hWnd, (IDOK) as i32, lParam as LPWSTR);
        }
        BFFM_SETSELECTIONA => {
            if wParam != 0 {
                return (BrFolder_OnSetSelectionA(info, lParam as LPSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetSelectionPidl(info, lParam as LPITEMIDLIST)) as INT_PTR;
            }
        }
        BFFM_SETSELECTIONW => {
            if wParam != 0 {
                return (BrFolder_OnSetSelectionW(info, lParam as LPWSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetSelectionPidl(info, lParam as LPITEMIDLIST)) as INT_PTR;
            }
        }
        BFFM_SETEXPANDED => {
            if wParam != 0 {
                return (BrFolder_OnSetExpanded(info, core::ptr::null_mut(), lParam as LPWSTR)) as INT_PTR;
            } else {
                return (BrFolder_OnSetExpanded(info, lParam as LPITEMIDLIST, core::ptr::null_mut())) as INT_PTR;
            }
        }
        SHV_CHANGE_NOTIFY => {
            BrFolder_OnChange(info, wParam, lParam);
        }
        WM_DESTROY => {
            BrFolder_OnDestroy(info);
        }
        _ => {}
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SHBrowseForFolderA(lpbi: LPBROWSEINFOA) -> LPITEMIDLIST {
    let mut bi: BROWSEINFOW = core::mem::zeroed();
    bi.hwndOwner = (*lpbi).hwndOwner;
    bi.pidlRoot = (*lpbi).pidlRoot;

    let mut szName: [u16; (MAX_PATH) as usize] = [0u16; (MAX_PATH) as usize];
    bi.pszDisplayName = if !(*lpbi).pszDisplayName.is_null() {
        szName.as_mut_ptr()
    } else {
        core::ptr::null_mut()
    };

    let mut pszTitle: CComHeapPtr<u16> = core::mem::zeroed();
    if !(*lpbi).lpszTitle.is_null() {
        __SHCloneStrAtoW((core::ptr::addr_of_mut!(pszTitle)).cast(), (*lpbi).lpszTitle);
    }
    bi.lpszTitle = pszTitle.Get();

    bi.ulFlags = (*lpbi).ulFlags;
    bi.lpfn = (*lpbi).lpfn;
    bi.lParam = (*lpbi).lParam;
    bi.iImage = (*lpbi).iImage;

    let pidl = SHBrowseForFolderW(&mut bi);

    if !bi.pszDisplayName.is_null() {
        SHUnicodeToAnsi(bi.pszDisplayName, (*lpbi).pszDisplayName, (MAX_PATH) as i32);
    }

    (*lpbi).iImage = bi.iImage;
    pidl
}

#[no_mangle]
pub unsafe extern "system" fn SHBrowseForFolderW(lpbi: LPBROWSEINFOW) -> LPITEMIDLIST {
    // MSDN says the caller must initialize COM; the C++ does it anyway via a COleInit RAII
    // local, and so must this — the guard must stay NAMED and live to the end of the fn.
    let _ole_init = COleInit::new();

    let mut info = BrFolder {
        lpBrowseInfo: lpbi,
        hWnd: core::ptr::null_mut(),
        hwndTreeView: core::ptr::null_mut(),
        pidlRet: core::ptr::null_mut(),
        layout: core::ptr::null_mut(),
        szMin: SIZE { cx: 0, cy: 0 },
        hChangeNotify: 0,
        pContextMenu: core::ptr::null_mut(),
    };

    let id = if ((*lpbi).ulFlags & BIF_USENEWUI) != 0 {
        IDD_BROWSE_FOR_FOLDER_NEW
    } else {
        IDD_BROWSE_FOR_FOLDER
    };

    let ret = DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCEW(id),
        (*lpbi).hwndOwner,
        Some(BrFolderDlgProc),
        &mut info as *mut _ as LPARAM,
    );

    if ret == (IDOK) as INT_PTR
        && ((*lpbi).ulFlags & BIF_NOTRANSLATETARGETS) == 0
        && RosGetProcessEffectiveVersion() >= _WIN32_WINNT_WINXP
    {
        let mut pidl_target: PIDLIST_ABSOLUTE = core::ptr::null_mut();
        if SHELL_GetIDListTarget(info.pidlRet, &mut pidl_target) == S_OK {
            ILFree(info.pidlRet);
            info.pidlRet = pidl_target;
        }
    }

    if ret != (IDOK) as INT_PTR {
        ILFree(info.pidlRet);
        return core::ptr::null_mut();
    }

    info.pidlRet
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     SHBrowseForFolderA/W functions
//  * COPYRIGHT:   Copyright 1999 Juergen Schmied
//  *              Copyright 2024 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// // FIXME: Many flags unimplemented
//
// #include "precomp.h"
//
// #include <ui/layout.h> // Resizable window
// #include <compat_undoc.h> // RosGetProcessEffectiveVersion
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// INT WINAPI Shell_GetCachedImageIndexW(LPCWSTR szPath, INT nIndex, UINT Flags);
//
// #define SHV_CHANGE_NOTIFY (WM_USER + 0x1111)
// enum { SHGDN_TREE = SHGDN_NORMAL | SHGDN_INFOLDER };
//
// static LPITEMIDLIST
// ILCloneToDepth(LPCITEMIDLIST pidlSrc, UINT depth)
// {
//     SIZE_T cb = 0;
//     for (LPCITEMIDLIST pidl = pidlSrc; pidl && depth--; pidl = ILGetNext(pidl))
//         cb += pidl->mkid.cb;
//
//     LPITEMIDLIST pidlOut = (LPITEMIDLIST)SHAlloc(cb + sizeof(WORD));
//     if (pidlOut)
//     {
//         CopyMemory(pidlOut, pidlSrc, cb);
//         ZeroMemory(((BYTE*)pidlOut) + cb, sizeof(WORD));
//     }
//     return pidlOut;
// }
//
// struct BrFolder
// {
//     LPBROWSEINFOW    lpBrowseInfo;
//     HWND             hWnd;
//     HWND             hwndTreeView;
//     PIDLIST_ABSOLUTE pidlRet;
//     LAYOUT_DATA*     layout;  // Filled by LayoutInit, used by LayoutUpdate
//     SIZE             szMin;
//     ULONG            hChangeNotify; // Change notification handle
//     IContextMenu*    pContextMenu; // Active context menu
// };
//
// struct BrItemData
// {
//     CComPtr<IShellFolder>            lpsfParent; // IShellFolder of the parent
//     PCIDLIST_RELATIVE                pidlChild;  // PIDL relative to parent
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFull;   // Fully qualified PIDL
// };
//
// static const LAYOUT_INFO g_layout_info[] =
// {
//     { IDC_BROWSE_FOR_FOLDER_TITLE,       BF_TOP | BF_LEFT | BF_RIGHT },
//     { IDC_BROWSE_FOR_FOLDER_STATUS,      BF_TOP | BF_LEFT | BF_RIGHT },
//     { IDC_BROWSE_FOR_FOLDER_TREEVIEW,    BF_TOP | BF_BOTTOM | BF_LEFT | BF_RIGHT },
//     { IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT, BF_TOP | BF_LEFT | BF_RIGHT },
//     { IDC_BROWSE_FOR_FOLDER_NEW_FOLDER,  BF_BOTTOM | BF_LEFT },
//     { IDOK,                              BF_BOTTOM | BF_RIGHT },
//     { IDCANCEL,                          BF_BOTTOM | BF_RIGHT },
// };
//
// #define SUPPORTED_FLAGS (BIF_STATUSTEXT | BIF_BROWSEFORCOMPUTER | BIF_RETURNFSANCESTORS | \
//                          BIF_RETURNONLYFSDIRS | BIF_NONEWFOLDERBUTTON | BIF_NEWDIALOGSTYLE | \
//                          BIF_BROWSEINCLUDEFILES)
//
// static HTREEITEM
// BrFolder_InsertItem(
//      BrFolder *info,
//      IShellFolder *lpsf,
//      PCUITEMID_CHILD pidlChild,
//      PCIDLIST_ABSOLUTE pidlParent,
//      HTREEITEM hParent);
//
// static inline DWORD
// BrowseFlagsToSHCONTF(UINT ulFlags)
// {
//     return SHCONTF_FOLDERS | ((ulFlags & BIF_BROWSEINCLUDEFILES) ? SHCONTF_NONFOLDERS : 0);
// }
//
// static void
// BrFolder_Callback(LPBROWSEINFOW lpBrowseInfo, HWND hWnd, UINT uMsg, LPARAM lParam)
// {
//     if (!lpBrowseInfo->lpfn)
//         return;
//     lpBrowseInfo->lpfn(hWnd, uMsg, lParam, lpBrowseInfo->lParam);
// }
//
// static BrItemData *
// BrFolder_GetItemData(BrFolder *info, HTREEITEM hItem)
// {
//     TVITEMW item = { TVIF_HANDLE | TVIF_PARAM };
//     item.hItem = hItem;
//     if (!TreeView_GetItem(info->hwndTreeView, &item))
//     {
//         ERR("TreeView_GetItem failed\n");
//         return NULL;
//     }
//     return (BrItemData *)item.lParam;
// }
//
// static SFGAOF
// BrFolder_GetItemAttributes(BrFolder *info, HTREEITEM hItem, SFGAOF Query)
// {
//     BrItemData *item = BrFolder_GetItemData(info, hItem);
//     SFGAOF Att = Query;
//     HRESULT hr = item ? item->lpsfParent->GetAttributesOf(1, &item->pidlChild, &Att) : E_FAIL;
//     return SUCCEEDED(hr) ? (Query & Att) : 0;
// }
//
// static HRESULT
// BrFolder_GetChildrenEnum(
//      BrFolder *info,
//      BrItemData *pItemData,
//      IEnumIDList **ppEnum)
// {
//     if (!pItemData)
//         return E_FAIL;
//
//     CComPtr<IEnumIDList> pEnumIL;
//     PCUITEMID_CHILD pidlRef = pItemData->pidlChild;
//     ULONG attrs = SFGAO_HASSUBFOLDER | SFGAO_FOLDER;
//     IShellFolder *lpsf = pItemData->lpsfParent;
//     HRESULT hr = lpsf->GetAttributesOf(1, &pidlRef, &attrs);
//     if (FAILED_UNEXPECTEDLY(hr) || !(attrs & SFGAO_FOLDER))
//         return E_FAIL;
//
//     CComPtr<IShellFolder> psfChild;
//     if (_ILIsDesktop(pItemData->pidlFull))
//         hr = SHGetDesktopFolder(&psfChild);
//     else
//         hr = lpsf->BindToObject(pidlRef, NULL, IID_PPV_ARG(IShellFolder, &psfChild));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return E_FAIL;
//
//     DWORD flags = BrowseFlagsToSHCONTF(info->lpBrowseInfo->ulFlags);
//     hr = psfChild->EnumObjects(info->hWnd, flags, &pEnumIL);
//     if (hr == S_OK)
//     {
//         if ((pEnumIL->Skip(1) != S_OK) || FAILED(pEnumIL->Reset()))
//             pEnumIL = NULL;
//     }
//
//     if (!pEnumIL || !(attrs & SFGAO_HASSUBFOLDER))
//         return E_FAIL; // No children
//
//     if (ppEnum)
//         *ppEnum = pEnumIL.Detach();
//
//     return S_OK; // There are children
// }
//
// /******************************************************************************
//  * BrFolder_InitTreeView [Internal]
//  *
//  * Called from WM_INITDIALOG handler.
//  */
// static void
// BrFolder_InitTreeView(BrFolder *info)
// {
//     HIMAGELIST hImageList;
//     HRESULT hr;
//     HTREEITEM hItem;
//
//     Shell_GetImageLists(NULL, &hImageList);
//     if (hImageList)
//         TreeView_SetImageList(info->hwndTreeView, hImageList, 0);
//
//     /* We want to call BrFolder_InsertItem down the code, in order to insert
//      * the root item of the treeview. Due to BrFolder_InsertItem's signature,
//      * we need the following to do this:
//      *
//      * + An ITEMIDLIST corresponding to _the parent_ of root.
//      * + An ITEMIDLIST, which is a relative path from root's parent to root
//      *   (containing a single SHITEMID).
//      * + An IShellFolder interface pointer of root's parent folder.
//      *
//      * If root is 'Desktop', then root's parent is also 'Desktop'.
//      */
//
//     PCIDLIST_ABSOLUTE pidlRoot = info->lpBrowseInfo->pidlRoot;
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlParent(ILClone(pidlRoot));
//     ILRemoveLastID(pidlParent);
//     PCIDLIST_RELATIVE pidlChild = ILFindLastID(pidlRoot);
//
//     CComPtr<IShellFolder> lpsfParent;
//     hr = SHBindToObject(NULL, pidlParent, /*NULL, */ IID_PPV_ARG(IShellFolder, &lpsfParent));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return;
//
//     TreeView_DeleteItem(info->hwndTreeView, TVI_ROOT);
//     hItem = BrFolder_InsertItem(info, lpsfParent, pidlChild, pidlParent, TVI_ROOT);
//     TreeView_Expand(info->hwndTreeView, hItem, TVE_EXPAND);
// }
//
// static HRESULT
// BrFolder_GetIconPair(BrItemData &item, TVITEMW &tvi)
// {
//     INT open = -1;
//     INT normal = SHMapPIDLToSystemImageListIndex(item.lpsfParent, item.pidlChild, &open);
//     if (normal >= 0)
//     {
//         tvi.iImage = normal;
//         tvi.iSelectedImage = (open >= 0) ? open : normal;
//         return S_OK;
//     }
//     tvi.iImage = tvi.iSelectedImage = Shell_GetCachedImageIndexW(swShell32Name, IDI_SHELL_FOLDER - 1, 0);
//     return S_FALSE;
// }
//
// /******************************************************************************
//  * BrFolder_GetName [Internal]
//  *
//  * Query a shell folder for the display name of one of its children
//  *
//  * PARAMS
//  *  lpsf           [I] IShellFolder interface of the folder to be queried.
//  *  pidlChild      [I] ITEMIDLIST of the child, relative to parent
//  *  dwFlags        [I] as in IShellFolder::GetDisplayNameOf
//  *  lpFriendlyName [O] The desired display name in unicode
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// static BOOL
// BrFolder_GetName(
//      IShellFolder *lpsf,
//      PCIDLIST_RELATIVE pidlChild,
//      DWORD dwFlags,
//      PWSTR lpFriendlyName)
// {
//     BOOL   bSuccess = FALSE;
//     STRRET str;
//
//     TRACE("%p %p %x %p\n", lpsf, pidlChild, dwFlags, lpFriendlyName);
//     if (!FAILED_UNEXPECTEDLY(lpsf->GetDisplayNameOf(pidlChild, dwFlags, &str)))
//         bSuccess = StrRetToStrNW(lpFriendlyName, MAX_PATH, &str, pidlChild);
//
//     TRACE("-- %s\n", debugstr_w(lpFriendlyName));
//     return bSuccess;
// }
//
// static BOOL
// BrFolder_GetName(
//      BrFolder *info,
//      HTREEITEM hItem,
//      UINT Flags,
//      PWSTR Buffer)
// {
//     if (BrItemData *item = BrFolder_GetItemData(info, hItem))
//         return BrFolder_GetName(item->lpsfParent, item->pidlChild, Flags, Buffer);
//     return FALSE;
// }
//
// static BOOL
// BrFolder_IsTreeItemInEnum(
//      BrFolder *info,
//      HTREEITEM hItem,
//      IEnumIDList *pEnum)
// {
//     BrItemData *pItemData = BrFolder_GetItemData(info, hItem);
//     if (!pItemData)
//         return FALSE;
//
//     pEnum->Reset();
//
//     CComHeapPtr<ITEMIDLIST_RELATIVE> pidlTemp;
//     while (pEnum->Next(1, &pidlTemp, NULL) == S_OK)
//     {
//         if (ILIsEqual(pidlTemp, pItemData->pidlChild))
//             return TRUE;
//
//         pidlTemp.Free();
//     }
//
//     return FALSE;
// }
//
// static BOOL
// BrFolder_TreeItemHasThisChild(
//      BrFolder *info,
//      HTREEITEM hItem,
//      PITEMID_CHILD pidlChild)
// {
//     for (hItem = TreeView_GetChild(info->hwndTreeView, hItem); hItem;
//          hItem = TreeView_GetNextSibling(info->hwndTreeView, hItem))
//     {
//         BrItemData *pItemData = BrFolder_GetItemData(info, hItem);
//         if (ILIsEqual(pItemData->pidlChild, pidlChild))
//             return TRUE;
//     }
//
//     return FALSE;
// }
//
// static HTREEITEM
// BrFolder_FindTreeItemOfAbsoluteItem(
//      BrFolder &info,
//      PCIDLIST_ABSOLUTE pidl,
//      HTREEITEM hItem = NULL)
// {
//     if (!hItem)
//         hItem = TreeView_GetRoot(info.hwndTreeView);
//
//     for (; pidl && hItem; hItem = TreeView_GetNextSibling(info.hwndTreeView, hItem))
//     {
//         BrItemData *pItemData = BrFolder_GetItemData(&info, hItem);
//         if (ILIsEqual(pItemData->pidlFull, pidl))
//             return hItem;
//         HTREEITEM hChild = TreeView_GetChild(info.hwndTreeView, hItem);
//         if (hChild && (hChild = BrFolder_FindTreeItemOfAbsoluteItem(info, pidl, hChild)) != NULL)
//             return hChild;
//     }
//     return NULL;
// }
//
// static BOOL
// BrFolder_UpdateItemEx(
//      BrFolder &info,
//      HTREEITEM hItem,
//      PCIDLIST_ABSOLUTE pidlFull,
//      UINT Flags = TVIF_TEXT | TVIF_IMAGE | TVIF_CHILDREN)
// {
//     ASSERT(hItem);
//     TVITEMW item = { (TVIF_HANDLE | Flags), hItem };
//     BrItemData *data = BrFolder_GetItemData(&info, hItem);
//     if (!data)
//         return FALSE;
//
//     if (pidlFull)
//     {
//         if ((pidlFull = ILClone(pidlFull)) == NULL)
//             return FALSE;
//         data->pidlFull.Free();
//         data->pidlFull.Attach(const_cast<PIDLIST_ABSOLUTE>(pidlFull));
//         data->pidlChild = ILFindLastID(data->pidlFull);
//         // Note: We assume lpsfParent does not change
//     }
//
//     WCHAR Name[MAX_PATH];
//     if (Flags & TVIF_TEXT)
//     {
//         item.pszText = Name;
//         if (!BrFolder_GetName(data->lpsfParent, data->pidlChild, SHGDN_TREE, Name))
//             return FALSE;
//     }
//
//     if (Flags & TVIF_IMAGE)
//     {
//         item.mask |= TVIF_SELECTEDIMAGE;
//         BrFolder_GetIconPair(*data, item);
//     }
//
//     item.cChildren = 0;
//     CComPtr<IEnumIDList> pEnum;
//     if ((Flags & TVIF_CHILDREN) && BrFolder_GetChildrenEnum(&info, data, &pEnum) == S_OK)
//     {
//         CComHeapPtr<ITEMIDLIST_RELATIVE> pidlTemp;
//         if (pEnum->Next(1, &pidlTemp, NULL) == S_OK)
//             ++item.cChildren;
//     }
//
//     return TreeView_SetItem(info.hwndTreeView, &item);
// }
//
// static void
// BrFolder_UpdateItem(
//      BrFolder *info,
//      HTREEITEM hItem)
// {
//     BrFolder_UpdateItemEx(*info, hItem, NULL, TVIF_IMAGE | TVIF_CHILDREN);
// }
//
// /******************************************************************************
//  * BrFolder_InsertItem [Internal]
//  *
//  * PARAMS
//  *  info       [I] data for the dialog
//  *  lpsf       [I] IShellFolder interface of the item's parent shell folder
//  *  pidlChild  [I] ITEMIDLIST of the child to insert, relative to parent
//  *  pidlParent [I] ITEMIDLIST of the parent shell folder
//  *  hParent    [I] The treeview-item that represents the parent shell folder
//  *
//  * RETURNS
//  *  Success: Handle to the created and inserted treeview-item
//  *  Failure: NULL
//  */
// static HTREEITEM
// BrFolder_InsertItem(
//      BrFolder *info,
//      IShellFolder *lpsf,
//      PCUITEMID_CHILD pidlChild,
//      PCIDLIST_ABSOLUTE pidlParent,
//      HTREEITEM hParent)
// {
//     if (!(BrowseFlagsToSHCONTF(info->lpBrowseInfo->ulFlags) & SHCONTF_NONFOLDERS))
//     {
//             if (SHGetAttributes(lpsf, pidlChild, SFGAO_STREAM) & SFGAO_STREAM)
//                 return NULL; // .zip files have both FOLDER and STREAM attributes set
//     }
//
//     WCHAR szName[MAX_PATH];
//     if (!BrFolder_GetName(lpsf, pidlChild, SHGDN_TREE, szName))
//         return NULL;
//
//     BrItemData *pItemData = new BrItemData();
//
//     TVITEMW item = { TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_PARAM | TVIF_CHILDREN };
//     item.pszText    = szName;
//     item.cchTextMax = _countof(szName);
//     item.lParam     = (LPARAM)pItemData;
//
//     PIDLIST_ABSOLUTE pidlFull =
//         (pidlParent ? ILCombine(pidlParent, pidlChild) : ILClone(pidlChild));
//
//     pItemData->lpsfParent = lpsf;
//     pItemData->pidlFull.Attach(pidlFull);
//     pItemData->pidlChild = ILFindLastID(pItemData->pidlFull);
//     BrFolder_GetIconPair(*pItemData, item);
//
//     if (BrFolder_GetChildrenEnum(info, pItemData, NULL) == S_OK)
//         item.cChildren = 1;
//
//     TVINSERTSTRUCTW tvins = { hParent };
//     tvins.item = item;
//     return TreeView_InsertItem(info->hwndTreeView, &tvins);
// }
//
// /******************************************************************************
//  * BrFolder_Expand [Internal]
//  *
//  * For each child (given by pEnum) of the parent shell folder, which is given by
//  * lpsf and whose PIDL is pidl, insert a treeview-item right under hParent
//  *
//  * PARAMS
//  *  info    [I] data for the dialog
//  *  lpsf    [I] IShellFolder interface of the parent shell folder
//  *  pidl    [I] ITEMIDLIST of the parent shell folder
//  *  hParent [I] The treeview item that represents the parent shell folder
//  *  pEnum   [I] An iterator for the children of the parent shell folder
//  */
// static void
// BrFolder_Expand(
//      BrFolder *info,
//      IShellFolder *lpsf,
//      PCIDLIST_ABSOLUTE pidlFull,
//      HTREEITEM hParent)
// {
//     TRACE("%p %p %p\n", lpsf, pidlFull, hParent);
//
//     // No IEnumIDList -> No children
//     BrItemData *pItemData = BrFolder_GetItemData(info, hParent);
//     CComPtr<IEnumIDList> pEnum;
//     HRESULT hr = BrFolder_GetChildrenEnum(info, pItemData, &pEnum);
//     if (FAILED(hr))
//         return;
//
//     SetCapture(info->hWnd);
//     SetCursor(LoadCursorW(NULL, (LPWSTR)IDC_WAIT));
//
//     CComHeapPtr<ITEMIDLIST_RELATIVE> pidlTemp;
//     ULONG ulFetched;
//     while (S_OK == pEnum->Next(1, &pidlTemp, &ulFetched))
//     {
//         /* Ignore return value of BrFolder_InsertItem to avoid incomplete folder listing */
//         BrFolder_InsertItem(info, lpsf, pidlTemp, pidlFull, hParent);
//         pidlTemp.Free(); // Finally, free the pidl that the shell gave us...
//     }
//
//     ReleaseCapture();
//     SetCursor(LoadCursorW(NULL, (LPWSTR)IDC_ARROW));
// }
//
// static inline BOOL
// PIDLIsType(LPCITEMIDLIST pidl, PIDLTYPE type)
// {
//     LPPIDLDATA data = _ILGetDataPointer(pidl);
//     if (!data)
//         return FALSE;
//     return (data->type == type);
// }
//
// static void
// BrFolder_CheckValidSelection(BrFolder *info, BrItemData *pItemData)
// {
//     LPBROWSEINFOW lpBrowseInfo = info->lpBrowseInfo;
//     PCIDLIST_RELATIVE pidlChild = pItemData->pidlChild;
//     DWORD dwAttributes;
//     HRESULT hr;
//
//     BOOL bEnabled = TRUE;
//     if ((lpBrowseInfo->ulFlags & BIF_BROWSEFORCOMPUTER) && !PIDLIsType(pidlChild, PT_COMP))
//         bEnabled = FALSE;
//
//     if (lpBrowseInfo->ulFlags & BIF_RETURNFSANCESTORS)
//     {
//         dwAttributes = SFGAO_FILESYSANCESTOR | SFGAO_FILESYSTEM;
//         hr = pItemData->lpsfParent->GetAttributesOf(1, &pidlChild, &dwAttributes);
//         if (FAILED(hr) || !(dwAttributes & (SFGAO_FILESYSANCESTOR | SFGAO_FILESYSTEM)))
//             bEnabled = FALSE;
//     }
//
//     dwAttributes = SFGAO_FOLDER | SFGAO_FILESYSTEM;
//     hr = pItemData->lpsfParent->GetAttributesOf(1, &pidlChild, &dwAttributes);
//     if (FAILED_UNEXPECTEDLY(hr) ||
//         ((dwAttributes & (SFGAO_FOLDER | SFGAO_FILESYSTEM)) != (SFGAO_FOLDER | SFGAO_FILESYSTEM)))
//     {
//         if (lpBrowseInfo->ulFlags & BIF_RETURNONLYFSDIRS)
//             bEnabled = FALSE;
//         EnableWindow(GetDlgItem(info->hWnd, IDC_BROWSE_FOR_FOLDER_NEW_FOLDER), FALSE);
//     }
//     else
//     {
//         EnableWindow(GetDlgItem(info->hWnd, IDC_BROWSE_FOR_FOLDER_NEW_FOLDER), TRUE);
//     }
//
//     SendMessageW(info->hWnd, BFFM_ENABLEOK, 0, bEnabled);
// }
//
// static LRESULT
// BrFolder_Treeview_Delete(BrFolder *info, NMTREEVIEWW *pnmtv)
// {
//     BrItemData *pItemData = (BrItemData *)pnmtv->itemOld.lParam;
//
//     TRACE("TVN_DELETEITEMA/W %p\n", pItemData);
//
//     delete pItemData;
//     return 0;
// }
//
// static LRESULT
// BrFolder_Treeview_Expand(BrFolder *info, NMTREEVIEWW *pnmtv)
// {
//     BrItemData *pItemData = (BrItemData *)pnmtv->itemNew.lParam;
//
//     TRACE("TVN_ITEMEXPANDINGA/W\n");
//
//     if ((pnmtv->itemNew.state & TVIS_EXPANDEDONCE))
//         return 0;
//
//     HRESULT hr = S_OK;
//     CComPtr<IShellFolder> lpsf2;
//     if (!_ILIsEmpty(pItemData->pidlChild))
//     {
//         hr = pItemData->lpsfParent->BindToObject(pItemData->pidlChild, NULL,
//                                                  IID_PPV_ARG(IShellFolder, &lpsf2));
//     }
//     else
//     {
//         lpsf2 = pItemData->lpsfParent;
//     }
//
//     HTREEITEM hItem = pnmtv->itemNew.hItem;
//     if (!FAILED_UNEXPECTEDLY(hr))
//         BrFolder_Expand(info, lpsf2, pItemData->pidlFull, hItem);
//
//     // My Computer is already sorted and trying to do a simple text
//     // sort will only mess things up
//     if (!_ILIsMyComputer(pItemData->pidlChild))
//         TreeView_SortChildren(info->hwndTreeView, hItem, FALSE);
//
//     return 0;
// }
//
// static HRESULT
// BrFolder_Treeview_Changed(BrFolder *info, NMTREEVIEWW *pnmtv)
// {
//     BrItemData *pItemData = (BrItemData *)pnmtv->itemNew.lParam;
//
//     ILFree(info->pidlRet);
//     info->pidlRet = ILClone(pItemData->pidlFull);
//
//     WCHAR szName[MAX_PATH];
//     if (BrFolder_GetName(pItemData->lpsfParent, pItemData->pidlChild, SHGDN_NORMAL | SHGDN_INFOLDER, szName))
//         SetDlgItemTextW(info->hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT, szName);
//
//     BrFolder_Callback(info->lpBrowseInfo, info->hWnd, BFFM_SELCHANGED, (LPARAM)info->pidlRet);
//     BrFolder_CheckValidSelection(info, pItemData);
//     return S_OK;
// }
//
// static LRESULT
// BrFolder_Treeview_Rename(BrFolder *info, NMTVDISPINFOW *pnmtv)
// {
//     if (!pnmtv->item.pszText)
//         return FALSE;
//
//     HTREEITEM hItem = TreeView_GetSelection(info->hwndTreeView);
//     BrItemData *data = BrFolder_GetItemData(info, hItem);
//     ASSERT(data);
//     ASSERT(BrFolder_GetItemAttributes(info, hItem, SFGAO_CANRENAME) & SFGAO_CANRENAME);
//
//     PITEMID_CHILD newChild;
//     HRESULT hr = data->lpsfParent->SetNameOf(info->hWnd, data->pidlChild, pnmtv->item.pszText,
//                                              SHGDN_NORMAL | SHGDN_INFOLDER, &newChild);
//     if (FAILED(hr))
//         return FALSE;
//
//     LPITEMIDLIST newFull;
//     if (SUCCEEDED(hr = SHILClone(data->pidlFull, &newFull)))
//     {
//         ILRemoveLastID(newFull);
//         if (SUCCEEDED(hr = SHILAppend(newChild, &newFull)))
//         {
//             data->pidlFull.Free();
//             data->pidlFull.Attach(newFull);
//             data->pidlChild = ILFindLastID(data->pidlFull);
//         }
//         newChild = NULL; // SHILAppend is nuts and frees this
//     }
//     ILFree(newChild);
//
//     BOOL AllowTreeTextChange = !BrFolder_UpdateItemEx(*info, hItem, NULL, TVIF_TEXT);
//
//     NMTREEVIEWW nmtv;
//     nmtv.itemNew.hItem = hItem;
//     nmtv.itemNew.mask = TVIF_PARAM;
//     nmtv.itemNew.lParam = (LPARAM)data;
//     BrFolder_Treeview_Changed(info, &nmtv);
//     return AllowTreeTextChange;
// }
//
// static HRESULT
// BrFolder_Rename(BrFolder *info, HTREEITEM hItem)
// {
//     TreeView_SelectItem(info->hwndTreeView, hItem);
//     TreeView_EditLabel(info->hwndTreeView, hItem);
//     return S_OK;
// }
//
// static void
// BrFolder_Delete(BrFolder *info, HTREEITEM hItem)
// {
//     BrItemData *data = BrFolder_GetItemData(info, hItem);
//     if (!data || !BrFolder_GetItemAttributes(info, hItem, SFGAO_CANDELETE))
//         return;
//     CComPtr<IContextMenu> pCM;
//     if (FAILED(data->lpsfParent->GetUIObjectOf(info->hWnd, 1, &data->pidlChild, IID_NULL_PPV_ARG(IContextMenu, &pCM))))
//         return;
//     UINT fCMIC = 0;
//     if (GetKeyState(VK_SHIFT) < 0)
//         fCMIC |= CMIC_MASK_SHIFT_DOWN;
//     if (GetKeyState(VK_CONTROL) < 0)
//         fCMIC |= CMIC_MASK_CONTROL_DOWN;
//     SHInvokeCommandOnContextMenu(info->hWnd, NULL, pCM, fCMIC, "delete");
// }
//
// static void
// BrFolder_Refresh( BrFolder *info);
//
// static LRESULT
// BrFolder_Treeview_Keydown(BrFolder *info, LPNMTVKEYDOWN keydown)
// {
//     // Old dialog doesn't support those advanced features
//     if (!(info->lpBrowseInfo->ulFlags & BIF_USENEWUI))
//         return 0;
//
//     HTREEITEM hItem = TreeView_GetSelection(info->hwndTreeView);
//
//     switch (keydown->wVKey)
//     {
//         case VK_F2:
//             BrFolder_Rename(info, hItem);
//             break;
//         case VK_DELETE:
//             BrFolder_Delete(info, hItem);
//             break;
//         case 'R':
//         {
//             if (GetKeyState(VK_CONTROL) < 0) // Ctrl+R
//                 BrFolder_Refresh(info);
//             break;
//         }
//         case VK_F5:
//         {
//             BrFolder_Refresh(info);
//             break;
//         }
//     }
//     return 0;
// }
//
// static LRESULT
// BrFolder_OnNotify(BrFolder *info, UINT CtlID, LPNMHDR lpnmh)
// {
//     NMTREEVIEWW *pnmtv = (NMTREEVIEWW *)lpnmh;
//
//     TRACE("%p %x %p msg=%x\n", info, CtlID, lpnmh, pnmtv->hdr.code);
//
//     if (pnmtv->hdr.idFrom != IDC_BROWSE_FOR_FOLDER_TREEVIEW)
//         return 0;
//
//     switch (pnmtv->hdr.code)
//     {
//         case TVN_DELETEITEMA:
//         case TVN_DELETEITEMW:
//             return BrFolder_Treeview_Delete(info, pnmtv);
//
//         case TVN_ITEMEXPANDINGA:
//         case TVN_ITEMEXPANDINGW:
//             return BrFolder_Treeview_Expand(info, pnmtv);
//
//         case TVN_SELCHANGEDA:
//         case TVN_SELCHANGEDW:
//             return BrFolder_Treeview_Changed(info, pnmtv);
//
//         case TVN_BEGINLABELEDITA:
//         case TVN_BEGINLABELEDITW:
//         {
//             HWND hWndEdit = TreeView_GetEditControl(lpnmh->hwndFrom);
//             NMTVDISPINFO &tvdi = *(NMTVDISPINFO*)lpnmh;
//             UINT att = BrFolder_GetItemAttributes(info, tvdi.item.hItem, SFGAO_CANRENAME);
//             if (!(att & SFGAO_CANRENAME))
//             {
//                 MessageBeep(0xffffffff);
//                 return S_FALSE;
//             }
//             WCHAR Name[MAX_PATH];
//             if (BrFolder_GetName(info, tvdi.item.hItem, SHGDN_INFOLDER | SHGDN_FOREDITING, Name))
//                 SetWindowTextW(hWndEdit, Name);
//             return S_OK;
//         }
//
//         case TVN_ENDLABELEDITW:
//             return BrFolder_Treeview_Rename(info, (LPNMTVDISPINFOW)pnmtv);
//
//         case TVN_KEYDOWN:
//             return BrFolder_Treeview_Keydown(info, (LPNMTVKEYDOWN)pnmtv);
//
//         default:
//             WARN("unhandled (%d)\n", pnmtv->hdr.code);
//             break;
//     }
//
//     return 0;
// }
//
// static BOOL
// BrFolder_OnInitDialog(HWND hWnd, BrFolder *info)
// {
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlDesktop;
//     SHChangeNotifyEntry ntreg;
//     LPBROWSEINFOW lpBrowseInfo = info->lpBrowseInfo;
//
//     info->hWnd = hWnd;
//     SetWindowLongPtrW(hWnd, DWLP_USER, (LONG_PTR)info);
//
//     if (lpBrowseInfo->ulFlags & BIF_NEWDIALOGSTYLE)
//         FIXME("flags BIF_NEWDIALOGSTYLE partially implemented\n");
//
//     if (lpBrowseInfo->ulFlags & ~SUPPORTED_FLAGS)
//         FIXME("flags %x not implemented\n", (lpBrowseInfo->ulFlags & ~SUPPORTED_FLAGS));
//
//     info->layout = NULL;
//     if (lpBrowseInfo->ulFlags & BIF_USENEWUI)
//     {
//         RECT rcWnd;
//
//         // Resize the treeview if there's not editbox
//         if ((lpBrowseInfo->ulFlags & BIF_NEWDIALOGSTYLE) &&
//             !(lpBrowseInfo->ulFlags & BIF_EDITBOX))
//         {
//             RECT rcEdit, rcTreeView;
//             GetWindowRect(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT), &rcEdit);
//             GetWindowRect(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_TREEVIEW), &rcTreeView);
//             LONG cy = rcTreeView.top - rcEdit.top;
//             MapWindowPoints(NULL, hWnd, (LPPOINT)&rcTreeView, sizeof(RECT) / sizeof(POINT));
//             rcTreeView.top -= cy;
//             MoveWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_TREEVIEW),
//                        rcTreeView.left, rcTreeView.top,
//                        rcTreeView.right - rcTreeView.left,
//                        rcTreeView.bottom - rcTreeView.top, TRUE);
//         }
//
//         if (lpBrowseInfo->ulFlags & BIF_NEWDIALOGSTYLE)
//             info->layout = LayoutInit(hWnd, g_layout_info, _countof(g_layout_info));
//
//         // TODO: Windows allows shrinking the windows a bit
//         GetWindowRect(hWnd, &rcWnd);
//         info->szMin.cx = rcWnd.right - rcWnd.left;
//         info->szMin.cy = rcWnd.bottom - rcWnd.top;
//     }
//
//     if (lpBrowseInfo->lpszTitle)
//         SetDlgItemTextW(hWnd, IDC_BROWSE_FOR_FOLDER_TITLE, lpBrowseInfo->lpszTitle);
//     else
//         ShowWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_TITLE), SW_HIDE);
//
//     if (!(lpBrowseInfo->ulFlags & BIF_STATUSTEXT) || (lpBrowseInfo->ulFlags & BIF_USENEWUI))
//         ShowWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_STATUS), SW_HIDE);
//
//     // Hide "Make New Folder" Button?
//     if ((lpBrowseInfo->ulFlags & BIF_NONEWFOLDERBUTTON) ||
//         !(lpBrowseInfo->ulFlags & BIF_NEWDIALOGSTYLE))
//     {
//         ShowWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_NEW_FOLDER), SW_HIDE);
//     }
//
//     // Hide the editbox?
//     if (!(lpBrowseInfo->ulFlags & BIF_EDITBOX))
//     {
//         ShowWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER), SW_HIDE);
//         ShowWindow(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT), SW_HIDE);
//     }
//
//     info->hwndTreeView = GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_TREEVIEW);
//     if (info->hwndTreeView)
//         BrFolder_InitTreeView(info);
//     else
//         ERR("treeview control missing!\n");
//
//     // Register for change notifications
//     SHGetFolderLocation(NULL, CSIDL_DESKTOP, NULL, 0, &pidlDesktop);
//
//     ntreg.pidl = pidlDesktop;
//     ntreg.fRecursive = TRUE;
//     info->hChangeNotify = SHChangeNotifyRegister(hWnd,
//                                                  SHCNRF_ShellLevel | SHCNRF_NewDelivery,
//                                                  SHCNE_ALLEVENTS,
//                                                  SHV_CHANGE_NOTIFY, 1, &ntreg);
//
//     if (!lpBrowseInfo->pidlRoot)
//     {
//         UINT csidl = (lpBrowseInfo->ulFlags & BIF_NEWDIALOGSTYLE) ? CSIDL_PERSONAL : CSIDL_DRIVES;
//         LPITEMIDLIST pidl = SHCloneSpecialIDList(NULL, csidl, TRUE);
//         if (pidl)
//         {
//             SendMessageW(info->hWnd, BFFM_SETSELECTION, FALSE, (LPARAM)pidl);
//             if (csidl == CSIDL_DRIVES)
//                 SendMessageW(info->hWnd, BFFM_SETEXPANDED, FALSE, (LPARAM)pidl);
//             ILFree(pidl);
//         }
//     }
//
//     BrFolder_Callback(info->lpBrowseInfo, hWnd, BFFM_INITIALIZED, 0);
//
//     SHAutoComplete(GetDlgItem(hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT),
//                    (SHACF_FILESYS_ONLY | SHACF_URLHISTORY | SHACF_FILESYSTEM));
//     return TRUE;
// }
//
// static HRESULT
// BrFolder_NewFolder(BrFolder *info)
// {
//     CComPtr<IShellFolder> desktop, cur;
//     WCHAR wszNewFolder[25], path[MAX_PATH], name[MAX_PATH];
//
//     HRESULT hr = SHGetDesktopFolder(&desktop);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     if (info->pidlRet)
//     {
//         hr = desktop->BindToObject(info->pidlRet, NULL, IID_PPV_ARG(IShellFolder, &cur));
//         if (FAILED_UNEXPECTEDLY(hr))
//             return hr;
//
//         hr = SHGetPathFromIDListW(info->pidlRet, path);
//     }
//     else
//     {
//         cur = desktop;
//         hr = SHGetFolderPathW(NULL, CSIDL_DESKTOPDIRECTORY, NULL, SHGFP_TYPE_CURRENT, path);
//     }
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = E_FAIL;
//     if (!LoadStringW(shell32_hInstance, IDS_NEWFOLDER, wszNewFolder, _countof(wszNewFolder)))
//         return hr;
//
//     if (!PathYetAnotherMakeUniqueName(name, path, NULL, wszNewFolder))
//         return hr;
//
//     INT len = lstrlenW(path);
//     if (len < MAX_PATH && name[len] == L'\\')
//         len++;
//
//     if (!CreateDirectoryW(name, NULL))
//         return hr;
//
//     // Update parent of newly created directory
//     HTREEITEM hParent = TreeView_GetSelection(info->hwndTreeView);
//     if (!hParent)
//         return hr;
//
//     TreeView_Expand(info->hwndTreeView, hParent, TVE_EXPAND);
//
//     TVITEMW item = { TVIF_PARAM | TVIF_STATE };
//     item.hItem = hParent;
//     TreeView_GetItem(info->hwndTreeView, &item);
//     BrItemData *item_data = (BrItemData *)item.lParam;
//     if (!item_data)
//         return hr;
//
//     // Update treeview
//     if (!(item.state & TVIS_EXPANDEDONCE))
//     {
//         item.mask = TVIF_STATE;
//         item.state = item.stateMask = TVIS_EXPANDEDONCE;
//         TreeView_SetItem(info->hwndTreeView, &item);
//     }
//
//     CComHeapPtr<ITEMIDLIST_RELATIVE> pidlNew;
//     hr = cur->ParseDisplayName(NULL, NULL, name + len, NULL, &pidlNew, NULL);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     HTREEITEM hAdded = BrFolder_InsertItem(info, cur, pidlNew, item_data->pidlFull, hParent);
//     TreeView_SortChildren(info->hwndTreeView, hParent, FALSE);
//     return BrFolder_Rename(info, hAdded);
// }
//
// static void
// BrFolder_OnOK(BrFolder *info)
// {
//     // Get the text
//     WCHAR szPath[MAX_PATH];
//     GetDlgItemTextW(info->hWnd, IDC_BROWSE_FOR_FOLDER_FOLDER_TEXT, szPath, _countof(szPath));
//     StrTrimW(szPath, L" \t");
//
//     // The original pidl is owned by the treeview and will be free'd.
//     if (!PathIsRelativeW(szPath) && PathIsDirectoryW(szPath))
//         info->pidlRet = ILCreateFromPathW(szPath); // It's valid path
//     else
//         info->pidlRet = ILClone(info->pidlRet);
//
//     if (!info->pidlRet) // A null pidl would mean a cancel
//         info->pidlRet = _ILCreateDesktop();
//
//     pdump(info->pidlRet);
//
//     LPBROWSEINFOW lpBrowseInfo = info->lpBrowseInfo;
//     if (!lpBrowseInfo->pszDisplayName)
//         return;
//
//     SHFILEINFOW fileInfo = { NULL };
//     lpBrowseInfo->pszDisplayName[0] = UNICODE_NULL;
//     if (SHGetFileInfoW((LPCWSTR)info->pidlRet, 0, &fileInfo, sizeof(fileInfo),
//                        SHGFI_PIDL | SHGFI_DISPLAYNAME))
//     {
//         lstrcpynW(lpBrowseInfo->pszDisplayName, fileInfo.szDisplayName, MAX_PATH);
//     }
// }
//
// static void
// BrFolder_OnCommand(BrFolder *info, UINT id)
// {
//     switch (id)
//     {
//         case IDOK:
//         {
//             BrFolder_OnOK(info);
//             EndDialog(info->hWnd, IDOK);
//             break;
//         }
//         case IDCANCEL:
//         {
//             EndDialog(info->hWnd, IDCANCEL);
//             break;
//         }
//         case IDC_BROWSE_FOR_FOLDER_NEW_FOLDER:
//         {
//             BrFolder_NewFolder(info);
//             break;
//         }
//     }
// }
//
// static void
// GetTreeViewItemContextMenuPos(HWND hWnd, HTREEITEM hItem, POINT *ppt)
// {
//     RECT rc;
//     if (TreeView_GetItemRect(hWnd, hItem, &rc, TRUE))
//     {
//         ppt->x = (rc.left + rc.right) / 2;
//         ppt->y = (rc.top + rc.bottom) / 2;
//     }
//     ClientToScreen(hWnd, ppt);
// }
//
// static void
// BrFolder_OnContextMenu(BrFolder &info, LPARAM lParam)
// {
//     enum { IDC_TOGGLE = 1, ID_FIRSTCMD, ID_LASTCMD = 0xffff };
//     HTREEITEM hSelected = TreeView_GetSelection(info.hwndTreeView);
//     CMINVOKECOMMANDINFOEX ici = { sizeof(ici), CMIC_MASK_PTINVOKE, info.hWnd };
//     ici.nShow = SW_SHOW;
//     ici.ptInvoke.x = GET_X_LPARAM(lParam);
//     ici.ptInvoke.y = GET_Y_LPARAM(lParam);
//     if ((int)lParam == -1) // Keyboard
//     {
//         GetTreeViewItemContextMenuPos(info.hwndTreeView, hSelected, &ici.ptInvoke);
//     }
//     else // Get correct item for right-click on not current item
//     {
//         TVHITTESTINFO hti = { ici.ptInvoke };
//         ScreenToClient(info.hwndTreeView, &hti.pt);
//         hSelected = TreeView_HitTest(info.hwndTreeView, &hti);
//     }
//     BrItemData *item = BrFolder_GetItemData(&info, hSelected);
//     if (!item)
//         return; // Not on an item
//
//     TV_ITEM tvi;
//     tvi.mask = TVIF_STATE | TVIF_CHILDREN;
//     tvi.stateMask = TVIS_EXPANDED;
//     tvi.hItem = hSelected;
//     TreeView_GetItem(info.hwndTreeView, &tvi);
//
//     CComPtr<IContextMenu> pcm;
//     HRESULT hr = item->lpsfParent->GetUIObjectOf(info.hWnd, 1, &item->pidlChild,
//                                                  IID_IContextMenu, NULL, (void**)&pcm);
//     if (FAILED(hr))
//         return;
//
//     HMENU hMenu = CreatePopupMenu();
//     if (!hMenu)
//         return;
//     info.pContextMenu = pcm;
//     UINT cmf = ((GetKeyState(VK_SHIFT) < 0) ? CMF_EXTENDEDVERBS : 0) | CMF_CANRENAME;
//     hr = pcm->QueryContextMenu(hMenu, 0, ID_FIRSTCMD, ID_LASTCMD, CMF_EXPLORE | cmf);
//     if (hr > 0)
//         _InsertMenuItemW(hMenu, 0, TRUE, 0, MFT_SEPARATOR, NULL, 0);
//     _InsertMenuItemW(hMenu, 0, TRUE, IDC_TOGGLE, MFT_STRING,
//         MAKEINTRESOURCEW((tvi.state & TVIS_EXPANDED) ? IDS_COLLAPSE : IDS_EXPAND),
//         MFS_DEFAULT | (tvi.cChildren ? 0 : MFS_GRAYED));
//
//     UINT cmd = TrackPopupMenuEx(hMenu, TPM_RETURNCMD, ici.ptInvoke.x, ici.ptInvoke.y, info.hWnd, NULL);
//     ici.lpVerb = MAKEINTRESOURCEA(cmd - ID_FIRSTCMD);
//     if (cmd == IDC_TOGGLE)
//     {
//         TreeView_SelectItem(info.hwndTreeView, hSelected);
//         TreeView_Expand(info.hwndTreeView, hSelected, TVE_TOGGLE);
//     }
//     else if (cmd != 0 && GetDfmCmd(pcm, ici.lpVerb) == DFM_CMD_RENAME)
//     {
//         BrFolder_Rename(&info, hSelected);
//     }
//     else if (cmd != 0)
//     {
//         if (GetKeyState(VK_SHIFT) < 0)
//             ici.fMask |= CMIC_MASK_SHIFT_DOWN;
//         if (GetKeyState(VK_CONTROL) < 0)
//             ici.fMask |= CMIC_MASK_CONTROL_DOWN;
//         pcm->InvokeCommand((CMINVOKECOMMANDINFO*)&ici);
//     }
//     info.pContextMenu = NULL;
//     DestroyMenu(hMenu);
// }
//
// static BOOL
// BrFolder_ExpandToPidl(BrFolder *info, LPITEMIDLIST pidlSelection, HTREEITEM *phItem)
// {
//     LPITEMIDLIST pidlCurrent = pidlSelection;
//     if (_ILIsDesktop(pidlSelection))
//     {
//         if (phItem)
//             *phItem = TVI_ROOT;
//         return TRUE;
//     }
//
//     // Initialize item to point to the first child of the root folder.
//     TVITEMEXW item = { TVIF_PARAM };
//     item.hItem = TreeView_GetRoot(info->hwndTreeView);
//     if (item.hItem)
//         item.hItem = TreeView_GetChild(info->hwndTreeView, item.hItem);
//
//     // Walk the tree along the nodes corresponding to the remaining ITEMIDLIST
//     UINT depth = _ILGetDepth(info->lpBrowseInfo->pidlRoot);
//     while (item.hItem && pidlCurrent)
//     {
//         LPITEMIDLIST pidlNeedle = ILCloneToDepth(pidlSelection, ++depth);
//         if (_ILIsEmpty(pidlNeedle))
//         {
//             ILFree(pidlNeedle);
//             item.hItem = NULL; // Failure
//             break;
//         }
// next:
//         TreeView_GetItem(info->hwndTreeView, &item);
//         const BrItemData *pItemData = (BrItemData *)item.lParam;
//         if (ILIsEqual(pItemData->pidlFull, pidlNeedle))
//         {
//             BOOL done = _ILGetDepth(pidlSelection) == _ILGetDepth(pidlNeedle);
//             if (done)
//             {
//                 pidlCurrent = NULL; // Success
//             }
//             else
//             {
//                 TreeView_Expand(info->hwndTreeView, item.hItem, TVE_EXPAND);
//                 item.hItem = TreeView_GetChild(info->hwndTreeView, item.hItem);
//             }
//         }
//         else
//         {
//             item.hItem = TreeView_GetNextSibling(info->hwndTreeView, item.hItem);
//             if (item.hItem)
//                 goto next;
//         }
//         ILFree(pidlNeedle);
//     }
//
//     if (phItem)
//         *phItem = item.hItem;
//
//     return (_ILIsEmpty(pidlCurrent) && item.hItem);
// }
//
// static BOOL
// BrFolder_ExpandToString(BrFolder *info, LPWSTR pszString, HTREEITEM *phItem)
// {
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlSelection;
//     HRESULT hr = SHParseDisplayName(pszString, NULL, &pidlSelection, 0, NULL);
//     return SUCCEEDED(hr) && BrFolder_ExpandToPidl(info, pidlSelection, phItem);
// }
//
// static BOOL
// BrFolder_OnSetExpanded(BrFolder *info, LPITEMIDLIST pidlSelection, LPWSTR pszString)
// {
//     HTREEITEM hItem;
//     BOOL ret;
//     if (pszString)
//         ret = BrFolder_ExpandToString(info, pszString, &hItem);
//     else
//         ret = BrFolder_ExpandToPidl(info, pidlSelection, &hItem);
//
//     if (ret)
//         TreeView_Expand(info->hwndTreeView, hItem, TVE_EXPAND);
//     return ret;
// }
//
// static BOOL
// BrFolder_OnSetSelectionPidl(BrFolder *info, LPITEMIDLIST pidlSelection)
// {
//     if (!pidlSelection)
//         return FALSE;
//
//     HTREEITEM hItem;
//     BOOL ret = BrFolder_ExpandToPidl(info, pidlSelection, &hItem);
//     if (ret)
//         TreeView_SelectItem(info->hwndTreeView, hItem);
//     return ret;
// }
//
// static BOOL
// BrFolder_OnSetSelectionW(BrFolder *info, LPWSTR pszSelection)
// {
//     if (!pszSelection)
//         return FALSE;
//
//     HTREEITEM hItem;
//     BOOL ret = BrFolder_ExpandToString(info, pszSelection, &hItem);
//     if (ret)
//         TreeView_SelectItem(info->hwndTreeView, hItem);
//     return ret;
// }
//
// static BOOL
// BrFolder_OnSetSelectionA(BrFolder *info, LPSTR pszSelectionA)
// {
//     if (!pszSelectionA)
//         return FALSE;
//
//     CComHeapPtr<WCHAR> pszSelectionW;
//     __SHCloneStrAtoW(&pszSelectionW, pszSelectionA);
//     if (!pszSelectionW)
//         return FALSE;
//
//     return BrFolder_OnSetSelectionW(info, pszSelectionW);
// }
//
// static void
// BrFolder_OnDestroy(BrFolder *info)
// {
//     if (info->layout)
//     {
//         LayoutDestroy(info->layout);
//         info->layout = NULL;
//     }
//
//     SHChangeNotifyDeregister(info->hChangeNotify);
// }
//
// static void
// BrFolder_RefreshRecurse(
//      BrFolder *info,
//      HTREEITEM hTarget)
// {
//     // Get enum
//     CComPtr<IEnumIDList> pEnum;
//     BrItemData *pItemData = BrFolder_GetItemData(info, hTarget);
//     HRESULT hrEnum = BrFolder_GetChildrenEnum(info, pItemData, &pEnum);
//
//     // Insert new items
//     if (SUCCEEDED(hrEnum))
//     {
//         CComHeapPtr<ITEMIDLIST_RELATIVE> pidlTemp;
//         while (S_OK == pEnum->Next(1, &pidlTemp, NULL))
//         {
//             if (!BrFolder_TreeItemHasThisChild(info, hTarget, pidlTemp))
//             {
//                 BrFolder_InsertItem(info, pItemData->lpsfParent, pidlTemp, pItemData->pidlFull,
//                                     hTarget);
//             }
//             pidlTemp.Free();
//         }
//     }
//
//     // Delete zombie items and update items
//     HTREEITEM hItem, hNextItem;
//     for (hItem = TreeView_GetChild(info->hwndTreeView, hTarget); hItem; hItem = hNextItem)
//     {
//         hNextItem = TreeView_GetNextSibling(info->hwndTreeView, hItem);
//
//         if (FAILED(hrEnum) || !BrFolder_IsTreeItemInEnum(info, hItem, pEnum))
//         {
//             TreeView_DeleteItem(info->hwndTreeView, hItem);
//             hNextItem = TreeView_GetChild(info->hwndTreeView, hTarget);
//             continue;
//         }
//
//         BrFolder_UpdateItem(info, hItem);
//         BrFolder_RefreshRecurse(info, hItem);
//     }
//
//     if (SUCCEEDED(hrEnum))
//         TreeView_SortChildren(info->hwndTreeView, hTarget, FALSE);
// }
//
// static void
// BrFolder_Refresh( BrFolder *info)
// {
//     HTREEITEM hRoot = TreeView_GetRoot(info->hwndTreeView);
//
//     SendMessageW(info->hwndTreeView, WM_SETREDRAW, FALSE, 0);
//
//     BrFolder_RefreshRecurse(info, hRoot);
//
//     SendMessageW(info->hwndTreeView, WM_SETREDRAW, TRUE, 0);
//     InvalidateRect(info->hwndTreeView, NULL, TRUE);
// }
//
// static void
// BrFolder_OnChangeEx(
//      BrFolder *info,
//      PCIDLIST_ABSOLUTE pidl1,
//      PCIDLIST_ABSOLUTE pidl2,
//      LONG event)
// {
//     TRACE("(%p)->(%p, %p, 0x%lX)\n", info, pidl1, pidl2, event);
//
//     switch (event)
//     {
//         case SHCNE_RENAMEFOLDER:
//         case SHCNE_RENAMEITEM:
//         case SHCNE_UPDATEITEM:
//         {
//             UINT UpdateFlags = (event == SHCNE_UPDATEITEM) ? (TVIF_IMAGE | TVIF_CHILDREN) : (TVIF_TEXT);
//             if (HTREEITEM hTI = BrFolder_FindTreeItemOfAbsoluteItem(*info, pidl1))
//             {
//                 if (BrFolder_UpdateItemEx(*info, hTI, pidl2, UpdateFlags))
//                 {
//                     if ((hTI = TreeView_GetParent(info->hwndTreeView, hTI)) != NULL)
//                         TreeView_SortChildren(info->hwndTreeView, hTI, FALSE);
//                     return;
//                 }
//             }
//             break;
//         }
//     }
//
//     switch (event)
//     {
//         case SHCNE_DRIVEADD:
//         case SHCNE_MKDIR:
//         case SHCNE_CREATE:
//         case SHCNE_DRIVEREMOVED:
//         case SHCNE_RMDIR:
//         case SHCNE_DELETE:
//         case SHCNE_RENAMEFOLDER:
//         case SHCNE_RENAMEITEM:
//         case SHCNE_UPDATEDIR:
//         case SHCNE_UPDATEITEM:
//         {
//             // FIXME: Avoid full refresh and optimize for speed. Use pidl0 and pidl1
//             BrFolder_Refresh(info);
//             break;
//         }
//     }
// }
//
// // SHV_CHANGE_NOTIFY
// static void
// BrFolder_OnChange(BrFolder *info, WPARAM wParam, LPARAM lParam)
// {
//     // We use SHCNRF_NewDelivery method
//     HANDLE hChange = (HANDLE)wParam;
//     DWORD dwProcID = (DWORD)lParam;
//
//     PIDLIST_ABSOLUTE *ppidl = NULL;
//     LONG event;
//     HANDLE hLock = SHChangeNotification_Lock(hChange, dwProcID, &ppidl, &event);
//     if (hLock == NULL)
//     {
//         ERR("hLock == NULL\n");
//         return;
//     }
//
//     BrFolder_OnChangeEx(info, ppidl[0], ppidl[1], (event & ~SHCNE_INTERRUPT));
//
//     SHChangeNotification_Unlock(hLock);
// }
//
// /*************************************************************************
//  *             BrFolderDlgProc32  (not an exported API function)
//  */
// static INT_PTR __stdcall
// BrFolderDlgProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     if (uMsg == WM_INITDIALOG)
//         return BrFolder_OnInitDialog(hWnd, (BrFolder *)lParam);
//
//     BrFolder *info = (BrFolder *)GetWindowLongPtrW(hWnd, DWLP_USER);
//     if (!info)
//         return 0;
//
//     if (info->pContextMenu && SHELL_IsContextMenuMsg(uMsg))
//     {
//         LRESULT result;
//         if (SHForwardContextMenuMsg(info->pContextMenu, uMsg, wParam,
//                                     lParam, &result, TRUE) == S_OK)
//         {
//             SetWindowLongPtr(hWnd, DWLP_MSGRESULT, result);
//             return TRUE;
//         }
//     }
//
//     switch (uMsg)
//     {
//         case WM_NOTIFY:
//             SetWindowLongPtr(hWnd, DWLP_MSGRESULT, BrFolder_OnNotify(info, (UINT)wParam, (LPNMHDR)lParam));
//             return TRUE;
//
//         case WM_COMMAND:
//             BrFolder_OnCommand(info, wParam);
//             break;
//
//         case WM_CONTEXTMENU:
//             if (info->hwndTreeView == (HWND)wParam)
//                 BrFolder_OnContextMenu(*info, lParam);
//             break;
//
//         case WM_GETMINMAXINFO:
//             ((LPMINMAXINFO)lParam)->ptMinTrackSize.x = info->szMin.cx;
//             ((LPMINMAXINFO)lParam)->ptMinTrackSize.y = info->szMin.cy;
//             break;
//
//         case WM_SIZE:
//             if (info->layout)  // New style dialogs
//                 LayoutUpdate(hWnd, info->layout, g_layout_info, _countof(g_layout_info));
//             break;
//
//         case BFFM_SETSTATUSTEXTA:
//             SetDlgItemTextA(hWnd, IDC_BROWSE_FOR_FOLDER_STATUS, (LPSTR)lParam);
//             break;
//
//         case BFFM_SETSTATUSTEXTW:
//             SetDlgItemTextW(hWnd, IDC_BROWSE_FOR_FOLDER_STATUS, (LPWSTR)lParam);
//             break;
//
//         case BFFM_ENABLEOK:
//             EnableWindow(GetDlgItem(hWnd, IDOK), lParam != 0);
//             break;
//
//         case BFFM_SETOKTEXT: // Unicode only
//             SetDlgItemTextW(hWnd, IDOK, (LPWSTR)lParam);
//             break;
//
//         case BFFM_SETSELECTIONA:
//             if (wParam) // String
//                 return BrFolder_OnSetSelectionA(info, (LPSTR)lParam);
//             else // PIDL
//                 return BrFolder_OnSetSelectionPidl(info, (LPITEMIDLIST)lParam);
//
//         case BFFM_SETSELECTIONW:
//             if (wParam) // String
//                 return BrFolder_OnSetSelectionW(info, (LPWSTR)lParam);
//             else // PIDL
//                 return BrFolder_OnSetSelectionPidl(info, (LPITEMIDLIST)lParam);
//
//         case BFFM_SETEXPANDED: // Unicode only
//             if (wParam) // String
//                 return BrFolder_OnSetExpanded(info, NULL, (LPWSTR)lParam);
//             else // PIDL
//                 return BrFolder_OnSetExpanded(info, (LPITEMIDLIST)lParam, NULL);
//
//         case SHV_CHANGE_NOTIFY:
//             BrFolder_OnChange(info, wParam, lParam);
//             break;
//
//         case WM_DESTROY:
//             BrFolder_OnDestroy(info);
//             break;
//     }
//
//     return 0;
// }
//
// /*************************************************************************
//  * SHBrowseForFolderA [SHELL32.@]
//  * SHBrowseForFolder  [SHELL32.@]
//  */
// extern "C"
// LPITEMIDLIST WINAPI
// SHBrowseForFolderA(LPBROWSEINFOA lpbi)
// {
//     BROWSEINFOW bi;
//     bi.hwndOwner = lpbi->hwndOwner;
//     bi.pidlRoot = lpbi->pidlRoot;
//
//     WCHAR szName[MAX_PATH];
//     bi.pszDisplayName = (lpbi->pszDisplayName ? szName : NULL);
//
//     CComHeapPtr<WCHAR> pszTitle;
//     if (lpbi->lpszTitle)
//         __SHCloneStrAtoW(&pszTitle, lpbi->lpszTitle);
//     bi.lpszTitle = pszTitle;
//
//     bi.ulFlags = lpbi->ulFlags;
//     bi.lpfn = lpbi->lpfn;
//     bi.lParam = lpbi->lParam;
//     bi.iImage = lpbi->iImage;
//     PIDLIST_ABSOLUTE pidl = SHBrowseForFolderW(&bi);
//
//     if (bi.pszDisplayName)
//         SHUnicodeToAnsi(bi.pszDisplayName, lpbi->pszDisplayName, MAX_PATH);
//
//     lpbi->iImage = bi.iImage;
//     return pidl;
// }
//
// /*************************************************************************
//  * SHBrowseForFolderW [SHELL32.@]
//  */
// extern "C"
// LPITEMIDLIST WINAPI
// SHBrowseForFolderW(LPBROWSEINFOW lpbi)
// {
//     TRACE("%p\n", lpbi);
//
//     // MSDN says the caller must initialize COM. We do it anyway in case the caller forgot.
//     COleInit OleInit;
//     BrFolder info = { lpbi };
//
//     INT id = ((lpbi->ulFlags & BIF_USENEWUI) ? IDD_BROWSE_FOR_FOLDER_NEW : IDD_BROWSE_FOR_FOLDER);
//     INT_PTR ret = DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(id), lpbi->hwndOwner,
//                                   BrFolderDlgProc, (LPARAM)&info);
//     if (ret == IDOK && !(lpbi->ulFlags & BIF_NOTRANSLATETARGETS) &&
//         RosGetProcessEffectiveVersion() >= _WIN32_WINNT_WINXP)
//     {
//         PIDLIST_ABSOLUTE pidlTarget;
//         if (SHELL_GetIDListTarget(info.pidlRet, &pidlTarget) == S_OK)
//         {
//             ILFree(info.pidlRet);
//             info.pidlRet = pidlTarget;
//         }
//     }
//     if (ret != IDOK)
//     {
//         ILFree(info.pidlRet);
//         return NULL;
//     }
//
//     return info.pidlRet;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
