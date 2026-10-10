use crate::servman::misc::InitImageList;
use crate::servman::dependencies_tv1::TV1_AddDependantsToTree;
use crate::servman::dependencies_tv1::TV1_Initialize;
use crate::servman::dependencies_tv2::TV2_AddDependantsToTree;
use crate::servman::dependencies_tv2::TV2_Initialize;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn AddItemToTreeView(hTreeView: HWND, hParent: HTREEITEM, lpDisplayName: LPWSTR, lpServiceName: LPWSTR, ServiceType: ULONG, bHasChildren: BOOL) -> HTREEITEM {
    let mut tvi: TVITEMW = core::mem::zeroed();
    let mut tvins: TVINSERTSTRUCTW = core::mem::zeroed();
    let mut lpName: LPWSTR = core::ptr::null_mut();
    let mut dwSize: DWORD;

    ZeroMemory!(
        core::ptr::addr_of_mut!(tvi) as *mut core::ffi::c_void,
        core::mem::size_of::<TVITEMW>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(tvins) as *mut core::ffi::c_void,
        core::mem::size_of::<TVINSERTSTRUCTW>(),
    );

    tvi.mask = (TVIF_TEXT | TVIF_PARAM | TVIF_SELECTEDIMAGE | TVIF_IMAGE | TVIF_CHILDREN) as UINT;
    tvi.pszText = lpDisplayName;
    tvi.cchTextMax = wcslen(lpDisplayName as *const u16) as i32;
    tvi.cChildren = bHasChildren;

    /* Select the image for this service */
    if ServiceType == SERVICE_WIN32_OWN_PROCESS || ServiceType == SERVICE_WIN32_SHARE_PROCESS {
        tvi.iImage = (IMAGE_SERVICE as i32);
        tvi.iSelectedImage = (IMAGE_SERVICE as i32);
    } else if ServiceType == SERVICE_KERNEL_DRIVER || ServiceType == SERVICE_FILE_SYSTEM_DRIVER {
        tvi.iImage = (IMAGE_DRIVER as i32);
        tvi.iSelectedImage = (IMAGE_DRIVER as i32);
    } else {
        tvi.iImage = (IMAGE_UNKNOWN as i32);
        tvi.iSelectedImage = (IMAGE_UNKNOWN as i32);
    }

    if !lpServiceName.is_null() {
        dwSize = (wcslen(lpServiceName as *const u16) + 1 as usize) as DWORD;
        /* Attach the service name */
        lpName = HeapAlloc(
            GetProcessHeap(),
            0,
            ((dwSize as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPWSTR;
        if !lpName.is_null() {
            StringCchCopyW(
                lpName as *mut u16,
                dwSize as usize,
                lpServiceName as *const u16,
            );
            tvi.lParam = lpName as LPARAM;
        }
    }

    tvins.__bindgen_anon_1.item = tvi;
    tvins.hParent = hParent;

    TreeView_InsertItem!(hTreeView, core::ptr::addr_of_mut!(tvins)) as HTREEITEM
}

unsafe extern "C" fn TreeView_GetItemParam(hTreeView: HWND, hItem: HTREEITEM) -> LPARAM {
    let mut lParam: LPARAM = 0;
    let mut tv: TVITEMW = core::mem::zeroed();

    tv.mask = (TVIF_PARAM | TVIF_HANDLE) as UINT;
    tv.hItem = hItem;

    if TreeView_GetItem!(hTreeView, core::ptr::addr_of_mut!(tv)) != 0
    {
        lParam = tv.lParam;
    }

    return lParam;
}

unsafe extern "C" fn DestroyItem(hTreeView: HWND, hItem: HTREEITEM) {
    let mut hChildItem: HTREEITEM = core::ptr::null_mut();
    let mut lpServiceName: LPWSTR = core::ptr::null_mut();

    /* Does this item have any children */
    hChildItem = TreeView_GetChild!(hTreeView, hItem);
    if !hChildItem.is_null() {
        /* It does, recurse to that one */
        DestroyItem(hTreeView, hChildItem);
    }

    /* Get the string and free it */
    lpServiceName = TreeView_GetItemParam(hTreeView, hItem) as LPWSTR;
    if !lpServiceName.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            lpServiceName as PVOID,
        );
    }
}

unsafe extern "C" fn DestroyTreeView(hTreeView: HWND) {
    let mut hItem: HTREEITEM;

    /* Get the first item in the top level */
    hItem = TreeView_GetFirstVisible!(hTreeView);
    if !hItem.is_null() {
        /* Kill it and all children */
        DestroyItem(hTreeView, hItem);

        /* Kill all remaining top level items */
        while !hItem.is_null() {
            /* Are there any more items at the top level */
            hItem = TreeView_GetNextSibling!(hTreeView, hItem);
            if !hItem.is_null() {
                /*  Kill it and all children */
                DestroyItem(hTreeView, hItem);
            }
        }
    }
}

unsafe extern "C" fn InitDependPage(pDependData: PDEPENDDATA) {
    /* Initialize the image list */
    (*pDependData).hDependsImageList = InitImageList(
        IDI_NODEPENDS as UINT,
        IDI_DRIVER as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32) as UINT,
        GetSystemMetrics((SM_CXSMICON) as i32) as UINT,
        IMAGE_ICON as ULONG,
    );

    /* Set the first tree view */
    TV1_Initialize(
        pDependData,
        (*(*(*pDependData).pDlgInfo).pService).lpServiceName,
    );

    /* Set the second tree view */
    TV2_Initialize(
        pDependData,
        (*(*(*pDependData).pDlgInfo).pService).lpServiceName,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DependenciesPageProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pDependData: PDEPENDDATA;

    /* Get the window context */
    pDependData = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PDEPENDDATA;
    if pDependData.is_null() && uMsg != WM_INITDIALOG as UINT {
        return FALSE as INT_PTR;
    }

    match uMsg {
        WM_INITDIALOG => {
            pDependData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<DEPENDDATA>() as SIZE_T,
            ) as PDEPENDDATA;
            if !pDependData.is_null() {
                SetWindowLongPtr(
                    hwndDlg,
                    GWLP_USERDATA,
                    pDependData as LONG_PTR,
                );

                (*pDependData).pDlgInfo =
                    (*(lParam as LPPROPSHEETPAGE)).lParam as PSERVICEPROPSHEET;
                (*pDependData).hDependsWnd = hwndDlg;

                InitDependPage(pDependData);
            }
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code {
                TVN_ITEMEXPANDING => {
                    let lpnmtv: LPNMTREEVIEW = lParam as LPNMTREEVIEW;

                    if (*lpnmtv).action == TVE_EXPAND as UINT {
                        if (*lpnmtv).hdr.idFrom == IDC_DEPEND_TREE1 as UINT_PTR {
                            /* Has this node been expanded before */
                            if TreeView_GetChild!((*pDependData).hDependsTreeView1, (*lpnmtv).itemNew.hItem).is_null() {
                                /* It's not, add the children */
                                TV1_AddDependantsToTree(
                                    pDependData,
                                    (*lpnmtv).itemNew.hItem,
                                    (*lpnmtv).itemNew.lParam as LPWSTR,
                                );
                            }
                        } else if (*lpnmtv).hdr.idFrom == IDC_DEPEND_TREE2 as UINT_PTR {
                            /* Has this node been expanded before */
                            if TreeView_GetChild!((*pDependData).hDependsTreeView2, (*lpnmtv).itemNew.hItem).is_null() {
                                /* It's not, add the children */
                                TV2_AddDependantsToTree(
                                    pDependData,
                                    (*lpnmtv).itemNew.hItem,
                                    (*lpnmtv).itemNew.lParam as LPWSTR,
                                );
                            }
                        }
                    }
                    // break;
                }
                _ => {}
            }
            // break;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) {
                _ => {}
            }
            // break;
        }

        WM_DESTROY => {
            DestroyTreeView((*pDependData).hDependsTreeView1);
            DestroyTreeView((*pDependData).hDependsTreeView2);

            if !(*pDependData).hDependsImageList.is_null() {
                ImageList_Destroy((*pDependData).hDependsImageList);
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pDependData as PVOID,
            );
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
