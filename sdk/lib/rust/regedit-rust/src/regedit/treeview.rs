use crate::regedit::edit::RenameKey;
use crate::regedit::childwnd::UpdateAddress;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_111FD57AB4CB: [u16; 20] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0043, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000,
];
pub static RUSTCONV_WSTR_22FF61FCDE09: [u16; 4] = [0x0048u16, 0x004B, 0x0055, 0x0000];
pub static RUSTCONV_WSTR_50C930882486: [u16; 19] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x004C, 0x004F, 0x0043, 0x0041, 0x004C,
    0x005F, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_5CDE252B65C7: [u16; 14] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0044, 0x0059, 0x004E, 0x005F, 0x0044,
    0x0041, 0x0054, 0x0041, 0x0000,
];
pub static RUSTCONV_WSTR_87D16031626E: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_89F38F0B7FC9: [u16; 5] = [0x0048u16, 0x004B, 0x004C, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_8E490F9323D5: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9C98935C198E: [u16; 11] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_A8B389DD9483: [u16; 39] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0063, 0x0072, 0x0065,
    0x0061, 0x0074, 0x0065, 0x0020, 0x006E, 0x0065, 0x0077, 0x0020, 0x006B, 0x0065,
    0x0079, 0x0021, 0x000A, 0x000A, 0x0045, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020,
    0x0043, 0x006F, 0x0064, 0x0065, 0x003A, 0x0020, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_BFE074CE3641: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0043, 0x0000];
pub static RUSTCONV_WSTR_C57F475C07AD: [u16; 5] = [0x0048u16, 0x004B, 0x0044, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_D02D0BCFE5D4: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0055, 0x0000];
pub static RUSTCONV_WSTR_D0EE0F231465: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_D30E8899E227: [u16; 12] = [
    0x0073u16, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_FB47CCC40FE6: [u16; 3] = [0x0025u16, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Image_Open: ::core::ffi::c_int = 0;

static mut Image_Closed: ::core::ffi::c_int = 0;

static mut Image_Root: ::core::ffi::c_int = 0;

static mut pathBuffer: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

pub const NUM_ICONS: i32 = 3;
pub const IDI_SHELL_FOLDER: i32 = 4;
pub const IDI_SHELL_FOLDER_OPEN: i32 = 5;
pub const IDI_SHELL_MY_COMPUTER: i32 = 16;

unsafe extern "C" fn get_item_path(hwndTV: HWND, hItem: HTREEITEM, phKey: *mut HKEY, pKeyPath: *mut LPWSTR, pPathLen: *mut core::ffi::c_int, pMaxLen: *mut core::ffi::c_int) -> BOOL {
    let mut item: TVITEMW = core::mem::zeroed();
    let mut maxLen: size_t = 0;
    let mut len: size_t = 0;
    let mut newStr: LPWSTR = core::ptr::null_mut();

    item.mask = TVIF_PARAM;
    item.hItem = hItem;
    if TreeView_GetItem!(hwndTV, core::ptr::addr_of_mut!(item)) == 0 {
        return (FALSE) as BOOL;
    }

    if item.lParam != 0 {
        /* found root key with valid key value */
        *phKey = item.lParam as HKEY;
        return (TRUE) as BOOL;
    }

    if get_item_path(hwndTV, TreeView_GetParent!(hwndTV, hItem), phKey, pKeyPath, pPathLen, pMaxLen) == 0 {
        return (FALSE) as BOOL;
    }
    if *pPathLen != 0 {
        *(*pKeyPath).offset(*pPathLen as isize) = 0x005Cu16;
        *pPathLen += 1;
    }

    loop {
        item.mask = TVIF_TEXT;
        item.hItem = hItem;
        item.pszText = (*pKeyPath).offset(*pPathLen as isize);
        maxLen = (*pMaxLen - *pPathLen) as size_t;
        item.cchTextMax = maxLen as core::ffi::c_int;
        if TreeView_GetItem!(hwndTV, core::ptr::addr_of_mut!(item)) == 0 {
            return (FALSE) as BOOL;
        }
        len = wcslen(item.pszText as *const u16) as size_t;
        if len < maxLen.wrapping_sub(1 as size_t) {
            *pPathLen += len as core::ffi::c_int;
            break;
        }
        // `*pMaxLen` counts WCHARs (see GetItemPath); the C doubled it as bytes.
        newStr = HeapReAlloc(
            GetProcessHeap(),
            0,
            *pKeyPath as *mut core::ffi::c_void,
            (*pMaxLen as SIZE_T * 2).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if newStr.is_null() {
            return (FALSE) as BOOL;
        }
        *pKeyPath = newStr;
        *pMaxLen *= 2;
        if (TRUE) == 0 {
            break;
        }
    }

    (TRUE) as i32
}
#[no_mangle]
pub unsafe extern "C" fn GetItemPath(hwndTV: HWND, mut hItem: HTREEITEM, phRootKey: *mut HKEY) -> LPCWSTR {
    let mut pathLen: ::core::ffi::c_int = 0;
    let mut maxLen: ::core::ffi::c_int = 0;

    *phRootKey = ::core::ptr::null_mut();

    if (*core::ptr::addr_of!(pathBuffer)).is_null() {
        *core::ptr::addr_of_mut!(pathBuffer) = HeapAlloc(GetProcessHeap(), 0, 1024) as LPWSTR;
    }
    if (*core::ptr::addr_of!(pathBuffer)).is_null() {
        return ::core::ptr::null();
    }

    *(*core::ptr::addr_of!(pathBuffer)) = UNICODE_NULL;

    // In WCHARs. Divergence from the C, which passed HeapSize's BYTE count on as the WCHAR
    // capacity get_item_path hands TreeView as cchTextMax: a long key path overran the buffer
    // by up to half its size before the doubling kicked in.
    maxLen = HeapSize(GetProcessHeap(), 0, *core::ptr::addr_of!(pathBuffer) as LPCVOID) as ::core::ffi::c_int;
    if maxLen != -1 {
        maxLen /= core::mem::size_of::<WCHAR>() as ::core::ffi::c_int;
    }

    if hItem.is_null() {
        hItem = TreeView_GetSelection!(hwndTV);
    }
    if maxLen == -1
        || hItem.is_null()
        || get_item_path(
            hwndTV,
            hItem,
            phRootKey,
            core::ptr::addr_of_mut!(pathBuffer),
            core::ptr::addr_of_mut!(pathLen),
            core::ptr::addr_of_mut!(maxLen),
        ) == 0
    {
        return ::core::ptr::null();
    }
    return *core::ptr::addr_of!(pathBuffer) as LPCWSTR;
}
#[no_mangle]
pub unsafe extern "C" fn DeleteNode(hwndTV: HWND, mut hItem: HTREEITEM) -> BOOL {
    if hItem.is_null() {
        hItem = TreeView_GetSelection!(hwndTV);
    }
    if hItem.is_null() {
        return (FALSE) as BOOL;
    }
    return TreeView_DeleteItem!(hwndTV, hItem);
}

unsafe extern "C" fn AddEntryToTree(
    mut hwndTV: HWND,
    mut hParent: HTREEITEM,
    mut label: LPWSTR,
    mut hKey: HKEY,
    mut dwChildren: DWORD,
) -> HTREEITEM {
    let mut tvi: TVITEMW = core::mem::zeroed();
    let mut tvins: TVINSERTSTRUCTW = core::mem::zeroed();

    if !hKey.is_null() {
        if (RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwChildren),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32 != ERROR_SUCCESS
        {
            dwChildren = 0;
        }
    }

    tvi.mask = (TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_CHILDREN | TVIF_PARAM) as UINT;
    tvi.pszText = label;
    tvi.cchTextMax = wcslen(tvi.pszText as *const u16) as i32;
    tvi.iImage = Image_Closed;
    tvi.iSelectedImage = Image_Open;
    tvi.cChildren = dwChildren as i32;
    tvi.lParam = hKey as LPARAM;
    tvins.__bindgen_anon_1.item = tvi;
    tvins.hInsertAfter = (if !hKey.is_null() { TVI_LAST } else { TVI_FIRST }) as HTREEITEM;
    tvins.hParent = hParent;
    return TreeView_InsertItem!(hwndTV, core::ptr::addr_of_mut!(tvins));
}
#[no_mangle]
pub unsafe extern "C" fn RefreshTreeItem(hwndTV: HWND, hItem: HTREEITEM) -> BOOL {
    let mut hRoot: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut childItem: HTREEITEM = core::ptr::null_mut();
    let mut KeyPath: LPCWSTR = core::ptr::null();
    let mut dwCount: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwMaxSubKeyLen: DWORD = 0;
    let mut Name: LPWSTR = core::ptr::null_mut();
    let mut tvItem: TVITEMW = core::mem::zeroed();
    let mut pszNodes: LPWSTR = core::ptr::null_mut();
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut s: LPWSTR = core::ptr::null_mut();
    let mut bAddedAny: BOOL = 0;

    KeyPath = GetItemPath(hwndTV, hItem, core::ptr::addr_of_mut!(hRoot));

    'done: {
        if *KeyPath != 0 {
            if (RegOpenKeyExW(hRoot, KeyPath, 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 != ERROR_SUCCESS {
                break 'done;
            }
        } else {
            hKey = hRoot;
        }

        if (RegQueryInfoKeyW(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwCount), core::ptr::addr_of_mut!(dwMaxSubKeyLen), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
            break 'done;
        }

        /* Set the number of children again */
        tvItem.mask = TVIF_CHILDREN;
        tvItem.hItem = hItem;
        tvItem.cChildren = dwCount as ::core::ffi::c_int;
        if TreeView_SetItem!(hwndTV, core::ptr::addr_of_mut!(tvItem)) == 0 {
            break 'done;
        }

        /* We don't have to bother with the rest if it's not expanded. */
        if TreeView_GetItemState!(hwndTV, hItem, TVIS_EXPANDED) == 0 {
            bSuccess = (TRUE) as i32;
            break 'done;
        }

        dwMaxSubKeyLen = dwMaxSubKeyLen.wrapping_add(1); /* account for the \0 terminator */
        Name = HeapAlloc(GetProcessHeap(), 0, (dwMaxSubKeyLen as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
        if Name.is_null() {
            break 'done;
        }
        tvItem.cchTextMax = dwMaxSubKeyLen as ::core::ffi::c_int;
        /*if (!(tvItem.pszText = HeapAlloc(GetProcessHeap(), 0, dwMaxSubKeyLen * sizeof(WCHAR)))) {
            goto done;
        }*/

        /* Get all of the tree node siblings in one contiguous block of memory */
        {
            let mut dwPhysicalSize: DWORD = 0;
            let mut dwActualSize: DWORD = 0;
            let mut dwNewPhysicalSize: DWORD = 0;
            let mut pszNewNodes: LPWSTR = core::ptr::null_mut();
            let mut dwStep: DWORD = 10000;

            childItem = TreeView_GetChild!(hwndTV, hItem);
            while !childItem.is_null() {
                if dwActualSize.wrapping_add(dwMaxSubKeyLen).wrapping_add(1) > dwPhysicalSize {
                    dwNewPhysicalSize = dwActualSize.wrapping_add(dwMaxSubKeyLen).wrapping_add(1).wrapping_add(dwStep);

                    if !pszNodes.is_null() {
                        pszNewNodes = HeapReAlloc(GetProcessHeap(), 0, pszNodes as LPVOID, (dwNewPhysicalSize as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                    } else {
                        pszNewNodes = HeapAlloc(GetProcessHeap(), 0, (dwNewPhysicalSize as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                    }
                    if pszNewNodes.is_null() {
                        break 'done;
                    }

                    dwPhysicalSize = dwNewPhysicalSize;
                    pszNodes = pszNewNodes;
                }

                tvItem.mask = TVIF_TEXT;
                tvItem.hItem = childItem;
                tvItem.pszText = pszNodes.add(dwActualSize as usize);
                tvItem.cchTextMax = dwPhysicalSize.wrapping_sub(dwActualSize) as ::core::ffi::c_int;
                if TreeView_GetItem!(hwndTV, core::ptr::addr_of_mut!(tvItem)) == 0 {
                    break 'done;
                }

                dwActualSize = dwActualSize.wrapping_add(wcslen(pszNodes.add(dwActualSize as usize)) as DWORD).wrapping_add(1);
                childItem = TreeView_GetNextSibling!(hwndTV, childItem);
            }

            if !pszNodes.is_null() {
                *pszNodes.add(dwActualSize as usize) = 0x0000u16;
            }
        }

        /* Now go through all the children in the tree, and check if any have to be removed. */
        childItem = TreeView_GetChild!(hwndTV, hItem);
        while !childItem.is_null() {
            let nextItem: HTREEITEM = TreeView_GetNextSibling!(hwndTV, childItem);
            if (RefreshTreeItem(hwndTV, childItem)) as u32 == FALSE {
                let _ = TreeView_DeleteItem!(hwndTV, childItem);
            }
            childItem = nextItem;
        }

        /* Now go through all the children in the registry, and check if any have to be added. */
        bAddedAny = (FALSE) as i32;
        dwIndex = 0;
        while dwIndex < dwCount {
            'body: {
                let mut cName: DWORD = dwMaxSubKeyLen;
                let mut dwSubCount: DWORD = 0;
                let mut found: BOOL = 0;

                found = (FALSE) as i32;
                if (RegEnumKeyExW(hKey, dwIndex, Name, core::ptr::addr_of_mut!(cName), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
                    break 'body;
                }

                /* Check if the node is already in there. */
                if !pszNodes.is_null() {
                    s = pszNodes;
                    while *s != 0 {
                        if wcscmp(s, Name) == 0 {
                            found = (TRUE) as i32;
                            break;
                        }
                        s = s.add(wcslen(s).wrapping_add(1));
                    }
                }

                if (found) as u32 == FALSE {
                    /* Find the number of children of the node. */
                    dwSubCount = 0;
                    if (RegOpenKeyExW(hKey, Name, 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hSubKey))) as u32 == ERROR_SUCCESS {
                        if (RegQueryInfoKeyW(hSubKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSubCount), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
                            dwSubCount = 0;
                        }
                        RegCloseKey(hSubKey);
                    }

                    AddEntryToTree(hwndTV, hItem, Name, core::ptr::null_mut(), dwSubCount);
                    bAddedAny = (TRUE) as i32;
                }
            }
            dwIndex = dwIndex.wrapping_add(1);
        }

        if bAddedAny != 0 {
            SendMessageW(hwndTV, TVM_SORTCHILDREN, 0, hItem as LPARAM);
        }

        bSuccess = (TRUE) as i32;
    }

    /* done: */
    // Divergence from the C: it closed hKey on the two success paths only, so every failure
    // after the open leaked it, and it closed it even when it WAS hRoot -- the tree item's own
    // root handle, which for a remote registry (RegConnectRegistryW) is a real key the tree
    // goes on using. Closed here, once, and only when this function opened it.
    if !hKey.is_null() && hKey != hRoot {
        RegCloseKey(hKey);
    }
    if !pszNodes.is_null() {
        HeapFree(GetProcessHeap(), 0, pszNodes as LPVOID);
    }
    if !Name.is_null() {
        HeapFree(GetProcessHeap(), 0, Name as LPVOID);
    }
    return bSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn RefreshTreeView(hwndTV: HWND) -> BOOL {
    let mut hItem: HTREEITEM = core::ptr::null_mut();
    let mut hSelectedItem: HTREEITEM = core::ptr::null_mut();
    let mut hcursorOld: HCURSOR = core::ptr::null_mut();

    hSelectedItem = TreeView_GetSelection!(hwndTV);
    hcursorOld = SetCursor(LoadCursor(core::ptr::null_mut(), IDC_WAIT));
    SendMessageW(hwndTV, WM_SETREDRAW, (FALSE) as WPARAM, 0);

    hItem = TreeView_GetChild!(hwndTV, TreeView_GetRoot!(hwndTV));
    while !hItem.is_null() {
        RefreshTreeItem(hwndTV, hItem);
        hItem = TreeView_GetNextSibling!(hwndTV, hItem);
    }

    SendMessageW(hwndTV, WM_SETREDRAW, (TRUE) as WPARAM, 0);
    SetCursor(hcursorOld);

    /* We reselect the currently selected node, this will prompt a refresh of the listview. */
    let _ = TreeView_SelectItem!(hwndTV, hSelectedItem);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InsertNode(hwndTV: HWND, mut hItem: HTREEITEM, name: LPWSTR) -> HTREEITEM {
    let mut buf: [WCHAR; MAX_NEW_KEY_LEN as usize] = [0 as WCHAR; MAX_NEW_KEY_LEN as usize];
    let mut hNewItem: HTREEITEM = core::ptr::null_mut();
    let mut item: TVITEMEXW = core::mem::zeroed();

    /* Default to the current selection */
    if hItem.is_null() {
        hItem = TreeView_GetSelection!(hwndTV);
        if hItem.is_null() {
            return FALSE as usize as HTREEITEM;
        }
    }

    memset(
        core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<TVITEMEXW>(),
    );
    item.hItem = hItem;
    item.mask = (TVIF_CHILDREN | TVIF_HANDLE | TVIF_STATE) as UINT;
    if TreeView_GetItem!(hwndTV, core::ptr::addr_of_mut!(item)) == 0 {
        return FALSE as usize as HTREEITEM;
    }

    if item.state & (TVIS_EXPANDEDONCE as UINT) != 0 {
        hNewItem = AddEntryToTree(
            hwndTV,
            hItem,
            name,
            core::ptr::null_mut(),
            0,
        );
        SendMessageW(
            hwndTV,
            TVM_SORTCHILDREN as UINT,
            0 as WPARAM,
            hItem as LPARAM,
        );
    } else {
        item.mask = (TVIF_CHILDREN | TVIF_HANDLE) as UINT;
        item.hItem = hItem;
        item.cChildren = 1 as core::ffi::c_int;
        if TreeView_SetItem!(hwndTV, core::ptr::addr_of_mut!(item)) == 0 {
            return FALSE as usize as HTREEITEM;
        }
    }

    let _ = TreeView_Expand!(hwndTV, hItem, TVE_EXPAND);
    if hNewItem.is_null() {
        hNewItem = TreeView_GetChild!(hwndTV, hItem);
        'search: loop {
            if hNewItem.is_null() {
                break;
            }
            'body: {
                item.mask = (TVIF_HANDLE | TVIF_TEXT) as UINT;
                item.hItem = hNewItem;
                item.pszText = core::ptr::addr_of_mut!(buf) as *mut WCHAR as LPWSTR;
                item.cchTextMax = ARRAY_SIZE!(buf) as _;
                if TreeView_GetItem!(hwndTV, core::ptr::addr_of_mut!(item)) == 0 {
                    break 'body;
                }
                if wcscmp(name, item.pszText) == 0 {
                    break 'search;
                }
            }
            hNewItem = TreeView_GetNextSibling!(hwndTV, hNewItem);
        }
    }
    if !hNewItem.is_null() {
        let _ = TreeView_SelectItem!(hwndTV, hNewItem);
    }

    return hNewItem;
}
#[no_mangle]
pub unsafe extern "C" fn StartKeyRename(hwndTV: HWND) -> HWND {
    let mut hItem: HTREEITEM;
    hItem = TreeView_GetSelection!(hwndTV);
    if hItem.is_null() {
        return core::ptr::null_mut();
    }
    return TreeView_EditLabel!(hwndTV, hItem);
}

unsafe extern "C" fn InitTreeViewItems(hwndTV: HWND, pHostName: LPWSTR) -> BOOL {
    let mut tvi: TVITEMW = core::mem::zeroed();
    let mut tvins: TVINSERTSTRUCTW = core::mem::zeroed();
    let mut hRoot: HTREEITEM = core::ptr::null_mut();

    tvi.mask = TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_CHILDREN | TVIF_PARAM;
    /* Set the text of the item.  */
    tvi.pszText = pHostName;
    tvi.cchTextMax = wcslen(tvi.pszText) as i32;
    /* Assume the item is not a parent item, so give it an image.  */
    tvi.iImage = Image_Root;
    tvi.iSelectedImage = Image_Root;
    tvi.cChildren = 5;
    /* Save the heading level in the item's application-defined data area.  */
    tvi.lParam = 0 as LPARAM;
    tvins.__bindgen_anon_1.item = tvi;
    tvins.hInsertAfter = (TVI_FIRST) as HTREEITEM;
    tvins.hParent = TVI_ROOT;
    /* Add the item to the tree view control.  */
    hRoot = TreeView_InsertItem!(hwndTV, core::ptr::addr_of_mut!(tvins));
    if hRoot.is_null() {
        return (FALSE) as BOOL;
    }

    if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_87D16031626E.as_ptr() as LPWSTR, HKEY_CLASSES_ROOT, 1).is_null() {
        return (FALSE) as BOOL;
    }
    if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_D0EE0F231465.as_ptr() as LPWSTR, HKEY_CURRENT_USER, 1).is_null() {
        return (FALSE) as BOOL;
    }
    if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_50C930882486.as_ptr() as LPWSTR, HKEY_LOCAL_MACHINE, 1).is_null() {
        return (FALSE) as BOOL;
    }
    if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_9C98935C198E.as_ptr() as LPWSTR, HKEY_USERS, 1).is_null() {
        return (FALSE) as BOOL;
    }
    if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_111FD57AB4CB.as_ptr() as LPWSTR, HKEY_CURRENT_CONFIG, 1).is_null() {
        return (FALSE) as BOOL;
    }

    if GetVersion() & 0x80000000 != 0 {
        /* Win9x specific key */
        if AddEntryToTree(hwndTV, hRoot, RUSTCONV_WSTR_5CDE252B65C7.as_ptr() as LPWSTR, HKEY_DYN_DATA, 1).is_null() {
            return (FALSE) as BOOL;
        }
    }

    /* expand and select host name */
    let _ = TreeView_Expand!(hwndTV, hRoot, TVE_EXPAND);
    let _ = TreeView_Select!(hwndTV, hRoot, TVGN_CARET);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitTreeViewImageLists(hwndTV: HWND) -> BOOL {
    let mut himl: HIMAGELIST;
    let mut hico: HICON;
    let cx: INT = GetSystemMetrics((SM_CXSMICON) as i32) as INT;
    let cy: INT = GetSystemMetrics((SM_CYSMICON) as i32) as INT;
    let hShell32: HMODULE = GetModuleHandleW(RUSTCONV_WSTR_D30E8899E227.as_ptr());

    /* Create the image list.  */
    himl = ImageList_Create(cx, cy, ILC_MASK | ILC_COLOR32, 0, NUM_ICONS);
    if himl.is_null() {
        return (FALSE) as BOOL;
    }

    /* Add the open file, closed file, and document bitmaps.  */
    hico = (LoadImageW(
        hShell32,
        MAKEINTRESOURCEW!(IDI_SHELL_FOLDER_OPEN),
        IMAGE_ICON,
        cx,
        cy,
        LR_DEFAULTCOLOR,
    )).cast();
    if !hico.is_null() {
        Image_Open = ImageList_AddIcon!(himl, hico);
        DestroyIcon(hico);
    }

    hico = (LoadImageW(
        hShell32,
        MAKEINTRESOURCEW!(IDI_SHELL_FOLDER),
        IMAGE_ICON,
        cx,
        cy,
        LR_DEFAULTCOLOR,
    )).cast();
    if !hico.is_null() {
        Image_Closed = ImageList_AddIcon!(himl, hico);
        DestroyIcon(hico);
    }

    hico = (LoadImageW(
        hShell32,
        MAKEINTRESOURCEW!(IDI_SHELL_MY_COMPUTER),
        IMAGE_ICON,
        cx,
        cy,
        LR_DEFAULTCOLOR,
    )).cast();
    if !hico.is_null() {
        Image_Root = ImageList_AddIcon!(himl, hico);
        DestroyIcon(hico);
    }

    /* Fail if not all of the images were added.  */
    if ImageList_GetImageCount(himl) < NUM_ICONS {
        ImageList_Destroy(himl);
        return (FALSE) as BOOL;
    }

    /* Associate the image list with the tree view control.  */
    let _ = TreeView_SetImageList!(hwndTV, himl, TVSIL_NORMAL);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn OnTreeExpanding(hwndTV: HWND, pnmtv: *mut NMTREEVIEWW) -> BOOL {
    let mut dwCount: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwMaxSubKeyLen: DWORD = 0;
    let mut hRoot: HKEY = core::ptr::null_mut();
    let mut hNewKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut keyPath: LPCWSTR = core::ptr::null();
    let mut Name: LPWSTR = core::ptr::null_mut();
    let mut errCode: LONG = 0;
    let mut hcursorOld: HCURSOR = core::ptr::null_mut();

    static mut expanding: ::core::ffi::c_int = 0;
    if expanding != 0 {
        return (FALSE) as BOOL;
    }
    if ((*pnmtv).itemNew.state & TVIS_EXPANDEDONCE) != 0 {
        return (TRUE) as BOOL;
    }
    expanding = (TRUE) as i32;
    hcursorOld = SetCursor(LoadCursor(
        core::ptr::null_mut(),
        IDC_WAIT,
    ));
    SendMessageW(hwndTV, WM_SETREDRAW, (FALSE) as WPARAM, 0);

    'done: {
        keyPath = GetItemPath(
            hwndTV,
            (*pnmtv).itemNew.hItem,
            core::ptr::addr_of_mut!(hRoot),
        );
        if keyPath.is_null() {
            break 'done;
        }

        if *keyPath != 0 {
            errCode = RegOpenKeyExW(
                hRoot,
                keyPath,
                0,
                KEY_READ,
                core::ptr::addr_of_mut!(hNewKey),
            ) as LONG;
            if (errCode) as u32 != ERROR_SUCCESS {
                break 'done;
            }
        } else {
            hNewKey = hRoot;
        }

        errCode = RegQueryInfoKeyW(
            hNewKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwCount),
            core::ptr::addr_of_mut!(dwMaxSubKeyLen),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if (errCode) as u32 != ERROR_SUCCESS {
            break 'done;
        }
        dwMaxSubKeyLen = dwMaxSubKeyLen.wrapping_add(1); /* account for the \0 terminator */
        Name = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwMaxSubKeyLen as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if Name.is_null() {
            break 'done;
        }

        dwIndex = 0;
        while dwIndex < dwCount {
            let mut cName: DWORD = dwMaxSubKeyLen;
            let mut dwSubCount: DWORD = 0;

            errCode = RegEnumKeyExW(
                hNewKey,
                dwIndex,
                Name,
                core::ptr::addr_of_mut!(cName),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as LONG;
            if !((errCode) as u32 != ERROR_SUCCESS) {
                errCode = RegOpenKeyExW(
                    hNewKey,
                    Name,
                    0,
                    KEY_QUERY_VALUE,
                    core::ptr::addr_of_mut!(hKey),
                ) as LONG;
                if (errCode) as u32 == ERROR_SUCCESS {
                    errCode = RegQueryInfoKeyW(
                        hKey,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwSubCount),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) as LONG;
                    RegCloseKey(hKey);
                }
                if (errCode) as u32 != ERROR_SUCCESS {
                    dwSubCount = 0;
                }
                AddEntryToTree(
                    hwndTV,
                    (*pnmtv).itemNew.hItem,
                    Name,
                    core::ptr::null_mut(),
                    dwSubCount,
                );
            }
            dwIndex = dwIndex.wrapping_add(1);
        }

        SendMessageW(
            hwndTV,
            TVM_SORTCHILDREN,
            0,
            (*pnmtv).itemNew.hItem as LPARAM,
        );

        RegCloseKey(hNewKey);
        HeapFree(GetProcessHeap(), 0, (Name).cast());
    }

    // done:
    SendMessageW(hwndTV, WM_SETREDRAW, (TRUE) as WPARAM, 0);
    SetCursor(hcursorOld);
    expanding = (FALSE) as i32;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CreateNewKey(hwndTV: HWND, hItem: HTREEITEM) -> BOOL {
    let mut szNewKeyFormat: [WCHAR; 128] = [0; 128];
    let mut szNewKey: [WCHAR; 128] = [0; 128];
    let mut pszKeyPath: LPCWSTR;
    let mut iIndex: i32 = 1;
    let mut nResult: LONG;
    let mut hRootKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hNewKey: HKEY = core::ptr::null_mut();
    let mut bSuccess: BOOL = (FALSE) as i32;
    let mut dwDisposition: DWORD = 0;
    let mut hNewItem: HTREEITEM;

    'done: {
        pszKeyPath = GetItemPath(hwndTV, hItem, core::ptr::addr_of_mut!(hRootKey));
        if pszKeyPath.is_null() {
            return bSuccess;
        }
        if *pszKeyPath.offset(0) == 0x0000u16 {
            hKey = hRootKey;
        } else if (RegOpenKeyW(hRootKey, pszKeyPath, core::ptr::addr_of_mut!(hKey))) as u32
            != ERROR_SUCCESS
        {
            break 'done;
        }

        if LoadStringW(
            hInst,
            IDS_NEW_KEY,
            szNewKeyFormat.as_mut_ptr(),
            ARRAY_SIZE!(szNewKeyFormat) as i32,
        ) <= 0
        {
            break 'done;
        }

        /* Need to create a new key with a unique name */
        loop {
            let fresh = iIndex;
            iIndex += 1;
            wsprintf(szNewKey.as_mut_ptr(), szNewKeyFormat.as_ptr(), fresh);
            nResult = RegCreateKeyExW(
                hKey,
                szNewKey.as_ptr(),
                0,
                core::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(hNewKey),
                core::ptr::addr_of_mut!(dwDisposition),
            );
            if !hNewKey.is_null() && dwDisposition == REG_OPENED_EXISTING_KEY {
                RegCloseKey(hNewKey);
                hNewKey = core::ptr::null_mut();
            } else if hNewKey.is_null() {
                InfoMessageBox(
                    hFrameWnd,
                    MB_OK | MB_ICONERROR,
                    core::ptr::null(),
                    RUSTCONV_WSTR_A8B389DD9483.as_ptr(),
                    nResult,
                );
                break 'done;
            }
            if !hNewKey.is_null() {
                break;
            }
        }

        /* Insert the new key */
        hNewItem = InsertNode(hwndTV, hItem, szNewKey.as_mut_ptr());
        if hNewItem.is_null() {
            break 'done;
        }

        /* The new key's name is probably not appropriate yet */
        let _ = TreeView_EditLabel!(hwndTV, hNewItem);

        bSuccess = (TRUE) as i32;
    }

    // done:
    if hKey != hRootKey && !hKey.is_null() {
        RegCloseKey(hKey);
    }
    if !hNewKey.is_null() {
        RegCloseKey(hNewKey);
    }
    return bSuccess;
}
#[no_mangle]
pub unsafe extern "C" fn TreeWndNotifyProc(hWnd: HWND, wParam: WPARAM, lParam: LPARAM, Result: *mut BOOL) -> BOOL {
    UNREFERENCED_PARAMETER!(wParam);
    *Result = (TRUE) as i32;

    match (*(lParam as LPNMHDR)).code {
        TVN_ITEMEXPANDING => {
            *Result = (OnTreeExpanding((*g_pChildWnd).hTreeWnd, lParam as *mut NMTREEVIEW) == 0) as BOOL;
            return (TRUE) as BOOL;
        }
        TVN_SELCHANGED => {
            let mut pnmtv: *mut NMTREEVIEW = lParam as *mut NMTREEVIEW;
            /* Get the parent of the current item */
            let mut hParentItem: HTREEITEM = TreeView_GetParent!((*g_pChildWnd).hTreeWnd, (*pnmtv).itemNew.hItem);

            UpdateAddress((*pnmtv).itemNew.hItem, core::ptr::null_mut(), core::ptr::null_mut(), (TRUE) as BOOL);

            /* Disable the Permissions and new key menu items for 'My Computer' */
            EnableMenuItem(hMenuFrame, ID_EDIT_PERMISSIONS, MF_BYCOMMAND | (if !hParentItem.is_null() { MF_ENABLED } else { MF_GRAYED }));
            EnableMenuItem(hMenuFrame, ID_EDIT_NEW_KEY, MF_BYCOMMAND | (if !hParentItem.is_null() { MF_ENABLED } else { MF_GRAYED }));

            /*
             * Disable Delete/Rename menu options for 'My Computer' (first item so doesn't have any parent)
             * and HKEY_* keys (their parent is 'My Computer' and the previous remark applies).
             */
            if hParentItem.is_null() || TreeView_GetParent!((*g_pChildWnd).hTreeWnd, hParentItem).is_null()
            {
                EnableMenuItem(hMenuFrame, ID_EDIT_DELETE, MF_BYCOMMAND | MF_GRAYED);
                EnableMenuItem(hMenuFrame, ID_EDIT_RENAME, MF_BYCOMMAND | MF_GRAYED);
                EnableMenuItem(hPopupMenus, ID_TREE_DELETE, MF_BYCOMMAND | MF_GRAYED);
                EnableMenuItem(hPopupMenus, ID_TREE_RENAME, MF_BYCOMMAND | MF_GRAYED);
            }
            else
            {
                EnableMenuItem(hMenuFrame, ID_EDIT_DELETE, MF_BYCOMMAND | MF_ENABLED);
                EnableMenuItem(hMenuFrame, ID_EDIT_RENAME, MF_BYCOMMAND | MF_ENABLED);
                EnableMenuItem(hPopupMenus, ID_TREE_DELETE, MF_BYCOMMAND | MF_ENABLED);
                EnableMenuItem(hPopupMenus, ID_TREE_RENAME, MF_BYCOMMAND | MF_ENABLED);
            }

            return (TRUE) as BOOL;
        }
        NM_SETFOCUS => {
            (*g_pChildWnd).nFocusPanel = 0;
        }
        TVN_BEGINLABELEDIT => {
            let mut ptvdi: LPNMTVDISPINFOW = lParam as LPNMTVDISPINFOW;
            let mut hWndFocus: HWND = GetFocus();

            /* cancel label edit for rootkeys */
            if TreeView_GetParent!((*g_pChildWnd).hTreeWnd, (*ptvdi).item.hItem).is_null() ||
                TreeView_GetParent!((*g_pChildWnd).hTreeWnd, TreeView_GetParent!((*g_pChildWnd).hTreeWnd, (*ptvdi).item.hItem)).is_null()
            {
                *Result = (TRUE) as i32;
            }
            else
            {
                *Result = (FALSE) as i32;
            }

            /* cancel label edit if VK_DELETE accelerator opened a MessageBox */
            if hWndFocus != (*g_pChildWnd).hTreeWnd && hWndFocus != TreeView_GetEditControl!((*g_pChildWnd).hTreeWnd)
            {
                *Result = (TRUE) as i32;
            }
            return (TRUE) as BOOL;
        }
        TVN_ENDLABELEDIT => {
            let mut keyPath: LPCWSTR;
            let mut hRootKey: HKEY = core::mem::zeroed();
            let mut hKey: HKEY = core::ptr::null_mut();
            let mut ptvdi: LPNMTVDISPINFOW = lParam as LPNMTVDISPINFOW;
            let mut nRenResult: LONG;
            let mut lResult: LONG = (TRUE) as i32;
            let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut Caption: [WCHAR; 128] = [0; 128];

            if !(*ptvdi).item.pszText.is_null()
            {
                keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, TreeView_GetParent!((*g_pChildWnd).hTreeWnd, (*ptvdi).item.hItem), &raw mut hRootKey);
                if wcslen(keyPath) != 0
                {
                    _snwprintf(szBuffer.as_mut_ptr(), ARRAY_SIZE!(szBuffer), RUSTCONV_WSTR_BA2342D760A2.as_ptr(), keyPath, (*ptvdi).item.pszText);
                }
                else
                {
                    _snwprintf(szBuffer.as_mut_ptr(), ARRAY_SIZE!(szBuffer), RUSTCONV_WSTR_FB47CCC40FE6.as_ptr(), (*ptvdi).item.pszText);
                }
                keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, (*ptvdi).item.hItem, &raw mut hRootKey);
                if (RegOpenKeyExW(hRootKey, szBuffer.as_mut_ptr(), 0, KEY_READ, &raw mut hKey)) as u32 == ERROR_SUCCESS
                {
                    lResult = (FALSE) as i32;
                    RegCloseKey(hKey);
                    TreeView_EditLabel!((*g_pChildWnd).hTreeWnd, (*ptvdi).item.hItem);
                }
                else
                {
                    nRenResult = RenameKey(hRootKey, keyPath, (*ptvdi).item.pszText);
                    if (nRenResult) as u32 != ERROR_SUCCESS
                    {
                        LoadStringW(hInst, IDS_ERROR, Caption.as_mut_ptr(), (ARRAY_SIZE!(Caption)) as i32);
                        ErrorMessageBox(hWnd, Caption.as_mut_ptr(), (nRenResult) as DWORD);
                        lResult = (FALSE) as i32;
                    }
                    else
                    {
                        UpdateAddress((*ptvdi).item.hItem, hRootKey, szBuffer.as_mut_ptr(), (FALSE) as BOOL);
                    }
                }
                *Result = lResult;
            }
            return (TRUE) as BOOL;
        }
        _ => {}
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CreateTreeView(hwndParent: HWND, pHostName: LPWSTR, id: HMENU) -> HWND {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut hwndTV: HWND;

    /* Get the dimensions of the parent window's client area, and create the tree view control.  */
    GetClientRect(hwndParent, core::ptr::addr_of_mut!(rcClient));
    hwndTV = CreateWindowExW(
        WS_EX_CLIENTEDGE as DWORD,
        WC_TREEVIEW as LPCWSTR,
        core::ptr::null(),
        (WS_VISIBLE | WS_CHILD | WS_TABSTOP | TVS_HASLINES | TVS_HASBUTTONS | TVS_LINESATROOT | TVS_EDITLABELS | TVS_SHOWSELALWAYS) as DWORD,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        rcClient.right,
        rcClient.bottom,
        hwndParent,
        id,
        hInst,
        core::ptr::null_mut(),
    );
    if hwndTV.is_null() {
        return core::ptr::null_mut();
    }

    /* Initialize the image list, and add items to the control.  */
    if InitTreeViewImageLists(hwndTV) == 0 || InitTreeViewItems(hwndTV, pHostName) == 0 {
        DestroyWindow(hwndTV);
        return core::ptr::null_mut();
    }
    return hwndTV;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyTreeView(hwndTV: HWND) {
    let mut himl: HIMAGELIST = core::ptr::null_mut();

    if !pathBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, pathBuffer as LPVOID);
    }

    /* Destroy the image list associated with the tree view control */
    himl = TreeView_GetImageList!(hwndTV, TVSIL_NORMAL as WPARAM) as HIMAGELIST;
    if !himl.is_null() {
        ImageList_Destroy(himl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SelectNode(hwndTV: HWND, mut keyPath: LPCWSTR) -> BOOL {
    let mut hRoot: HTREEITEM;
    let mut hItem: HTREEITEM;
    let mut hChildItem: HTREEITEM;
    let mut szPathPart: [WCHAR; 128] = [0; 128];
    let mut szBuffer: [WCHAR; 128] = [0; 128];
    let mut s: LPCWSTR = core::ptr::null();
    let mut tvi: TVITEMW = core::mem::zeroed();

    /* Load "My Computer" string... */
    LoadStringW(hInst, IDS_MY_COMPUTER, addr_of_mut!(szBuffer) as *mut WCHAR, (ARRAY_SIZE!(szBuffer)) as i32);
    StringCbCatW(addr_of_mut!(szBuffer) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_B7FC02907B1F.as_ptr());

    /* ... and remove it from the key path */
    if _wcsnicmp(keyPath, addr_of_mut!(szBuffer) as *mut WCHAR, wcslen(addr_of_mut!(szBuffer) as *mut WCHAR)) == 0 {
        keyPath = keyPath.offset(wcslen(addr_of_mut!(szBuffer) as *mut WCHAR) as isize);
    }

    /* Reinitialize szBuffer */
    szBuffer[0 as usize] = 0x0000u16;

    hRoot = TreeView_GetRoot!(hwndTV);
    hItem = hRoot;

    while *keyPath.offset(0 as isize) != 0 {
        let mut copyLength: usize;
        s = wcschr(keyPath, 0x005Cu16);
        if !s.is_null() {
            copyLength = (s.offset_from(keyPath) as usize) * core::mem::size_of::<WCHAR>();
        } else {
            copyLength = core::mem::size_of::<[WCHAR; 128]>();
        }
        StringCbCopyNW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), keyPath, copyLength);

        /* Special case for root to expand root key abbreviations */
        if hItem == hRoot {
            if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_8E490F9323D5.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_87D16031626E.as_ptr());
            } else if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_D02D0BCFE5D4.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_D0EE0F231465.as_ptr());
            } else if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_89F38F0B7FC9.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_50C930882486.as_ptr());
            } else if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_22FF61FCDE09.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_9C98935C198E.as_ptr());
            } else if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_BFE074CE3641.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_111FD57AB4CB.as_ptr());
            } else if _wcsicmp(addr_of_mut!(szPathPart) as *mut WCHAR, RUSTCONV_WSTR_C57F475C07AD.as_ptr()) == 0 {
                StringCbCopyW(addr_of_mut!(szPathPart) as *mut WCHAR, core::mem::size_of::<[WCHAR; 128]>(), RUSTCONV_WSTR_5CDE252B65C7.as_ptr());
            }
        }

        hChildItem = TreeView_GetChild!(hwndTV, hItem);
        while !hChildItem.is_null() {
            memset(addr_of_mut!(tvi) as *mut core::ffi::c_void, 0, core::mem::size_of::<TVITEMW>());
            tvi.hItem = hChildItem;
            tvi.mask = TVIF_TEXT | TVIF_CHILDREN;
            tvi.pszText = addr_of_mut!(szBuffer) as *mut WCHAR;
            tvi.cchTextMax = (ARRAY_SIZE!(szBuffer)) as i32;

            let _ = TreeView_GetItem!(hwndTV, addr_of_mut!(tvi));

            if _wcsicmp(addr_of_mut!(szBuffer) as *mut WCHAR, addr_of_mut!(szPathPart) as *mut WCHAR) == 0 {
                break;
            }
            hChildItem = TreeView_GetNextSibling!(hwndTV, hChildItem);
        }

        if hChildItem.is_null() {
            return (FALSE) as BOOL;
        }

        if tvi.cChildren > 0 {
            if TreeView_Expand!(hwndTV, hChildItem, TVE_EXPAND) == 0 {
                return (FALSE) as BOOL;
            }
        }

        keyPath = if !s.is_null() { s.offset(1 as isize) } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() };
        hItem = hChildItem;
    }

    let _ = TreeView_SelectItem!(hwndTV, hItem);
    let _ = TreeView_EnsureVisible!(hwndTV, hItem);

    return (TRUE) as BOOL;
}
