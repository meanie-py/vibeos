use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn FaxAbort(FaxHandle: HANDLE, JobId: DWORD) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxAccessCheck(FaxHandle: HANDLE, AccessMask: DWORD) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxClose(FaxHandle: HANDLE) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxCompleteJobParamsA(
    JobParams: *mut PFAX_JOB_PARAMA,
    CoverageInfo: *mut PFAX_COVERAGE_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxCompleteJobParamsW(
    JobParams: *mut PFAX_JOB_PARAMW,
    CoverageInfo: *mut PFAX_COVERAGE_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxConnectFaxServerA(
    MachineName: LPCSTR,
    FaxHandle: LPHANDLE,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxConnectFaxServerW(
    MachineName: LPCWSTR,
    FaxHandle: LPHANDLE,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnableRoutingMethodA(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCSTR,
    Enabled: BOOL,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnableRoutingMethodW(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCWSTR,
    Enabled: BOOL,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumGlobalRoutingInfoA(
    FaxHandle: HANDLE,
    RoutingInfo: *mut PFAX_GLOBAL_ROUTING_INFOA,
    MethodsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumGlobalRoutingInfoW(
    FaxHandle: HANDLE,
    RoutingInfo: *mut PFAX_GLOBAL_ROUTING_INFOW,
    MethodsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumJobsA(
    FaxHandle: HANDLE,
    JobEntry: *mut PFAX_JOB_ENTRYA,
    JobsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumJobsW(
    FaxHandle: HANDLE,
    JobEntry: *mut PFAX_JOB_ENTRYW,
    JobsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumPortsA(
    FaxHandle: HANDLE,
    PortInfo: *mut PFAX_PORT_INFOA,
    PortsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumPortsW(
    FaxHandle: HANDLE,
    PortInfo: *mut PFAX_PORT_INFOW,
    PortsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumRoutingMethodsA(
    FaxPortHandle: HANDLE,
    RoutingMethod: *mut PFAX_ROUTING_METHODA,
    MethodsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxEnumRoutingMethodsW(
    FaxPortHandle: HANDLE,
    RoutingMethod: *mut PFAX_ROUTING_METHODW,
    MethodsReturned: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxFreeBuffer(Buffer: LPVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetConfigurationA(
    FaxHandle: HANDLE,
    FaxConfig: *mut PFAX_CONFIGURATIONA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetConfigurationW(
    FaxHandle: HANDLE,
    FaxConfig: *mut PFAX_CONFIGURATIONW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetDeviceStatusA(
    FaxPortHandle: HANDLE,
    DeviceStatus: *mut PFAX_DEVICE_STATUSA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetDeviceStatusW(
    FaxPortHandle: HANDLE,
    DeviceStatus: *mut PFAX_DEVICE_STATUSW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetJobA(
    FaxHandle: HANDLE,
    JobId: DWORD,
    JobEntry: *mut PFAX_JOB_ENTRYA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetJobW(
    FaxHandle: HANDLE,
    JobId: DWORD,
    JobEntry: *mut PFAX_JOB_ENTRYW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetLoggingCategoriesA(
    FaxHandle: HANDLE,
    Categories: *mut PFAX_LOG_CATEGORYA,
    NumberCategories: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetLoggingCategoriesW(
    FaxHandle: HANDLE,
    Categories: *mut PFAX_LOG_CATEGORYW,
    NumberCategories: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetPageData(
    FaxHandle: HANDLE,
    JobId: DWORD,
    Buffer: *mut LPBYTE,
    BufferSize: LPDWORD,
    ImageWidth: LPDWORD,
    ImageHeight: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetPortA(
    FaxPortHandle: HANDLE,
    PortInfo: *mut PFAX_PORT_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetPortW(
    FaxPortHandle: HANDLE,
    PortInfo: *mut PFAX_PORT_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetRoutingInfoA(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCSTR,
    RoutingInfoBuffer: *mut LPBYTE,
    RoutingInfoBufferSize: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxGetRoutingInfoW(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCWSTR,
    RoutingInfoBuffer: *mut LPBYTE,
    RoutingInfoBufferSize: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxInitializeEventQueue(
    FaxHandle: HANDLE,
    CompletionPort: HANDLE,
    CompletionKey: ULONG_PTR,
    hWnd: HWND,
    MessageStart: UINT,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxOpenPort(
    FaxHandle: HANDLE,
    DeviceId: DWORD,
    Flags: DWORD,
    FaxPortHandle: LPHANDLE,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxPrintCoverPageA(
    FaxContextInfo: *const FAX_CONTEXT_INFOA,
    CoverPageInfo: *const FAX_COVERAGE_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxPrintCoverPageW(
    FaxContextInfo: *const FAX_CONTEXT_INFOW,
    CoverPageInfo: *const FAX_COVERAGE_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxRegisterRoutingExtensionW(
    FaxHandle: HANDLE,
    ExtensionName: LPCWSTR,
    FriendlyName: LPCWSTR,
    ImageName: LPCWSTR,
    CallBack: PFAX_ROUTING_INSTALLATION_CALLBACKW,
    Context: LPVOID,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxRegisterServiceProviderW(
    DeviceProvider: LPCWSTR,
    FriendlyName: LPCWSTR,
    ImageName: LPCWSTR,
    TspName: LPCWSTR,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSendDocumentA(
    FaxHandle: HANDLE,
    FileName: LPCSTR,
    JobParams: PFAX_JOB_PARAMA,
    CoverpageInfo: *const FAX_COVERAGE_INFOA,
    FaxJobId: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSendDocumentForBroadcastA(
    FaxHandle: HANDLE,
    FileName: LPCSTR,
    FaxJobId: LPDWORD,
    FaxRecipientCallback: PFAX_RECIPIENT_CALLBACKA,
    Context: LPVOID,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSendDocumentForBroadcastW(
    FaxHandle: HANDLE,
    FileName: LPCWSTR,
    FaxJobId: LPDWORD,
    FaxRecipientCallback: PFAX_RECIPIENT_CALLBACKW,
    Context: LPVOID,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSendDocumentW(
    FaxHandle: HANDLE,
    FileName: LPCWSTR,
    JobParams: PFAX_JOB_PARAMW,
    CoverpageInfo: *const FAX_COVERAGE_INFOW,
    FaxJobId: LPDWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetConfigurationA(
    FaxHandle: HANDLE,
    FaxConfig: *const FAX_CONFIGURATIONA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetConfigurationW(
    FaxHandle: HANDLE,
    FaxConfig: *const FAX_CONFIGURATIONW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetGlobalRoutingInfoA(
    FaxHandle: HANDLE,
    RoutingInfo: *const FAX_GLOBAL_ROUTING_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetGlobalRoutingInfoW(
    FaxHandle: HANDLE,
    RoutingInfo: *const FAX_GLOBAL_ROUTING_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetJobA(
    FaxHandle: HANDLE,
    JobId: DWORD,
    Command: DWORD,
    JobEntry: *const FAX_JOB_ENTRYA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetJobW(
    FaxHandle: HANDLE,
    JobId: DWORD,
    Command: DWORD,
    JobEntry: *const FAX_JOB_ENTRYW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetLoggingCategoriesA(
    FaxHandle: HANDLE,
    Categories: *const FAX_LOG_CATEGORYA,
    NumberCategories: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetLoggingCategoriesW(
    FaxHandle: HANDLE,
    Categories: *const FAX_LOG_CATEGORYW,
    NumberCategories: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetPortA(
    FaxPortHandle: HANDLE,
    PortInfo: *const FAX_PORT_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetPortW(
    FaxPortHandle: HANDLE,
    PortInfo: *const FAX_PORT_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetRoutingInfoA(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCSTR,
    RoutingInfoBuffer: *const BYTE,
    RoutingInfoBufferSize: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxSetRoutingInfoW(
    FaxPortHandle: HANDLE,
    RoutingGuid: LPCWSTR,
    RoutingInfoBuffer: *const BYTE,
    RoutingInfoBufferSize: DWORD,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxStartPrintJobA(
    PrinterName: LPCSTR,
    PrintInfo: *const FAX_PRINT_INFOA,
    FaxJobId: LPDWORD,
    FaxContextInfo: PFAX_CONTEXT_INFOA,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn FaxStartPrintJobW(
    PrinterName: LPCWSTR,
    PrintInfo: *const FAX_PRINT_INFOW,
    FaxJobId: LPDWORD,
    FaxContextInfo: PFAX_CONTEXT_INFOW,
) -> BOOL {
    // UNIMPLEMENTED;
    (FALSE as i32)
}
