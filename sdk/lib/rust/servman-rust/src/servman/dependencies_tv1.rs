use crate::servman::propsheet_depends::AddItemToTreeView;
use crate::servman::misc::AllocAndLoadString;
use crate::servman::query::GetServiceConfig;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn TV1_GetDependants(hService: SC_HANDLE) -> LPWSTR {
    let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut lpStr: LPWSTR = core::ptr::null_mut();
    let mut bytesNeeded: DWORD = 0;
    let mut bytes: DWORD = 0;

    /* Get the info for this service */
    if QueryServiceConfigW(
        hService,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(bytesNeeded),
    ) == 0
        && GetLastError() == ERROR_INSUFFICIENT_BUFFER
    {
        lpServiceConfig = HeapAlloc(
            ProcessHeap,
            0,
            bytesNeeded as SIZE_T,
        ) as LPQUERY_SERVICE_CONFIG;
        if !lpServiceConfig.is_null() {
            if QueryServiceConfigW(
                hService,
                lpServiceConfig as LPQUERY_SERVICE_CONFIGW,
                bytesNeeded,
                core::ptr::addr_of_mut!(bytesNeeded),
            ) != 0
            {
                /* Does this service have any dependencies? */
                if !(*lpServiceConfig).lpDependencies.is_null()
                    && *(*lpServiceConfig).lpDependencies != 0
                {
                    lpStr = (*lpServiceConfig).lpDependencies;
                    bytes = 0;

                    /* Work out how many bytes we need to hold the list */
                    loop {
                        bytes += 1;

                        if *lpStr == 0 && *lpStr.offset(1 as isize) == 0 {
                            bytes += 1;
                            break;
                        }

                        lpStr = lpStr.offset(1);
                    }

                    /* Allocate and copy the list */
                    bytes *= core::mem::size_of::<WCHAR>() as DWORD;
                    lpStr = HeapAlloc(
                        ProcessHeap,
                        0,
                        bytes as SIZE_T,
                    ) as LPWSTR;
                    if !lpStr.is_null() {
                        CopyMemory!(
                            lpStr as *mut core::ffi::c_void,
                            (*lpServiceConfig).lpDependencies as *const core::ffi::c_void,
                            bytes as usize,
                        );
                    }
                }
            }

            HeapFree(
                GetProcessHeap(),
                0,
                lpServiceConfig as PVOID,
            );
        }
    }

    lpStr
}
#[no_mangle]
pub unsafe extern "C" fn TV1_AddDependantsToTree(pDependData: PDEPENDDATA, hParent: HTREEITEM, lpServiceName: LPWSTR) {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut lpServiceConfig: LPQUERY_SERVICE_CONFIG = core::ptr::null_mut();
    let mut lpDependants: LPWSTR = core::ptr::null_mut();
    let mut lpStr: LPWSTR = core::ptr::null_mut();
    let mut lpNoDepends: LPWSTR = core::ptr::null_mut();
    let mut bHasChildren: BOOL = 0;

    hSCManager = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_ALL_ACCESS as DWORD,
    );
    if !hSCManager.is_null() {
        hService = OpenServiceW(
            hSCManager,
            lpServiceName as LPCWSTR,
            (SERVICE_QUERY_STATUS | SERVICE_ENUMERATE_DEPENDENTS | SERVICE_QUERY_CONFIG) as DWORD,
        );
        if !hService.is_null() {
            /* Get a list of service dependents */
            lpDependants = TV1_GetDependants(hService);
            if !lpDependants.is_null() {
                lpStr = lpDependants;

                /* Make sure this isn't the end of the list */
                while *lpStr != 0 {
                    /* Get the info for this service */
                    lpServiceConfig = GetServiceConfig(lpStr);
                    if !lpServiceConfig.is_null() {
                        /* Does this item need a +/- box? */
                        if !(*lpServiceConfig).lpDependencies.is_null()
                            && *(*lpServiceConfig).lpDependencies != 0
                        {
                            bHasChildren = (TRUE) as i32;
                        } else {
                            bHasChildren = (FALSE) as i32;
                        }

                        /* Add it */
                        AddItemToTreeView(
                            (*pDependData).hDependsTreeView1,
                            hParent,
                            (*lpServiceConfig).lpDisplayName,
                            lpStr,
                            (*lpServiceConfig).dwServiceType as ULONG,
                            bHasChildren,
                        );

                        HeapFree(
                            GetProcessHeap(),
                            0 as DWORD,
                            lpServiceConfig as PVOID,
                        );
                    }

                    /* Move to the end of the string */
                    loop {
                        let fresh31 = lpStr;
                        lpStr = lpStr.offset(1);
                        if !(*fresh31 != 0) {
                            break;
                        }
                    }
                }

                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    lpDependants as PVOID,
                );
            } else {
                /* If there is no parent, set the tree to 'no dependencies' */
                if hParent.is_null() {
                    /* Load the 'No dependencies' string */
                    AllocAndLoadString(
                        core::ptr::addr_of_mut!(lpNoDepends),
                        hInstance,
                        IDS_NO_DEPENDS as UINT,
                    );

                    AddItemToTreeView(
                        (*pDependData).hDependsTreeView1,
                        core::ptr::null_mut(),
                        lpNoDepends,
                        core::ptr::null_mut(),
                        0 as ULONG,
                        (FALSE) as BOOL,
                    );

                    LocalFree(lpNoDepends as HLOCAL);

                    /* Disable the window */
                    EnableWindow((*pDependData).hDependsTreeView1, (FALSE) as BOOL);
                }
            }

            CloseServiceHandle(hService);
        }

        CloseServiceHandle(hSCManager);
    }
}
#[no_mangle]
pub unsafe extern "C" fn TV1_Initialize(pDependData: PDEPENDDATA, lpServiceName: LPWSTR) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;

    /* Associate the imagelist with TV1 */
    (*pDependData).hDependsTreeView1 = GetDlgItem((*pDependData).hDependsWnd, (IDC_DEPEND_TREE1 as i32));
    if (*pDependData).hDependsTreeView1.is_null() {
        ImageList_Destroy((*pDependData).hDependsImageList);
        (*pDependData).hDependsImageList = core::ptr::null_mut();
        return (FALSE) as BOOL;
    }
    let _ = TreeView_SetImageList!(
        (*pDependData).hDependsTreeView1,
        (*pDependData).hDependsImageList,
        TVSIL_NORMAL,
    );

    /* Set the first items in the control */
    TV1_AddDependantsToTree(pDependData, core::ptr::null_mut(), lpServiceName);

    return bRet;
}
