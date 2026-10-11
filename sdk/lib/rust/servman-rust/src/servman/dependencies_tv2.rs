use crate::servman::propsheet_depends::AddItemToTreeView;
use crate::servman::misc::AllocAndLoadString;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn TV2_HasDependantServices(mut lpServiceName: LPWSTR) -> BOOL {
    let mut hSCManager: HANDLE = core::ptr::null_mut();
    let mut hService: HANDLE = core::ptr::null_mut();
    let mut dwBytesNeeded: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut bRet: BOOL = FALSE as BOOL;
    hSCManager = OpenSCManagerW(
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<WCHAR>(),
        SC_MANAGER_ALL_ACCESS as DWORD,
    ) as HANDLE;
    if !hSCManager.is_null() {
        hService = OpenServiceW(
            hSCManager as SC_HANDLE,
            lpServiceName as LPCWSTR,
            (SERVICE_QUERY_STATUS | SERVICE_ENUMERATE_DEPENDENTS) as DWORD,
        ) as HANDLE;
        if !hService.is_null() {
            /* Does this have any dependencies? */
            if EnumDependentServices(
                hService as SC_HANDLE,
                SERVICE_STATE_ALL as DWORD,
                core::ptr::null_mut::<_ENUM_SERVICE_STATUSW>(),
                0 as DWORD,
                core::ptr::addr_of_mut!(dwBytesNeeded),
                core::ptr::addr_of_mut!(dwCount),
            ) == 0
            {
                if GetLastError() == ERROR_MORE_DATA as DWORD {
                    /* It does, return TRUE */
                    bRet = TRUE as BOOL;
                }
            }

            CloseServiceHandle(hService as SC_HANDLE);
        }

        CloseServiceHandle(hSCManager as SC_HANDLE);
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn TV2_GetDependants(mut lpServiceName: LPWSTR, mut lpdwCount: LPDWORD) -> LPENUM_SERVICE_STATUS {
    let mut hSCManager: SC_HANDLE = ::core::ptr::null_mut::<SC_HANDLE__>();
    let mut hService: SC_HANDLE = ::core::ptr::null_mut::<SC_HANDLE__>();
    let mut lpDependencies: LPENUM_SERVICE_STATUSW =
        ::core::ptr::null_mut::<_ENUM_SERVICE_STATUSW>();
    let mut dwBytesNeeded: DWORD = 0;
    let mut dwCount: DWORD = 0;

    /* Set the first items in each tree view */
    hSCManager = OpenSCManagerW(
        ::core::ptr::null::<WCHAR>(),
        ::core::ptr::null::<WCHAR>(),
        SC_MANAGER_ALL_ACCESS as DWORD,
    );
    if !hSCManager.is_null() {
        hService = OpenServiceW(
            hSCManager,
            lpServiceName as LPCWSTR,
            (SERVICE_QUERY_STATUS | SERVICE_ENUMERATE_DEPENDENTS | SERVICE_QUERY_CONFIG) as DWORD,
        );
        if !hService.is_null() {
            /* Does this have any dependencies? */
            if EnumDependentServicesW(
                hService,
                SERVICE_STATE_ALL as DWORD,
                ::core::ptr::null_mut::<_ENUM_SERVICE_STATUSW>(),
                0 as DWORD,
                &raw mut dwBytesNeeded,
                &raw mut dwCount,
            ) == 0
                && GetLastError() == ERROR_MORE_DATA as DWORD
            {
                lpDependencies = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    dwBytesNeeded as SIZE_T,
                ) as LPENUM_SERVICE_STATUSW;
                if !lpDependencies.is_null() {
                    /* Get the list of dependents */
                    if EnumDependentServicesW(
                        hService,
                        SERVICE_STATE_ALL as DWORD,
                        lpDependencies,
                        dwBytesNeeded,
                        &raw mut dwBytesNeeded,
                        &raw mut dwCount,
                    ) != 0
                    {
                        /* Set the count */
                        *lpdwCount = dwCount;
                    } else {
                        HeapFree(
                            ProcessHeap,
                            0 as DWORD,
                            lpDependencies as PVOID,
                        );

                        lpDependencies = ::core::ptr::null_mut::<_ENUM_SERVICE_STATUSW>();
                    }
                }
            }

            CloseServiceHandle(hService);
        }

        CloseServiceHandle(hSCManager);
    }

    return lpDependencies as LPENUM_SERVICE_STATUS;
}
#[no_mangle]
pub unsafe extern "C" fn TV2_AddDependantsToTree(mut pDependData: PDEPENDDATA, mut hParent: HTREEITEM, mut lpServiceName: LPWSTR) {
    let mut lpServiceStatus: LPENUM_SERVICE_STATUSW =
        ::core::ptr::null_mut::<_ENUM_SERVICE_STATUSW>();
    let mut lpNoDepends: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut count: DWORD = 0;
    let mut i: DWORD = 0;
    let mut bHasChildren: BOOL = 0;
    lpServiceStatus = TV2_GetDependants(lpServiceName, core::ptr::addr_of_mut!(count))
        as LPENUM_SERVICE_STATUSW;
    if !lpServiceStatus.is_null() {
        i = 0;
        while i < count {
            bHasChildren =
                TV2_HasDependantServices((*lpServiceStatus.offset(i as isize)).lpServiceName);
            AddItemToTreeView(
                (*pDependData).hDependsTreeView2,
                hParent,
                (*lpServiceStatus.offset(i as isize)).lpDisplayName,
                (*lpServiceStatus.offset(i as isize)).lpServiceName,
                (*lpServiceStatus.offset(i as isize))
                    .ServiceStatus
                    .dwServiceType as ULONG,
                bHasChildren,
            );
            i = i.wrapping_add(1);
        }
        HeapFree(
            GetProcessHeap(),
            0,
            lpServiceStatus as PVOID,
        );
    } else if hParent.is_null() {
        AllocAndLoadString(
            core::ptr::addr_of_mut!(lpNoDepends),
            hInstance,
            IDS_NO_DEPENDS as UINT,
        );
        AddItemToTreeView(
            (*pDependData).hDependsTreeView2,
            core::ptr::null_mut::<_TREEITEM>(),
            lpNoDepends,
            core::ptr::null_mut::<WCHAR>(),
            0 as ULONG,
            (FALSE) as BOOL,
        );
        LocalFree(lpNoDepends as HLOCAL);
        EnableWindow((*pDependData).hDependsTreeView2, (FALSE) as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn TV2_Initialize(mut pDependData: PDEPENDDATA, mut lpServiceName: LPWSTR) -> BOOL {
    let mut bRet: BOOL = (FALSE) as i32;

    /* Associate the imagelist with TV2 */
    (*pDependData).hDependsTreeView2 = GetDlgItem((*pDependData).hDependsWnd, (IDC_DEPEND_TREE2 as i32));
    if (*pDependData).hDependsTreeView2.is_null() {
        ImageList_Destroy((*pDependData).hDependsImageList);
        (*pDependData).hDependsImageList = core::ptr::null_mut();
        return (FALSE) as BOOL;
    }
    let _ = TreeView_SetImageList!(
        (*pDependData).hDependsTreeView2,
        (*pDependData).hDependsImageList,
        TVSIL_NORMAL,
    );

    /* Set the first items in the control */
    TV2_AddDependantsToTree(pDependData, core::ptr::null_mut(), lpServiceName);

    return bRet;
}
