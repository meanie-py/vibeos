use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct INetCfgComponentImpl {
    pub lpVtbl: *const INetCfgComponent,
    pub lpVtblBindings: *const INetCfgComponentBindings,
    pub lpVtblPrivate: *const INetCfgComponentPrivate,
    pub ref_: LONG,
    pub pItem: *mut NetCfgComponentItem,
    pub pProperty: *mut INetCfgComponentPropertyUi,
    pub pNCfg: *mut INetCfg,
}

pub type PINetCfgComponentImpl = *mut INetCfgComponentImpl;

#[repr(C)]
pub struct IEnumNetCfgComponentImpl {
    pub lpVtbl: *const IEnumNetCfgComponent,
    pub ref_: LONG,
    pub pCurrent: *mut NetCfgComponentItem,
    pub pHead: *mut NetCfgComponentItem,
    pub pNCfg: *mut INetCfg,
}

pub type PIEnumNetCfgComponentImpl = *mut IEnumNetCfgComponentImpl;
unsafe extern "C" fn impl_from_INetCfgComponentBindings(iface: *mut INetCfgComponentBindings) -> *mut INetCfgComponentImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(INetCfgComponentImpl, lpVtblBindings) as isize)) as *mut INetCfgComponentImpl
}
unsafe extern "C" fn impl_from_INetCfgComponentPrivate(iface: *mut INetCfgComponentPrivate) -> *mut INetCfgComponentImpl {
    (iface as *mut u8).offset(-(FIELD_OFFSET!(INetCfgComponentImpl, lpVtblPrivate) as isize)) as *mut INetCfgComponentImpl
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnQueryInterface(
    iface: *mut INetCfgComponentBindings,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_INetCfgComponentBindings(iface);
    INetCfgComponent_QueryInterface(This as *mut INetCfgComponent, iid, ppvObj)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnAddRef(
    iface: *mut INetCfgComponentBindings
) -> u32 {
    let This = impl_from_INetCfgComponentBindings(iface);
    INetCfgComponent_AddRef(This as *mut INetCfgComponent)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnRelease(
    iface: *mut INetCfgComponentBindings
) -> ULONG {
    let This = impl_from_INetCfgComponentBindings(iface);
    INetCfgComponent_Release(This as *mut INetCfgComponent)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnBindTo(
    iface: *mut INetCfgComponentBindings,
    pnccItem: *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnUnbindFrom(
    iface: *mut INetCfgComponentBindings,
    pnccItem: *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnSupportsBindingInterface(
    iface: *mut INetCfgComponentBindings,
    dwFlags: DWORD,
    pszwInterfaceName: LPCWSTR,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnIsBoundTo(
    iface: *mut INetCfgComponentBindings,
    pnccItem: *mut INetCfgComponent,
) -> HRESULT {
    let pComponent: *mut INetCfgComponentImpl;
    let mut pszBindName: PWSTR;
    let mut ptr: PWSTR;
    let mut len: i32;

    pComponent = impl_from_INetCfgComponentBindings(iface);
    if pComponent.is_null()
        || (*pComponent).pItem.is_null()
        || (*(*pComponent).pItem).pszBinding.is_null()
    {
        return E_POINTER;
    }

    if pnccItem.is_null()
        || (*(pnccItem as *mut INetCfgComponentImpl)).pItem.is_null()
        || (*(*(pnccItem as *mut INetCfgComponentImpl)).pItem)
            .szBindName
            .is_null()
    {
        return E_POINTER;
    }

    pszBindName = (*(*(pnccItem as *mut INetCfgComponentImpl)).pItem).szBindName;

    ptr = (*(*pComponent).pItem).pszBinding;
    while *ptr != UNICODE_NULL {
        len = (wcslen(ptr)) as i32;

        if len > 8
            && _wcsicmp(ptr.offset(8), pszBindName)
                == 0
        {
            return S_OK;
        }

        ptr = ptr.offset(len as isize + 1);
    }

    return S_FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnIsBindableTo(
    iface: *mut INetCfgComponentBindings,
    pnccItem: *mut INetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnEnumBindingPaths(
    iface: *mut INetCfgComponentBindings,
    dwFlags: u32,
    ppIEnum: *mut *mut IEnumNetCfgBindingPath,
) -> HRESULT {
    return IEnumNetCfgBindingPath_Constructor(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IEnumNetCfgBindingPath),
        ppIEnum as *mut LPVOID,
        dwFlags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnMoveBefore(
    iface: *mut INetCfgComponentBindings,
    dwFlags: DWORD,
    pncbItemSrc: *mut INetCfgBindingPath,
    pncbItemDest: *mut INetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentBindings_fnMoveAfter(
    iface: *mut INetCfgComponentBindings,
    dwFlags: DWORD,
    pncbItemSrc: *mut INetCfgBindingPath,
    pncbItemDest: *mut INetCfgBindingPath,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPrivate_fnQueryInterface(
    iface: *mut INetCfgComponentPrivate,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_INetCfgComponentPrivate(iface);
    INetCfgComponent_QueryInterface(This as *mut INetCfgComponent, iid, ppvObj)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPrivate_fnAddRef(
    iface: *mut INetCfgComponentPrivate
) -> ULONG {
    let This = impl_from_INetCfgComponentPrivate(iface);
    INetCfgComponent_AddRef(This as *mut INetCfgComponent)
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponentPrivate_fnRelease(
    iface: *mut INetCfgComponentPrivate
) -> ULONG {
    let This = impl_from_INetCfgComponentPrivate(iface);
    INetCfgComponent_Release(This as *mut INetCfgComponent)
}

#[no_mangle]
// Upstream-sync 2026-09-20. This vtable slot is `Unknown1(THIS_ REFIID riid, void **ppv)`
// (sdk/include/reactos/netcfgx_undoc.h) — a QueryInterface-shaped call, NOT the two DWORDs
// the pre-merge stub was translated from. Upstream has since given it a real body:
// CreateNotifyObject(This, ...) then delegate to This->pItem->pControl's QueryInterface.
// Neither CreateNotifyObject nor the pItem/pControl fields exist in netcfgx-rust yet, so
// the body stays unimplemented — but the SIGNATURE and the RETURN are fixed here, because
// the merge made this reachable and dangerous:
//
//   dll/win32/ifmon/ip.c:189 (upstream's new `netsh interface ip`, +2409 lines, which we
//   now build) does
//       hr = INetCfgComponentPrivate_Unknown1(p, &IID_ITcpipProperties, (PVOID*)&pProps);
//       if (hr != S_OK) goto exit;
//   and then dereferences pProps. The old stub returned S_OK WITHOUT WRITING ppv, so the
//   caller sailed past its own error check straight into an uninitialized stack pointer.
//
// Returning E_NOTIMPL with *ppv cleared makes the caller take the path it already has.
pub unsafe extern "system" fn INetCfgComponentPrivate_fnUnknown1(
    _iface: *mut INetCfgComponentPrivate,
    _riid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    if !ppvObj.is_null() {
        *ppvObj = core::ptr::null_mut();
    }
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnQueryInterface(
    iface: *mut INetCfgComponent,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_INetCfgComponent)
    {
        *ppvObj = This as LPVOID;
        INetCfg_AddRef(iface as *mut INetCfg);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgComponentBindings)
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblBindings) as LPVOID;
        INetCfgComponentBindings_AddRef(iface as *mut INetCfgComponentBindings);
        return S_OK;
    }
    else if IsEqualIID!(iid, &IID_INetCfgComponentPrivate)
    {
        *ppvObj = core::ptr::addr_of_mut!((*This).lpVtblPrivate) as LPVOID;
        INetCfgComponentPrivate_AddRef(iface as *mut INetCfgComponentPrivate);
        return S_OK;
    }

    E_NOINTERFACE
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnAddRef(
    iface: *mut INetCfgComponent
) -> ULONG {
    let This = iface as *mut INetCfgComponentImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    return (refCount) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnRelease(
    iface: *mut INetCfgComponent
) -> ULONG {
    let This = iface as *mut INetCfgComponentImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    if refCount == 0 {
        CoTaskMemFree(This as LPVOID);
    }
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetDisplayName(
    iface: *mut INetCfgComponent,
    ppszwDisplayName: *mut LPWSTR,
) -> HRESULT {
    let szName: LPWSTR;
    let Length: u32;
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwDisplayName.is_null() {
        return E_POINTER;
    }

    if !(*This).pItem.is_null() && !(*(*This).pItem).szDisplayName.is_null() {
        Length = (wcslen((*(*This).pItem).szDisplayName) + 1) as u32;
    } else {
        Length = 1;
    }

    szName = CoTaskMemAlloc(((Length as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
    if szName.is_null() {
        return E_OUTOFMEMORY;
    }

    if Length > 1 {
        wcscpy(szName, (*(*This).pItem).szDisplayName);
    } else {
        *szName.offset(0 as isize) = 0u16;
    }

    *ppszwDisplayName = szName;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnSetDisplayName(
    iface: *mut INetCfgComponent,
    ppszwDisplayName: LPCWSTR,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwDisplayName.is_null() {
        return E_POINTER;
    }

    if !IsEqualGUID!(
        core::ptr::addr_of_mut!((*(*This).pItem).ClassGUID),
        core::ptr::addr_of!(GUID_DEVCLASS_NET),
    )
    {
        return E_NOTIMPL;
    }

    let szName = CoTaskMemAlloc(((wcslen(ppszwDisplayName) + 1)
            * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if szName.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy(szName, ppszwDisplayName);
    CoTaskMemFree((*(*This).pItem).szDisplayName as LPVOID);
    (*(*This).pItem).szDisplayName = szName;
    (*(*This).pItem).bChanged = (TRUE) as i32;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetHelpText(
    iface: *mut INetCfgComponent,
    ppszwHelpText: *mut LPWSTR,
) -> HRESULT {
    let szHelp: LPWSTR;
    let Length: u32;
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwHelpText.is_null() {
        return E_POINTER;
    }

    if !(*(*This).pItem).szHelpText.is_null() {
        Length = (wcslen((*(*This).pItem).szHelpText) + 1) as u32;
    } else {
        Length = 1;
    }

    szHelp = CoTaskMemAlloc(((Length as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
    if szHelp.is_null() {
        return E_OUTOFMEMORY;
    }

    if Length > 1 {
        wcscpy(szHelp, (*(*This).pItem).szHelpText);
    } else {
        *szHelp.offset(0 as isize) = 0u16;
    }

    *ppszwHelpText = szHelp;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetId(
    iface: *mut INetCfgComponent,
    ppszwId: *mut LPWSTR,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwId.is_null() {
        return E_POINTER;
    }

    let szId = CoTaskMemAlloc(((wcslen((*(*This).pItem).szId) + 1)
            * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if szId.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy(szId, (*(*This).pItem).szId);
    *ppszwId = szId;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetCharacteristics(
    iface: *mut INetCfgComponent,
    pdwCharacteristics: *mut DWORD,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || pdwCharacteristics.is_null() {
        return E_POINTER;
    }

    *pdwCharacteristics = (*(*This).pItem).dwCharacteristics;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetInstanceGuid(
    iface: *mut INetCfgComponent,
    pGuid: *mut GUID,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || pGuid.is_null() {
        return E_POINTER;
    }

    CopyMemory!(
        pGuid as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*(*This).pItem).InstanceId) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>(),
    );
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetPnpDevNodeId(
    iface: *mut INetCfgComponent,
    ppszwDevNodeId: *mut LPWSTR,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwDevNodeId.is_null() {
        return E_POINTER;
    }

    if !IsEqualGUID!(
        &GUID_DEVCLASS_NET,
        &(*(*This).pItem).ClassGUID,
    )
    {
        return E_NOTIMPL;
    }

    let szNode = CoTaskMemAlloc(((wcslen((*(*This).pItem).szNodeId) + 1)
            * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if szNode.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy(szNode, (*(*This).pItem).szNodeId);
    *ppszwDevNodeId = szNode;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetClassGuid(
    iface: *mut INetCfgComponent,
    pGuid: *mut GUID,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || pGuid.is_null() {
        return E_POINTER;
    }

    CopyMemory!(
        pGuid as LPVOID,
        core::ptr::addr_of!((*(*This).pItem).ClassGUID) as LPVOID,
        core::mem::size_of::<GUID>(),
    );
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetBindName(
    iface: *mut INetCfgComponent,
    ppszwBindName: *mut LPWSTR,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || ppszwBindName.is_null() {
        return E_POINTER;
    }

    let szBind = CoTaskMemAlloc(((wcslen((*(*This).pItem).szBindName) + 1)
            * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;

    if szBind.is_null() {
        return E_OUTOFMEMORY;
    }

    wcscpy(szBind, (*(*This).pItem).szBindName);
    *ppszwBindName = szBind;

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnGetDeviceStatus(
    iface: *mut INetCfgComponent,
    pStatus: *mut ULONG,
) -> HRESULT {
    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || pStatus.is_null() {
        return E_POINTER;
    }

    if !IsEqualGUID!(
        &GUID_DEVCLASS_NET as *const GUID,
        &(*(*This).pItem).ClassGUID as *const GUID,
    )
    {
        return E_UNEXPECTED;
    }

    *pStatus = (*(*This).pItem).Status;

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnOpenParamKey(
    iface: *mut INetCfgComponent,
    phkey: *mut HKEY,
) -> HRESULT {
    let mut szBuffer: [u16; 200] = [0; 200];
    let prefix = [
        'S' as u16, 'Y' as u16, 'S' as u16, 'T' as u16, 'E' as u16, 'M' as u16,
        '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16,
        'n' as u16, 't' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 't' as u16,
        'r' as u16, 'o' as u16, 'l' as u16, 'S' as u16, 'e' as u16, 't' as u16,
        '\\' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16,
        'c' as u16, 'e' as u16, 's' as u16, '\\' as u16, 0,
    ];
    core::ptr::copy_nonoverlapping(prefix.as_ptr(), szBuffer.as_mut_ptr(), prefix.len());

    let This = iface as *mut INetCfgComponentImpl;

    if This.is_null() || phkey.is_null() {
        return E_POINTER;
    }

    wcscat(szBuffer.as_mut_ptr(), (*(*This).pItem).szBindName);
    wcscat(szBuffer.as_mut_ptr(), [
        '\\' as u16, 'P' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'm' as u16,
        'e' as u16, 't' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0,
    ].as_ptr());

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szBuffer.as_ptr(),
        0,
        KEY_READ | KEY_WRITE,
        phkey,
    ) == (ERROR_SUCCESS) as i32
    {
        return S_OK;
    } else {
        return E_FAIL;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateNotificationObject(
    This: *mut INetCfgComponentImpl,
    iface: *mut INetCfgComponent,
    pUnk: *mut IUnknown,
) -> HRESULT {
    let mut szName: [u16; 150] = [0; 150];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwSize: u32 = 0;
    let mut dwType: u32 = 0;
    let mut CLSID_NotifyObject: GUID = core::mem::zeroed();
    let mut pStr: LPOLESTR = core::ptr::null_mut();
    let mut pNCCPU: *mut INetCfgComponentPropertyUi = core::ptr::null_mut();
    let mut pNCCC: *mut INetCfgComponentControl = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut lRet: i32;
    let mut ClassGUID: CLSID = core::mem::zeroed();
    let mut InstanceGUID: CLSID = core::mem::zeroed();

    wcscpy(szName.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Control\\Network\\") as LPCWSTR);

    hr = INetCfgComponent_GetClassGuid(iface, &mut ClassGUID);
    if hr < 0 {
        return hr;
    }

    hr = StringFromCLSID(&ClassGUID, &mut pStr);
    if hr < 0 {
        return hr;
    }

    wcscat(szName.as_mut_ptr(), pStr);
    CoTaskMemFree((pStr).cast());
    wcscat(szName.as_mut_ptr(), w!("\\") as LPCWSTR);

    hr = INetCfgComponent_GetInstanceGuid(iface, &mut InstanceGUID);
    if hr < 0 {
        return hr;
    }

    hr = StringFromCLSID(&InstanceGUID, &mut pStr);
    if hr < 0 {
        return hr;
    }

    wcscat(szName.as_mut_ptr(), pStr);
    CoTaskMemFree((pStr).cast());

    wcscat(szName.as_mut_ptr(), w!("\\NDI") as LPCWSTR);

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName.as_ptr(), 0, KEY_READ, &mut hKey) != (ERROR_SUCCESS) as i32 {
        return E_FAIL;
    }

    dwSize = core::mem::size_of_val(&szName) as u32;
    lRet = RegQueryValueExW(hKey, w!("ClsID") as LPCWSTR, core::ptr::null_mut(), &mut dwType, szName.as_mut_ptr() as LPBYTE, &mut dwSize);
    RegCloseKey(hKey);

    if lRet != (ERROR_SUCCESS) as i32 && dwType != (REG_SZ) as u32 {
        return E_FAIL;
    }

    hr = CLSIDFromString(szName.as_ptr(), &mut CLSID_NotifyObject);
    if hr < 0 {
        return E_FAIL;
    }

    hr = CoCreateInstance(&CLSID_NotifyObject, core::ptr::null_mut(), CLSCTX_INPROC_SERVER, &IID_INetCfgComponentPropertyUi, &mut pNCCPU as *mut *mut INetCfgComponentPropertyUi as *mut LPVOID);
    if hr < 0 {
        return E_FAIL;
    }

    hr = INetCfgComponentPropertyUi_QueryInterface(pNCCPU, &IID_INetCfgComponentControl, &mut pNCCC as *mut *mut INetCfgComponentControl as *mut LPVOID);
    if hr < 0 {
        INetCfgComponentPropertyUi_Release(pNCCPU);
        return hr;
    }

    hr = INetCfgComponentPropertyUi_QueryPropertyUi(pNCCPU, pUnk);
    if hr < 0 {
        INetCfgComponentPropertyUi_Release(pNCCPU);
        return hr;
    }

    hr = INetCfgComponentControl_Initialize(pNCCC, iface, (*This).pNCfg, (FALSE) as i32);
    if hr < 0 {
        INetCfgComponentControl_Release(pNCCC);
        INetCfgComponentPropertyUi_Release(pNCCPU);
        return hr;
    }

    hr = INetCfgComponentPropertyUi_SetContext(pNCCPU, pUnk);
    if hr < 0 {
        INetCfgComponentPropertyUi_Release(pNCCPU);
        return hr;
    }
    (*This).pProperty = pNCCPU;
    (*(*This).pItem).pNCCC = pNCCC;

    S_OK
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDI_INTERNET));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(netcfgx_hInstance, MAKEINTRESOURCEW!(IDI_INTERNET));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as SIZE_T, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_fnRaisePropertyUi(
    iface: *mut INetCfgComponent,
    hwndParent: HWND,
    dwFlags: u32,
    pUnk: *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut dwDefPages: u32;
    let mut Pages: u32;
    // C:682-683 leaves both uninitialised; pinfo is ZeroMemory'd and hppages is assigned
    // from the MergePropPages out-parameter before either is read.
    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hppages: *mut HPROPSHEETPAGE = core::ptr::null_mut();
    let iResult: INT_PTR;
    let This = iface as *mut INetCfgComponentImpl;

    if (*This).pProperty.is_null() {
        hr = CreateNotificationObject(This, iface, pUnk);
        if hr < 0 {
            return hr;
        }
    }

    if dwFlags == NCRP_QUERY_PROPERTY_UI {
        return S_OK;
    }

    dwDefPages = 0;
    Pages = 0;

    hr = INetCfgComponentPropertyUi_MergePropPages(
        (*This).pProperty,
        &mut dwDefPages,
        &mut hppages as *mut *mut HPROPSHEETPAGE as *mut LPBYTE,
        &mut Pages,
        hwndParent,
        core::ptr::null_mut(),
    );
    if hr < 0 || Pages == 0 {
        return hr;
    }
    ZeroMemory!(
        &mut pinfo as *mut PROPSHEETHEADERW as LPVOID,
        core::mem::size_of::<PROPSHEETHEADERW>(),
    );
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
    pinfo.dwFlags = (PSH_NOCONTEXTHELP
        | PSH_PROPTITLE
        | PSH_NOAPPLYNOW
        | PSH_USEICONID
        | PSH_USECALLBACK) as u32;
    pinfo.u3.phpage = hppages;
    pinfo.hwndParent = hwndParent;
    pinfo.nPages = Pages;
    pinfo.hInstance = netcfgx_hInstance;
    pinfo.pszCaption = (*(*This).pItem).szDisplayName;
    *pinfo.u.pszIcon.as_mut() = MAKEINTRESOURCEW!(IDI_INTERNET) as *const u16;
    pinfo.pfnCallback = Some(PropSheetProc);

    iResult = PropertySheetW(&mut pinfo);
    CoTaskMemFree(hppages as LPVOID);
    if iResult > 0 {
        (*(*This).pItem).bChanged = (TRUE) as i32;
        return S_OK;
    }
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn INetCfgComponent_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
    pItem: *mut NetCfgComponentItem,
    pNCfg: *mut INetCfg,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    let This = CoTaskMemAlloc((core::mem::size_of::<INetCfgComponentImpl>()) as SIZE_T) as *mut INetCfgComponentImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_NetCfgComponent as *const INetCfgComponentVtbl as *const INetCfgComponent;
    (*This).lpVtblBindings = &vt_NetCfgComponentBindings as *const INetCfgComponentBindingsVtbl as *const INetCfgComponentBindings;
    (*This).lpVtblPrivate = &vt_NetCfgComponentPrivate as *const INetCfgComponentPrivateVtbl as *const INetCfgComponentPrivate;
    (*This).pProperty = core::ptr::null_mut();
    (*This).pItem = pItem;
    (*This).pNCfg = pNCfg;

    if !(INetCfgComponent_QueryInterface(This as *mut INetCfgComponent, riid, ppv) >= 0) {
        INetCfgComponent_Release(This as *mut INetCfg);
        return E_NOINTERFACE;
    }

    INetCfgComponent_Release(This as *mut INetCfgComponent);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnQueryInterface(
    iface: *mut IEnumNetCfgComponent,
    iid: *const GUID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgComponentImpl;
    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(iid, &IID_IUnknown) ||
       IsEqualIID!(iid, &IID_IEnumNetCfgComponent)
    {
        *ppvObj = This as LPVOID;
        INetCfg_AddRef(iface as *mut INetCfg);
        return S_OK;
    }

    return E_NOINTERFACE;
}
#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnAddRef(
    iface: *mut IEnumNetCfgComponent
) -> ULONG {
    let This = iface as *mut IEnumNetCfgComponentImpl;
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_));

    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnRelease(
    iface: *mut IEnumNetCfgComponent
) -> u32 {
    let This = iface as *mut IEnumNetCfgComponentImpl;
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));

    refCount as u32
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnNext(
    iface: *mut IEnumNetCfgComponent,
    celt: u32,
    rgelt: *mut *mut INetCfgComponent,
    pceltFetched: *mut u32,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgComponentImpl;

    if iface.is_null() || rgelt.is_null() {
        return E_POINTER;
    }

    if celt != 1 {
        return E_INVALIDARG;
    }

    if (*This).pCurrent.is_null() {
        return S_FALSE;
    }

    let hr = INetCfgComponent_Constructor(
        core::ptr::null_mut(),
        &IID_INetCfgComponent,
        rgelt as *mut LPVOID,
        (*This).pCurrent,
        (*This).pNCfg,
    );

    if hr >= 0 {
        (*This).pCurrent = (*(*This).pCurrent).pNext;
        if !pceltFetched.is_null() {
            *pceltFetched = 1;
        }
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnSkip(
    iface: *mut IEnumNetCfgComponent,
    mut celt: ULONG,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgComponentImpl;

    if (*This).pCurrent.is_null() {
        return S_FALSE;
    }

    while celt > 0 && !(*This).pCurrent.is_null() {
        celt = celt.wrapping_sub(1);
        (*This).pCurrent = (*(*This).pCurrent).pNext;
    }

    if celt == 0 {
        return S_OK;
    } else {
        return S_FALSE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnReset(
    iface: *mut IEnumNetCfgComponent,
) -> HRESULT {
    let This = iface as *mut IEnumNetCfgComponentImpl;
    (*This).pCurrent = (*This).pHead;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_fnClone(
    iface: *mut IEnumNetCfgComponent,
    ppenum: *mut *mut IEnumNetCfgComponent,
) -> HRESULT {
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumNetCfgComponent_Constructor(
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppv: *mut LPVOID,
    pItem: *mut NetCfgComponentItem,
    pNCfg: *mut INetCfg,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    let This = CoTaskMemAlloc((core::mem::size_of::<IEnumNetCfgComponentImpl>()) as SIZE_T) as *mut IEnumNetCfgComponentImpl;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ref_ = 1;
    (*This).lpVtbl = &vt_EnumNetCfgComponent as *const IEnumNetCfgComponentVtbl as *const IEnumNetCfgComponent;
    (*This).pCurrent = pItem;
    (*This).pHead = pItem;
    (*This).pNCfg = pNCfg;

    if !((IEnumNetCfgComponent_QueryInterface(This as *mut INetCfgComponent, riid, ppv)) >= 0) {
        IEnumNetCfgComponent_Release(This as *mut INetCfg);
        return E_NOINTERFACE;
    }

    IEnumNetCfgComponent_Release(This as *mut IEnumNetCfgComponent);
    return S_OK;
}

// COM vtable tables — generated from inetcfgcomp_iface.c by vtbl_gen.py
/// inetcfgcomp_iface.c:172 `static const INetCfgComponentBindingsVtbl vt_NetCfgComponentBindings`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgComponentBindings: INetCfgComponentBindingsVtbl = INetCfgComponentBindingsVtbl {
    QueryInterface: Some(INetCfgComponentBindings_fnQueryInterface),
    AddRef: Some(INetCfgComponentBindings_fnAddRef),
    Release: Some(INetCfgComponentBindings_fnRelease),
    BindTo: Some(INetCfgComponentBindings_fnBindTo),
    UnbindFrom: Some(INetCfgComponentBindings_fnUnbindFrom),
    SupportsBindingInterface: Some(INetCfgComponentBindings_fnSupportsBindingInterface),
    IsBoundTo: Some(INetCfgComponentBindings_fnIsBoundTo),
    IsBindableTo: Some(INetCfgComponentBindings_fnIsBindableTo),
    EnumBindingPaths: Some(INetCfgComponentBindings_fnEnumBindingPaths),
    MoveBefore: Some(INetCfgComponentBindings_fnMoveBefore),
    MoveAfter: Some(INetCfgComponentBindings_fnMoveAfter),
};

/// inetcfgcomp_iface.c:232 `static const INetCfgComponentPrivateVtbl vt_NetCfgComponentPrivate`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgComponentPrivate: INetCfgComponentPrivateVtbl = INetCfgComponentPrivateVtbl {
    QueryInterface: Some(INetCfgComponentPrivate_fnQueryInterface),
    AddRef: Some(INetCfgComponentPrivate_fnAddRef),
    Release: Some(INetCfgComponentPrivate_fnRelease),
    // The bindgen island still carries the PRE-merge `(DWORD, DWORD)` shape for this
    // slot; the merged netcfgx_undoc.h says `(REFIID, void**)` and that is what the C
    // callers use. Transmute past the stale field type — same precedent as fltmgr's
    // CloseProcedure and ob's SecurityProcedure — until the island is regenerated.
    Unknown1: Some(unsafe {
        core::mem::transmute::<
            unsafe extern "system" fn(*mut INetCfgComponentPrivate, *const GUID, *mut LPVOID) -> HRESULT,
            unsafe extern "system" fn(*mut INetCfgComponentPrivate, u32, u32) -> HRESULT,
        >(INetCfgComponentPrivate_fnUnknown1)
    }),
};

/// inetcfgcomp_iface.c:727 `static const INetCfgComponentVtbl vt_NetCfgComponent`  (slot order: rust:types_amd64.rs)
pub static vt_NetCfgComponent: INetCfgComponentVtbl = INetCfgComponentVtbl {
    QueryInterface: Some(INetCfgComponent_fnQueryInterface),
    AddRef: Some(INetCfgComponent_fnAddRef),
    Release: Some(INetCfgComponent_fnRelease),
    GetDisplayName: Some(INetCfgComponent_fnGetDisplayName),
    SetDisplayName: Some(INetCfgComponent_fnSetDisplayName),
    GetHelpText: Some(INetCfgComponent_fnGetHelpText),
    GetId: Some(INetCfgComponent_fnGetId),
    GetCharacteristics: Some(INetCfgComponent_fnGetCharacteristics),
    GetInstanceGuid: Some(INetCfgComponent_fnGetInstanceGuid),
    GetPnpDevNodeId: Some(INetCfgComponent_fnGetPnpDevNodeId),
    GetClassGuid: Some(INetCfgComponent_fnGetClassGuid),
    GetBindName: Some(INetCfgComponent_fnGetBindName),
    GetDeviceStatus: Some(INetCfgComponent_fnGetDeviceStatus),
    OpenParamKey: Some(INetCfgComponent_fnOpenParamKey),
    RaisePropertyUi: Some(INetCfgComponent_fnRaisePropertyUi),
};

/// inetcfgcomp_iface.c:896 `static const IEnumNetCfgComponentVtbl vt_EnumNetCfgComponent`  (slot order: rust:types_amd64.rs)
pub static vt_EnumNetCfgComponent: IEnumNetCfgComponentVtbl = IEnumNetCfgComponentVtbl {
    QueryInterface: Some(IEnumNetCfgComponent_fnQueryInterface),
    AddRef: Some(IEnumNetCfgComponent_fnAddRef),
    Release: Some(IEnumNetCfgComponent_fnRelease),
    Next: Some(IEnumNetCfgComponent_fnNext),
    Skip: Some(IEnumNetCfgComponent_fnSkip),
    Reset: Some(IEnumNetCfgComponent_fnReset),
    Clone: Some(IEnumNetCfgComponent_fnClone),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// typedef struct
// {
//     const INetCfgComponent *lpVtbl;
//     const INetCfgComponentBindings *lpVtblBindings;
//     const INetCfgComponentPrivate  *lpVtblPrivate;
//     LONG  ref;
//     NetCfgComponentItem * pItem;
//     INetCfgComponentPropertyUi * pProperty;
//     INetCfg * pNCfg;
// } INetCfgComponentImpl;
//
// typedef struct
// {
//     const IEnumNetCfgComponent * lpVtbl;
//     LONG  ref;
//     NetCfgComponentItem * pCurrent;
//     NetCfgComponentItem * pHead;
//     INetCfg * pNCfg;
// } IEnumNetCfgComponentImpl;
//
// static __inline INetCfgComponentImpl* impl_from_INetCfgComponentBindings(INetCfgComponentBindings *iface)
// {
//     return (INetCfgComponentImpl*)((char *)iface - FIELD_OFFSET(INetCfgComponentImpl, lpVtblBindings));
// }
//
// static __inline INetCfgComponentImpl* impl_from_INetCfgComponentPrivate(INetCfgComponentPrivate *iface)
// {
//     return (INetCfgComponentImpl*)((char *)iface - FIELD_OFFSET(INetCfgComponentImpl, lpVtblPrivate));
// }
//
// /***************************************************************
//  * INetCfgComponentBindings
//  */
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnQueryInterface(
//     INetCfgComponentBindings *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentBindings(iface);
//     return INetCfgComponent_QueryInterface((INetCfgComponent*)This, iid, ppvObj);
// }
//
// ULONG
// WINAPI
// INetCfgComponentBindings_fnAddRef(
//     INetCfgComponentBindings *iface)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentBindings(iface);
//     return INetCfgComponent_AddRef((INetCfgComponent*)This);
// }
//
// ULONG
// WINAPI
// INetCfgComponentBindings_fnRelease(
//     INetCfgComponentBindings *iface)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentBindings(iface);
//     return INetCfgComponent_Release((INetCfgComponent*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnBindTo(
//     INetCfgComponentBindings *iface,
//     INetCfgComponent *pnccItem)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnUnbindFrom(
//     INetCfgComponentBindings *iface,
//     INetCfgComponent *pnccItem)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnSupportsBindingInterface(
//     INetCfgComponentBindings *iface,
//     DWORD dwFlags,
//     LPCWSTR pszwInterfaceName)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnIsBoundTo(
//     INetCfgComponentBindings *iface,
//     INetCfgComponent *pnccItem)
// {
//     INetCfgComponentImpl *pComponent;
//     PWSTR pszBindName, ptr;
//     INT len;
//
//     pComponent = impl_from_INetCfgComponentBindings(iface);
//     if (pComponent == NULL ||
//         pComponent->pItem == NULL ||
//         pComponent->pItem->pszBinding == NULL)
//         return E_POINTER;
//
//     if (pnccItem == NULL ||
//         ((INetCfgComponentImpl*)pnccItem)->pItem == NULL ||
//         ((INetCfgComponentImpl*)pnccItem)->pItem->szBindName == NULL)
//         return E_POINTER;
//
//     pszBindName = ((INetCfgComponentImpl*)pnccItem)->pItem->szBindName;
//
//     ptr = pComponent->pItem->pszBinding;
//     while (*ptr != UNICODE_NULL)
//     {
//         len = wcslen(ptr);
//
//         if (len > 8 && _wcsicmp(&ptr[8], pszBindName) == 0)
//             return S_OK;
//
//         ptr = ptr + len + 1;
//     }
//
//     return S_FALSE;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnIsBindableTo(
//     INetCfgComponentBindings *iface,
//     INetCfgComponent *pnccItem)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnEnumBindingPaths(
//     INetCfgComponentBindings *iface,
//     DWORD dwFlags,
//     IEnumNetCfgBindingPath **ppIEnum)
// {
//     return IEnumNetCfgBindingPath_Constructor(NULL, &IID_IEnumNetCfgBindingPath, (LPVOID *)ppIEnum, dwFlags);
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnMoveBefore(
//     INetCfgComponentBindings *iface,
//     DWORD dwFlags,
//     INetCfgBindingPath *pncbItemSrc,
//     INetCfgBindingPath *pncbItemDest)
// {
//     return E_NOTIMPL;
// }
//
// HRESULT
// WINAPI
// INetCfgComponentBindings_fnMoveAfter(
//     INetCfgComponentBindings *iface,
//     DWORD dwFlags,
//     INetCfgBindingPath *pncbItemSrc,
//     INetCfgBindingPath *pncbItemDest)
// {
//     return E_NOTIMPL;
// }
//
// static const INetCfgComponentBindingsVtbl vt_NetCfgComponentBindings =
// {
//     INetCfgComponentBindings_fnQueryInterface,
//     INetCfgComponentBindings_fnAddRef,
//     INetCfgComponentBindings_fnRelease,
//     INetCfgComponentBindings_fnBindTo,
//     INetCfgComponentBindings_fnUnbindFrom,
//     INetCfgComponentBindings_fnSupportsBindingInterface,
//     INetCfgComponentBindings_fnIsBoundTo,
//     INetCfgComponentBindings_fnIsBindableTo,
//     INetCfgComponentBindings_fnEnumBindingPaths,
//     INetCfgComponentBindings_fnMoveBefore,
//     INetCfgComponentBindings_fnMoveAfter,
// };
//
// /***************************************************************
//  * INetCfgComponentPrivate
//  */
//
// HRESULT
// WINAPI
// INetCfgComponentPrivate_fnQueryInterface(
//     INetCfgComponentPrivate *iface,
//     REFIID iid,
//     LPVOID *ppvObj)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentPrivate(iface);
//     return INetCfgComponent_QueryInterface((INetCfgComponent*)This, iid, ppvObj);
// }
//
// ULONG
// WINAPI
// INetCfgComponentPrivate_fnAddRef(
//     INetCfgComponentPrivate *iface)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentPrivate(iface);
//     return INetCfgComponent_AddRef((INetCfgComponent*)This);
// }
//
// ULONG
// WINAPI
// INetCfgComponentPrivate_fnRelease(
//     INetCfgComponentPrivate *iface)
// {
//     INetCfgComponentImpl *This = impl_from_INetCfgComponentPrivate(iface);
//     return INetCfgComponent_Release((INetCfgComponent*)This);
// }
//
// HRESULT
// WINAPI
// INetCfgComponentPrivate_fnUnknown1(
//     INetCfgComponentPrivate *iface,
//     DWORD dwParam1,
//     DWORD dwParam2)
// {
// //    INetCfgComponentImpl *This = impl_from_INetCfgComponentPrivate(iface);
//     ERR("INetCfgComponentPrivate_fnUnknown1(%p %lx %lx)\n", iface, dwParam1, dwParam2);
//     return S_OK;
// }
//
// static const INetCfgComponentPrivateVtbl vt_NetCfgComponentPrivate =
// {
//     INetCfgComponentPrivate_fnQueryInterface,
//     INetCfgComponentPrivate_fnAddRef,
//     INetCfgComponentPrivate_fnRelease,
//     INetCfgComponentPrivate_fnUnknown1,
// };
//
// /***************************************************************
//  * INetCfgComponent
//  */
//
// HRESULT
// WINAPI
// INetCfgComponent_fnQueryInterface(
//     INetCfgComponent * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_INetCfgComponent))
//     {
//         *ppvObj = This;
//         INetCfg_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID (iid, &IID_INetCfgComponentBindings))
//     {
//         *ppvObj = (LPVOID)&This->lpVtblBindings;
//         INetCfgComponentBindings_AddRef(iface);
//         return S_OK;
//     }
//     else if (IsEqualIID (iid, &IID_INetCfgComponentPrivate))
//     {
//         TRACE("IID_INetCfgComponentPrivate\n");
//         *ppvObj = (LPVOID)&This->lpVtblPrivate;
//         INetCfgComponentPrivate_AddRef(iface);
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
// ULONG
// WINAPI
// INetCfgComponent_fnAddRef(
//     INetCfgComponent * iface)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// INetCfgComponent_fnRelease(
//     INetCfgComponent * iface)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     if (!refCount)
//     {
//        CoTaskMemFree(This);
//     }
//     return refCount;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetDisplayName(
//     INetCfgComponent * iface,
//     LPWSTR * ppszwDisplayName)
// {
//     LPWSTR szName;
//     UINT Length;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwDisplayName == NULL)
//         return E_POINTER;
//
//     if (This->pItem->szDisplayName)
//         Length = wcslen(This->pItem->szDisplayName)+1;
//     else
//         Length = 1;
//
//     szName = CoTaskMemAlloc(Length * sizeof(WCHAR));
//     if (!szName)
//         return E_OUTOFMEMORY;
//
//     if (Length > 1)
//         wcscpy(szName, This->pItem->szDisplayName);
//     else
//         szName[0] = L'\0';
//
//     *ppszwDisplayName = szName;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnSetDisplayName(
//     INetCfgComponent * iface,
//     LPCWSTR ppszwDisplayName)
// {
//     LPWSTR szName;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwDisplayName == NULL)
//         return E_POINTER;
//
//     /* setting name is only supported for network cards */
//     if (!IsEqualGUID(&This->pItem->ClassGUID, &GUID_DEVCLASS_NET))
//         return E_NOTIMPL;
//
//     /// FIXME
//     /// check for invalid characters
//     /// check for write lock
//
//     szName = CoTaskMemAlloc((wcslen(ppszwDisplayName)+1) * sizeof(WCHAR));
//     if (!szName)
//         return E_OUTOFMEMORY;
//
//     wcscpy(szName, ppszwDisplayName);
//     CoTaskMemFree(This->pItem->szDisplayName);
//     This->pItem->szDisplayName = szName;
//     This->pItem->bChanged = TRUE;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetHelpText(
//     INetCfgComponent * iface,
//     LPWSTR * ppszwHelpText)
// {
//     LPWSTR szHelp;
//     UINT Length;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwHelpText == NULL)
//         return E_POINTER;
//
//     if (This->pItem->szHelpText)
//         Length = wcslen(This->pItem->szHelpText)+1;
//     else
//         Length = 1;
//
//     szHelp = CoTaskMemAlloc(Length * sizeof(WCHAR));
//     if (!szHelp)
//         return E_OUTOFMEMORY;
//
//     if (Length > 1)
//         wcscpy(szHelp, This->pItem->szHelpText);
//     else
//         szHelp[0] = L'\0';
//
//     *ppszwHelpText = szHelp;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetId(
//     INetCfgComponent * iface,
//     LPWSTR * ppszwId)
// {
//     LPWSTR szId;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwId == NULL)
//         return E_POINTER;
//
//     szId = CoTaskMemAlloc((wcslen(This->pItem->szId)+1) * sizeof(WCHAR));
//     if (!szId)
//         return E_OUTOFMEMORY;
//
//      wcscpy(szId, This->pItem->szId);
//     *ppszwId = szId;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetCharacteristics(
//     INetCfgComponent * iface,
//     DWORD * pdwCharacteristics)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || pdwCharacteristics == NULL)
//         return E_POINTER;
//
//     *pdwCharacteristics = This->pItem->dwCharacteristics;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetInstanceGuid(
//     INetCfgComponent * iface,
//     GUID * pGuid)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || pGuid == NULL)
//         return E_POINTER;
//
//     CopyMemory(pGuid, &This->pItem->InstanceId, sizeof(GUID));
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetPnpDevNodeId(
//     INetCfgComponent * iface,
//     LPWSTR * ppszwDevNodeId)
// {
//     LPWSTR szNode;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwDevNodeId == NULL)
//         return E_POINTER;
//
//     if (!IsEqualGUID(&GUID_DEVCLASS_NET, &This->pItem->ClassGUID))
//         return E_NOTIMPL;
//
//     szNode = CoTaskMemAlloc((wcslen(This->pItem->szNodeId)+1) * sizeof(WCHAR));
//     if (!szNode)
//         return E_OUTOFMEMORY;
//
//     wcscpy(szNode, This->pItem->szNodeId);
//     *ppszwDevNodeId = szNode;
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetClassGuid(
//     INetCfgComponent * iface,
//     GUID * pGuid)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || pGuid == NULL)
//         return E_POINTER;
//
//     CopyMemory(pGuid, &This->pItem->ClassGUID, sizeof(GUID));
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetBindName(
//     INetCfgComponent * iface,
//     LPWSTR * ppszwBindName)
// {
//     LPWSTR szBind;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || ppszwBindName == NULL)
//         return E_POINTER;
//
//     szBind = CoTaskMemAlloc((wcslen(This->pItem->szBindName)+1) * sizeof(WCHAR));
//     if (!szBind)
//         return E_OUTOFMEMORY;
//
//      wcscpy(szBind, This->pItem->szBindName);
//     *ppszwBindName = szBind;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnGetDeviceStatus(
//     INetCfgComponent * iface,
//     ULONG * pStatus)
// {
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || pStatus == NULL)
//         return E_POINTER;
//
//     if (!IsEqualGUID(&GUID_DEVCLASS_NET, &This->pItem->ClassGUID))
//         return E_UNEXPECTED;
//
//     *pStatus = This->pItem->Status;
//
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnOpenParamKey(
//     INetCfgComponent * iface,
//     HKEY * phkey)
// {
//     WCHAR szBuffer[200] = L"SYSTEM\\CurrentControlSet\\Services\\";
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (This == NULL || phkey == NULL)
//         return E_POINTER;
//
//     wcscat(szBuffer, This->pItem->szBindName);
//     wcscat(szBuffer, L"\\Parameters");
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szBuffer, 0, KEY_READ | KEY_WRITE, phkey) == ERROR_SUCCESS)
//         return S_OK;
//     else
//         return E_FAIL;
// }
//
//
// HRESULT
// CreateNotificationObject(
//     INetCfgComponentImpl * This,
//     INetCfgComponent * iface,
//     IUnknown  *pUnk)
// {
//     WCHAR szName[150];
//     HKEY hKey;
//     DWORD dwSize, dwType;
//     GUID CLSID_NotifyObject;
//     LPOLESTR pStr;
//     INetCfgComponentPropertyUi * pNCCPU;
//     INetCfgComponentControl * pNCCC;
//     HRESULT hr;
//     LONG lRet;
//     CLSID ClassGUID;
//     CLSID InstanceGUID;
//
//     wcscpy(szName,L"SYSTEM\\CurrentControlSet\\Control\\Network\\");
//
//     /* get the Class GUID */
//     hr = INetCfgComponent_GetClassGuid(iface, &ClassGUID);
//     if (FAILED(hr))
//         return hr;
//
//     hr = StringFromCLSID(&ClassGUID, &pStr);
//     if (FAILED(hr))
//         return hr;
//
//     wcscat(szName, pStr);
//     CoTaskMemFree(pStr);
//     wcscat(szName, L"\\");
//
//     /* get the Instance GUID */
//     hr = INetCfgComponent_GetInstanceGuid(iface, &InstanceGUID);
//     if (FAILED(hr))
//         return hr;
//
//     hr = StringFromCLSID(&InstanceGUID, &pStr);
//     if (FAILED(hr))
//         return hr;
//
//     wcscat(szName, pStr);
//     CoTaskMemFree(pStr);
//
//     wcscat(szName, L"\\NDI");
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, szName, 0, KEY_READ, &hKey) != ERROR_SUCCESS)
//         return E_FAIL;
//
//     dwSize = sizeof(szName);
//     lRet = RegQueryValueExW(hKey, L"ClsID", NULL, &dwType, (LPBYTE)szName, &dwSize);
//     RegCloseKey(hKey);
//
//     if (lRet != ERROR_SUCCESS && dwType != REG_SZ)
//         return E_FAIL;
//
//     hr = CLSIDFromString(szName, &CLSID_NotifyObject);
//     if (FAILED(hr))
//         return E_FAIL;
//
//     hr = CoCreateInstance(&CLSID_NotifyObject, NULL, CLSCTX_INPROC_SERVER, &IID_INetCfgComponentPropertyUi, (LPVOID*)&pNCCPU);
//     if (FAILED(hr))
//         return E_FAIL;
//
//     hr = INetCfgComponentPropertyUi_QueryInterface(pNCCPU, &IID_INetCfgComponentControl, (LPVOID*)&pNCCC);
//     if (FAILED(hr))
//     {
//         INetCfgComponentPropertyUi_Release(pNCCPU);
//         return hr;
//     }
//
//     hr = INetCfgComponentPropertyUi_QueryPropertyUi(pNCCPU, pUnk);
//     if (FAILED(hr))
//     {
//         INetCfgComponentPropertyUi_Release(pNCCPU);
//         return hr;
//     }
//
//     hr = INetCfgComponentControl_Initialize(pNCCC, iface, This->pNCfg, FALSE);
//     if (FAILED(hr))
//     {
//         INetCfgComponentControl_Release(pNCCC);
//         INetCfgComponentPropertyUi_Release(pNCCPU);
//         return hr;
//     }
//
//     hr = INetCfgComponentPropertyUi_SetContext(pNCCPU, pUnk);
//     if (FAILED(hr))
//     {
//         INetCfgComponentPropertyUi_Release(pNCCPU);
//         return hr;
//     }
//     This->pProperty = pNCCPU;
//     This->pItem->pNCCC = pNCCC;
//
//     return S_OK;
// }
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(netcfgx_hInstance, MAKEINTRESOURCEW(IDI_INTERNET));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// HRESULT
// WINAPI
// INetCfgComponent_fnRaisePropertyUi(
//     INetCfgComponent * iface,
//      HWND  hwndParent,
//      DWORD  dwFlags,
//      IUnknown  *pUnk)
// {
//     HRESULT hr;
//     DWORD dwDefPages;
//     UINT Pages;
//     PROPSHEETHEADERW pinfo;
//     HPROPSHEETPAGE * hppages;
//     INT_PTR iResult;
//     INetCfgComponentImpl * This = (INetCfgComponentImpl*)iface;
//
//     if (!This->pProperty)
//     {
//          hr = CreateNotificationObject(This,iface, pUnk);
//          if (FAILED(hr))
//              return hr;
//     }
//
//     if (dwFlags == NCRP_QUERY_PROPERTY_UI)
//         return S_OK;
//
//     dwDefPages = 0;
//     Pages = 0;
//
//     hr = INetCfgComponentPropertyUi_MergePropPages(This->pProperty, &dwDefPages, (BYTE**)&hppages, &Pages, hwndParent, NULL);
//     if (FAILED(hr) || !Pages)
//     {
//         return hr;
//     }
//     ZeroMemory(&pinfo, sizeof(PROPSHEETHEADERW));
//     pinfo.dwSize = sizeof(PROPSHEETHEADERW);
//     pinfo.dwFlags = PSH_NOCONTEXTHELP | PSH_PROPTITLE | PSH_NOAPPLYNOW |
//                     PSH_USEICONID | PSH_USECALLBACK;
//     pinfo.u3.phpage = hppages;
//     pinfo.hwndParent = hwndParent;
//     pinfo.nPages = Pages;
//     pinfo.hInstance = netcfgx_hInstance;
//     pinfo.pszCaption = This->pItem->szDisplayName;
//     pinfo.u.pszIcon = MAKEINTRESOURCEW(IDI_INTERNET);
//     pinfo.pfnCallback = PropSheetProc;
//
//     iResult = PropertySheetW(&pinfo);
//     CoTaskMemFree(hppages);
//     if (iResult > 0)
//     {
//         /* indicate that settings should be stored */
//         This->pItem->bChanged = TRUE;
//         return S_OK;
//     }
//     return S_FALSE;
// }
// static const INetCfgComponentVtbl vt_NetCfgComponent =
// {
//     INetCfgComponent_fnQueryInterface,
//     INetCfgComponent_fnAddRef,
//     INetCfgComponent_fnRelease,
//     INetCfgComponent_fnGetDisplayName,
//     INetCfgComponent_fnSetDisplayName,
//     INetCfgComponent_fnGetHelpText,
//     INetCfgComponent_fnGetId,
//     INetCfgComponent_fnGetCharacteristics,
//     INetCfgComponent_fnGetInstanceGuid,
//     INetCfgComponent_fnGetPnpDevNodeId,
//     INetCfgComponent_fnGetClassGuid,
//     INetCfgComponent_fnGetBindName,
//     INetCfgComponent_fnGetDeviceStatus,
//     INetCfgComponent_fnOpenParamKey,
//     INetCfgComponent_fnRaisePropertyUi
// };
//
// HRESULT
// WINAPI
// INetCfgComponent_Constructor (IUnknown * pUnkOuter, REFIID riid, LPVOID * ppv, NetCfgComponentItem * pItem, INetCfg * pNCfg)
// {
//     INetCfgComponentImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (INetCfgComponentImpl *) CoTaskMemAlloc(sizeof (INetCfgComponentImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const INetCfgComponent*)&vt_NetCfgComponent;
//     This->lpVtblBindings = (const INetCfgComponentBindings*)&vt_NetCfgComponentBindings;
//     This->lpVtblPrivate = (const INetCfgComponentPrivate*)&vt_NetCfgComponentPrivate;
//     This->pProperty = NULL;
//     This->pItem = pItem;
//     This->pNCfg = pNCfg;
//
//     if (!SUCCEEDED (INetCfgComponent_QueryInterface ((INetCfgComponent*)This, riid, ppv)))
//     {
//         INetCfgComponent_Release((INetCfg*)This);
//         return E_NOINTERFACE;
//     }
//
//     INetCfgComponent_Release((INetCfgComponent*)This);
//     return S_OK;
// }
//
//
// /***************************************************************
//  * IEnumNetCfgComponent
//  */
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_fnQueryInterface(
//     IEnumNetCfgComponent * iface,
//     REFIID iid,
//     LPVOID * ppvObj)
// {
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//     *ppvObj = NULL;
//
//     if (IsEqualIID (iid, &IID_IUnknown) ||
//         IsEqualIID (iid, &IID_IEnumNetCfgComponent))
//     {
//         *ppvObj = This;
//         INetCfg_AddRef(iface);
//         return S_OK;
//     }
//
//     return E_NOINTERFACE;
// }
//
//
// ULONG
// WINAPI
// IEnumNetCfgComponent_fnAddRef(
//     IEnumNetCfgComponent * iface)
// {
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//     ULONG refCount = InterlockedIncrement(&This->ref);
//
//     return refCount;
// }
//
// ULONG
// WINAPI
// IEnumNetCfgComponent_fnRelease(
//     IEnumNetCfgComponent * iface)
// {
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//     ULONG refCount = InterlockedDecrement(&This->ref);
//
//     return refCount;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_fnNext(
//     IEnumNetCfgComponent * iface,
//     ULONG celt,
//     INetCfgComponent **rgelt,
//     ULONG *pceltFetched)
// {
//     HRESULT hr;
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//
//     if (!iface || !rgelt)
//         return E_POINTER;
//
//     if (celt != 1)
//         return E_INVALIDARG;
//
//     if (!This->pCurrent)
//         return S_FALSE;
//
//     hr = INetCfgComponent_Constructor (NULL, &IID_INetCfgComponent, (LPVOID*)rgelt, This->pCurrent, This->pNCfg);
//     if (SUCCEEDED(hr))
//     {
//         This->pCurrent = This->pCurrent->pNext;
//         if (pceltFetched)
//             *pceltFetched = 1;
//     }
//     return hr;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_fnSkip(
//     IEnumNetCfgComponent * iface,
//     ULONG celt)
// {
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//
//     if (!This->pCurrent)
//         return S_FALSE;
//
//     while(celt-- > 0 && This->pCurrent)
//         This->pCurrent = This->pCurrent->pNext;
//
//     if (!celt)
//         return S_OK;
//     else
//         return S_FALSE;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_fnReset(
//     IEnumNetCfgComponent * iface)
// {
//     IEnumNetCfgComponentImpl * This = (IEnumNetCfgComponentImpl*)iface;
//
//     This->pCurrent = This->pHead;
//     return S_OK;
// }
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_fnClone(
//     IEnumNetCfgComponent * iface,
//     IEnumNetCfgComponent **ppenum)
// {
//     return E_NOTIMPL;
// }
//
// static const IEnumNetCfgComponentVtbl vt_EnumNetCfgComponent =
// {
//     IEnumNetCfgComponent_fnQueryInterface,
//     IEnumNetCfgComponent_fnAddRef,
//     IEnumNetCfgComponent_fnRelease,
//     IEnumNetCfgComponent_fnNext,
//     IEnumNetCfgComponent_fnSkip,
//     IEnumNetCfgComponent_fnReset,
//     IEnumNetCfgComponent_fnClone
// };
//
// HRESULT
// WINAPI
// IEnumNetCfgComponent_Constructor (IUnknown * pUnkOuter, REFIID riid, LPVOID * ppv, NetCfgComponentItem * pItem, INetCfg * pNCfg)
// {
//     IEnumNetCfgComponentImpl *This;
//
//     if (!ppv)
//         return E_POINTER;
//
//     This = (IEnumNetCfgComponentImpl *) CoTaskMemAlloc(sizeof (IEnumNetCfgComponentImpl));
//     if (!This)
//         return E_OUTOFMEMORY;
//
//     This->ref = 1;
//     This->lpVtbl = (const IEnumNetCfgComponent*)&vt_EnumNetCfgComponent;
//     This->pCurrent = pItem;
//     This->pHead = pItem;
//     This->pNCfg = pNCfg;
//
//     if (!SUCCEEDED (IEnumNetCfgComponent_QueryInterface ((INetCfgComponent*)This, riid, ppv)))
//     {
//         IEnumNetCfgComponent_Release((INetCfg*)This);
//         return E_NOINTERFACE;
//     }
//
//     IEnumNetCfgComponent_Release((IEnumNetCfgComponent*)This);
//     return S_OK;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
