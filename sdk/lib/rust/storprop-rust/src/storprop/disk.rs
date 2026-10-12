use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "system" fn DiskPropPageDialog(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    DPRINT1!("DiskPropPageDialog()\n");

    match uMsg {
        WM_INITDIALOG => {
            return TRUE as INT_PTR; //OnInitDialog(hwnd, wParam, lParam);
        }
        WM_COMMAND => {
            EndDialog(hwnd, 0 as INT_PTR); //OnCommand(hwnd, wParam, lParam);
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn DiskClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut AddPropertyPageData: SP_ADDPROPERTYPAGE_DATA = core::mem::zeroed();
    let mut Page: PROPSHEETPAGE = core::mem::zeroed();
    let mut PageHandle: HPROPSHEETPAGE;

    DPRINT1!("DiskClassInstaller(%u %p %p)\n", InstallFunction, DeviceInfoSet, DeviceInfoData);

    if InstallFunction == DIF_ADDPROPERTYPAGE_ADVANCED {
        if DeviceInfoData.is_null() {
            return ERROR_DI_DO_DEFAULT;
        }

        AddPropertyPageData.ClassInstallHeader.cbSize = core::mem::size_of::<SP_CLASSINSTALL_HEADER>() as DWORD;
        if SetupDiGetClassInstallParamsW(
            DeviceInfoSet,
            DeviceInfoData,
            &raw mut AddPropertyPageData as PSP_CLASSINSTALL_HEADER,
            core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as DWORD,
            core::ptr::null_mut::<DWORD>(),
        ) != 0
        {
            DPRINT1!("\n");
            if AddPropertyPageData.NumDynamicPages >= MAX_INSTALLWIZARD_DYNAPAGES as DWORD {
                return ERROR_SUCCESS;
            }

            ZeroMemory!(
                &raw mut Page as *mut core::ffi::c_void,
                core::mem::size_of::<PROPSHEETPAGE>() as usize,
            );
            Page.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
            Page.dwFlags = PSP_USECALLBACK as DWORD;
            Page.hInstance = hInstance;
            Page.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_DISK_POLICIES);
            Page.pfnDlgProc = Some(DiskPropPageDialog);
            Page.pfnCallback = None;
            Page.lParam = 0 as LPARAM;

            PageHandle = CreatePropertySheetPage(&raw mut Page);
            if PageHandle.is_null() {
                DPRINT1!("CreatePropertySheetPage() failed!\n");
                return ERROR_SUCCESS;
            }

            AddPropertyPageData.DynamicPages[AddPropertyPageData.NumDynamicPages as usize] =
                PageHandle;
            AddPropertyPageData.NumDynamicPages += 1;
            DPRINT1!("Pages: %ld\n", AddPropertyPageData.NumDynamicPages);

            if SetupDiSetClassInstallParamsW(
                DeviceInfoSet,
                DeviceInfoData,
                &raw mut AddPropertyPageData as PSP_CLASSINSTALL_HEADER,
                core::mem::size_of::<SP_ADDPROPERTYPAGE_DATA>() as DWORD,
            ) == 0
            {
                DPRINT1!(
                    "SetupDiSetClassInstallParamsW() failed (Error %lu)\n",
                    GetLastError()
                );
            }
        }

        return ERROR_SUCCESS;
    }

    return ERROR_DI_DO_DEFAULT;
}
