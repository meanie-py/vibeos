// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellDispatch4 {
    pub lpVtbl: *const IShellDispatch4Vtbl,
}
#[repr(C)]
pub struct IShellDispatch4Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDispatch4, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDispatch4) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDispatch4) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellDispatch4, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellDispatch4, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellDispatch4, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellDispatch4, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT,
    pub NameSpace: unsafe extern "system" fn(This: *mut IShellDispatch4, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub BrowseForFolder: unsafe extern "system" fn(This: *mut IShellDispatch4, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub Windows: unsafe extern "system" fn(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Open: unsafe extern "system" fn(This: *mut IShellDispatch4, vDir: VARIANT) -> HRESULT,
    pub Explore: unsafe extern "system" fn(This: *mut IShellDispatch4, vDir: VARIANT) -> HRESULT,
    pub MinimizeAll: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub UndoMinimizeALL: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub FileRun: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub CascadeWindows: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub TileVertically: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub TileHorizontally: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub ShutdownWindows: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub Suspend: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub EjectPC: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub SetTime: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub TrayProperties: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub Help: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub FindFiles: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub FindComputer: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub RefreshMenu: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub ControlPanelItem: unsafe extern "system" fn(This: *mut IShellDispatch4, szDir: BSTR) -> HRESULT,
    pub IsRestricted: unsafe extern "system" fn(This: *mut IShellDispatch4, group: BSTR, restriction: BSTR, value: *mut LONG) -> HRESULT,
    pub ShellExecute: unsafe extern "system" fn(This: *mut IShellDispatch4, file: BSTR, args: VARIANT, dir: VARIANT, op: VARIANT, show: VARIANT) -> HRESULT,
    pub FindPrinter: unsafe extern "system" fn(This: *mut IShellDispatch4, name: BSTR, location: BSTR, model: BSTR) -> HRESULT,
    pub GetSystemInformation: unsafe extern "system" fn(This: *mut IShellDispatch4, name: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStart: unsafe extern "system" fn(This: *mut IShellDispatch4, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStop: unsafe extern "system" fn(This: *mut IShellDispatch4, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub IsServiceRunning: unsafe extern "system" fn(This: *mut IShellDispatch4, service: BSTR, running: *mut VARIANT) -> HRESULT,
    pub CanStartStopService: unsafe extern "system" fn(This: *mut IShellDispatch4, service: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ShowBrowserBar: unsafe extern "system" fn(This: *mut IShellDispatch4, clsid: BSTR, show: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub AddToRecent: unsafe extern "system" fn(This: *mut IShellDispatch4, file: VARIANT, category: BSTR) -> HRESULT,
    pub WindowsSecurity: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub ToggleDesktop: unsafe extern "system" fn(This: *mut IShellDispatch4) -> HRESULT,
    pub ExplorerPolicy: unsafe extern "system" fn(This: *mut IShellDispatch4, policy: BSTR, value: *mut VARIANT) -> HRESULT,
    pub GetSetting: unsafe extern "system" fn(This: *mut IShellDispatch4, setting: LONG, result: *mut VARIANT_BOOL) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDispatch4: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellDispatch3 {
    pub lpVtbl: *const IShellDispatch3Vtbl,
}
#[repr(C)]
pub struct IShellDispatch3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDispatch3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDispatch3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDispatch3) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellDispatch3, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellDispatch3, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellDispatch3, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellDispatch3, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellDispatch3, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellDispatch3, ppid: *mut *mut IDispatch) -> HRESULT,
    pub NameSpace: unsafe extern "system" fn(This: *mut IShellDispatch3, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub BrowseForFolder: unsafe extern "system" fn(This: *mut IShellDispatch3, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub Windows: unsafe extern "system" fn(This: *mut IShellDispatch3, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Open: unsafe extern "system" fn(This: *mut IShellDispatch3, vDir: VARIANT) -> HRESULT,
    pub Explore: unsafe extern "system" fn(This: *mut IShellDispatch3, vDir: VARIANT) -> HRESULT,
    pub MinimizeAll: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub UndoMinimizeALL: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub FileRun: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub CascadeWindows: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub TileVertically: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub TileHorizontally: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub ShutdownWindows: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub Suspend: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub EjectPC: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub SetTime: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub TrayProperties: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub Help: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub FindFiles: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub FindComputer: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub RefreshMenu: unsafe extern "system" fn(This: *mut IShellDispatch3) -> HRESULT,
    pub ControlPanelItem: unsafe extern "system" fn(This: *mut IShellDispatch3, szDir: BSTR) -> HRESULT,
    pub IsRestricted: unsafe extern "system" fn(This: *mut IShellDispatch3, group: BSTR, restriction: BSTR, value: *mut LONG) -> HRESULT,
    pub ShellExecute: unsafe extern "system" fn(This: *mut IShellDispatch3, file: BSTR, args: VARIANT, dir: VARIANT, op: VARIANT, show: VARIANT) -> HRESULT,
    pub FindPrinter: unsafe extern "system" fn(This: *mut IShellDispatch3, name: BSTR, location: BSTR, model: BSTR) -> HRESULT,
    pub GetSystemInformation: unsafe extern "system" fn(This: *mut IShellDispatch3, name: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStart: unsafe extern "system" fn(This: *mut IShellDispatch3, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStop: unsafe extern "system" fn(This: *mut IShellDispatch3, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub IsServiceRunning: unsafe extern "system" fn(This: *mut IShellDispatch3, service: BSTR, running: *mut VARIANT) -> HRESULT,
    pub CanStartStopService: unsafe extern "system" fn(This: *mut IShellDispatch3, service: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ShowBrowserBar: unsafe extern "system" fn(This: *mut IShellDispatch3, clsid: BSTR, show: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub AddToRecent: unsafe extern "system" fn(This: *mut IShellDispatch3, file: VARIANT, category: BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDispatch3: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellDispatch2 {
    pub lpVtbl: *const IShellDispatch2Vtbl,
}
#[repr(C)]
pub struct IShellDispatch2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDispatch2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDispatch2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDispatch2) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellDispatch2, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellDispatch2, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellDispatch2, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellDispatch2, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellDispatch2, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellDispatch2, ppid: *mut *mut IDispatch) -> HRESULT,
    pub NameSpace: unsafe extern "system" fn(This: *mut IShellDispatch2, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub BrowseForFolder: unsafe extern "system" fn(This: *mut IShellDispatch2, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub Windows: unsafe extern "system" fn(This: *mut IShellDispatch2, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Open: unsafe extern "system" fn(This: *mut IShellDispatch2, vDir: VARIANT) -> HRESULT,
    pub Explore: unsafe extern "system" fn(This: *mut IShellDispatch2, vDir: VARIANT) -> HRESULT,
    pub MinimizeAll: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub UndoMinimizeALL: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub FileRun: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub CascadeWindows: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub TileVertically: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub TileHorizontally: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub ShutdownWindows: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub Suspend: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub EjectPC: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub SetTime: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub TrayProperties: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub Help: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub FindFiles: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub FindComputer: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub RefreshMenu: unsafe extern "system" fn(This: *mut IShellDispatch2) -> HRESULT,
    pub ControlPanelItem: unsafe extern "system" fn(This: *mut IShellDispatch2, szDir: BSTR) -> HRESULT,
    pub IsRestricted: unsafe extern "system" fn(This: *mut IShellDispatch2, group: BSTR, restriction: BSTR, value: *mut LONG) -> HRESULT,
    pub ShellExecute: unsafe extern "system" fn(This: *mut IShellDispatch2, file: BSTR, args: VARIANT, dir: VARIANT, op: VARIANT, show: VARIANT) -> HRESULT,
    pub FindPrinter: unsafe extern "system" fn(This: *mut IShellDispatch2, name: BSTR, location: BSTR, model: BSTR) -> HRESULT,
    pub GetSystemInformation: unsafe extern "system" fn(This: *mut IShellDispatch2, name: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStart: unsafe extern "system" fn(This: *mut IShellDispatch2, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub ServiceStop: unsafe extern "system" fn(This: *mut IShellDispatch2, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT,
    pub IsServiceRunning: unsafe extern "system" fn(This: *mut IShellDispatch2, service: BSTR, running: *mut VARIANT) -> HRESULT,
    pub CanStartStopService: unsafe extern "system" fn(This: *mut IShellDispatch2, service: BSTR, ret: *mut VARIANT) -> HRESULT,
    pub ShowBrowserBar: unsafe extern "system" fn(This: *mut IShellDispatch2, clsid: BSTR, show: VARIANT, ret: *mut VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDispatch2: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellDispatch {
    pub lpVtbl: *const IShellDispatchVtbl,
}
#[repr(C)]
pub struct IShellDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub NameSpace: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub BrowseForFolder: unsafe extern "system" fn(This: *mut IShellDispatch, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT,
    pub Windows: unsafe extern "system" fn(This: *mut IShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT,
    pub Open: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT) -> HRESULT,
    pub Explore: unsafe extern "system" fn(This: *mut IShellDispatch, vDir: VARIANT) -> HRESULT,
    pub MinimizeAll: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub UndoMinimizeALL: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FileRun: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub CascadeWindows: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TileVertically: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TileHorizontally: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub ShutdownWindows: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub Suspend: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub EjectPC: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub SetTime: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub TrayProperties: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub Help: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FindFiles: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub FindComputer: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub RefreshMenu: unsafe extern "system" fn(This: *mut IShellDispatch) -> HRESULT,
    pub ControlPanelItem: unsafe extern "system" fn(This: *mut IShellDispatch, szDir: BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellDispatch: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IDispatch {
    pub lpVtbl: *const IDispatchVtbl,
}
#[repr(C)]
pub struct IDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IDispatch: IID;
}

// Deterministic from objsafe.h
#[repr(C)]
pub struct IObjectSafety {
    pub lpVtbl: *const IObjectSafetyVtbl,
}
#[repr(C)]
pub struct IObjectSafetyVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IObjectSafety, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectSafety) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectSafety) -> ULONG,
    pub GetInterfaceSafetyOptions: unsafe extern "system" fn(This: *mut IObjectSafety, riid: REFIID, pdwSupportedOptions: *mut DWORD, pdwEnabledOptions: *mut DWORD) -> HRESULT,
    pub SetInterfaceSafetyOptions: unsafe extern "system" fn(This: *mut IObjectSafety, riid: REFIID, dwOptionSetMask: DWORD, dwEnabledOptions: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IObjectSafety: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}
#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT,
}
extern "C" {
    pub static IID_IObjectWithSite: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeInfo {
    pub lpVtbl: *const ITypeInfoVtbl,
}
#[repr(C)]
pub struct ITypeInfoVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeInfo, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, ppTypeAttr: *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppFuncDesc: *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppVarDesc: *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, rgBstrNames: *mut BSTR, cMaxNames: UINT, pcNames: *mut UINT) -> HRESULT,
    pub GetRefTypeOfImplType: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pRefType: *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pImplTypeFlags: *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut ITypeInfo, rgszNames: *mut LPOLESTR, cNames: UINT, pMemId: *mut MEMBERID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut ITypeInfo, pvInstance: PVOID, memid: MEMBERID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, pBstrDllName: *mut BSTR, pBstrName: *mut BSTR, pwOrdinal: *mut WORD) -> HRESULT,
    pub GetRefTypeInfo: unsafe extern "system" fn(This: *mut ITypeInfo, hRefType: HREFTYPE, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, ppv: *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(This: *mut ITypeInfo, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObj: *mut PVOID) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrMops: *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib: unsafe extern "system" fn(This: *mut ITypeInfo, ppTLib: *mut *mut ITypeLib, pIndex: *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, pTypeAttr: *mut TYPEATTR) -> core::ffi::c_void,
    pub ReleaseFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pFuncDesc: *mut FUNCDESC) -> core::ffi::c_void,
    pub ReleaseVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pVarDesc: *mut VARDESC) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeInfo: IID;
}

// Deterministic from unknwn.h
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUnknown, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
}
extern "C" {
    pub static IID_IUnknown: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeComp {
    pub lpVtbl: *const ITypeCompVtbl,
}
#[repr(C)]
pub struct ITypeCompVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeComp, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Bind: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, wFlags: WORD, ppTInfo: *mut *mut ITypeInfo, pDescKind: *mut DESCKIND, pBindPtr: *mut BINDPTR) -> HRESULT,
    pub BindType: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
}
extern "C" {
    pub static IID_ITypeComp: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeLib {
    pub lpVtbl: *const ITypeLibVtbl,
}
#[repr(C)]
pub struct ITypeLibVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeLib, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut ITypeLib) -> UINT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, pTKind: *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid: unsafe extern "system" fn(This: *mut ITypeLib, guid: REFGUID, ppTinfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, ppTLibAttr: *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeLib, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeLib, index: INT, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub IsName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, pfName: *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, rgMemId: *mut MEMBERID, pcFound: *mut USHORT) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, pTLibAttr: *mut TLIBATTR) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeLib: IID;
}

// Deterministic COM scaffold for CShellDispatch; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CShellDispatch {
    pub IShellDispatch4_iface: IShellDispatch4,
    pub IObjectSafety_iface: IObjectSafety,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pTypeInfo: *mut ITypeInfo,
}

unsafe fn CShellDispatch_cpp_com_assign_m_pTypeInfo(this: *mut CShellDispatch, value: *mut ITypeInfo) {
    let old = (*this).m_pTypeInfo;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pTypeInfo = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CShellDispatch_IShellDispatch4_QueryInterface(This: *mut IShellDispatch4, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_AddRef(This: *mut IShellDispatch4) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_AddRef(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Release(This: *mut IShellDispatch4) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Release(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_GetTypeInfoCount(This: *mut IShellDispatch4, pctinfo: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_GetTypeInfoCount_v1_body(this, pctinfo)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_GetTypeInfo(This: *mut IShellDispatch4, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_GetTypeInfo_v1_body(this, iTInfo, lcid, ppTInfo)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_GetIDsOfNames(This: *mut IShellDispatch4, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_GetIDsOfNames_v1_body(this, riid, rgszNames, cNames, lcid, rgDispId)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Invoke(This: *mut IShellDispatch4, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Invoke_v1_body(this, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_get_Application(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_get_Application_body(this, ppid)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_get_Parent(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_get_Parent_body(this, ppid)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_NameSpace(This: *mut IShellDispatch4, vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_NameSpace_body(this, vDir, ppsdf)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_BrowseForFolder(This: *mut IShellDispatch4, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_BrowseForFolder_body(this, Hwnd, Title, Options, RootFolder, ppsdf)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Windows(This: *mut IShellDispatch4, ppid: *mut *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Windows_body(this, ppid)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Open(This: *mut IShellDispatch4, vDir: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Open_body(this, vDir)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Explore(This: *mut IShellDispatch4, vDir: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Explore_body(this, vDir)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_MinimizeAll(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_MinimizeAll_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_UndoMinimizeALL(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_UndoMinimizeALL_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_FileRun(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_FileRun_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_CascadeWindows(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_CascadeWindows_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_TileVertically(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_TileVertically_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_TileHorizontally(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_TileHorizontally_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ShutdownWindows(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ShutdownWindows_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Suspend(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Suspend_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_EjectPC(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_EjectPC_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_SetTime(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_SetTime_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_TrayProperties(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_TrayProperties_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_Help(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_Help_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_FindFiles(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_FindFiles_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_FindComputer(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_FindComputer_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_RefreshMenu(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_RefreshMenu_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ControlPanelItem(This: *mut IShellDispatch4, szDir: BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ControlPanelItem_body(this, szDir)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_IsRestricted(This: *mut IShellDispatch4, group: BSTR, restriction: BSTR, value: *mut LONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_IsRestricted_body(this, group, restriction, value)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ShellExecute(This: *mut IShellDispatch4, file: BSTR, args: VARIANT, dir: VARIANT, op: VARIANT, show: VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ShellExecute_body(this, file, args, dir, op, show)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_FindPrinter(This: *mut IShellDispatch4, name: BSTR, location: BSTR, model: BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_FindPrinter_body(this, name, location, model)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_GetSystemInformation(This: *mut IShellDispatch4, name: BSTR, ret: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_GetSystemInformation_body(this, name, ret)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ServiceStart(This: *mut IShellDispatch4, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ServiceStart_body(this, service, persistent, ret)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ServiceStop(This: *mut IShellDispatch4, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ServiceStop_body(this, service, persistent, ret)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_IsServiceRunning(This: *mut IShellDispatch4, service: BSTR, running: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_IsServiceRunning_body(this, service, running)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_CanStartStopService(This: *mut IShellDispatch4, service: BSTR, ret: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_CanStartStopService_body(this, service, ret)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ShowBrowserBar(This: *mut IShellDispatch4, clsid: BSTR, show: VARIANT, ret: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ShowBrowserBar_body(this, clsid, show, ret)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_AddToRecent(This: *mut IShellDispatch4, file: VARIANT, category: BSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_AddToRecent_body(this, file, category)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_WindowsSecurity(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_WindowsSecurity_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ToggleDesktop(This: *mut IShellDispatch4) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ToggleDesktop_body(this)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_ExplorerPolicy(This: *mut IShellDispatch4, policy: BSTR, value: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_ExplorerPolicy_body(this, policy, value)
}
unsafe extern "system" fn CShellDispatch_IShellDispatch4_GetSetting(This: *mut IShellDispatch4, setting: LONG, result: *mut VARIANT_BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IShellDispatch4_iface)) as *mut CShellDispatch;
    CShellDispatch_GetSetting_body(this, setting, result)
}

pub static CSHELLDISPATCH_ISHELLDISPATCH4_VTBL: IShellDispatch4Vtbl = IShellDispatch4Vtbl {
    QueryInterface: CShellDispatch_IShellDispatch4_QueryInterface,
    AddRef: CShellDispatch_IShellDispatch4_AddRef,
    Release: CShellDispatch_IShellDispatch4_Release,
    GetTypeInfoCount: CShellDispatch_IShellDispatch4_GetTypeInfoCount,
    GetTypeInfo: CShellDispatch_IShellDispatch4_GetTypeInfo,
    GetIDsOfNames: CShellDispatch_IShellDispatch4_GetIDsOfNames,
    Invoke: CShellDispatch_IShellDispatch4_Invoke,
    get_Application: CShellDispatch_IShellDispatch4_get_Application,
    get_Parent: CShellDispatch_IShellDispatch4_get_Parent,
    NameSpace: CShellDispatch_IShellDispatch4_NameSpace,
    BrowseForFolder: CShellDispatch_IShellDispatch4_BrowseForFolder,
    Windows: CShellDispatch_IShellDispatch4_Windows,
    Open: CShellDispatch_IShellDispatch4_Open,
    Explore: CShellDispatch_IShellDispatch4_Explore,
    MinimizeAll: CShellDispatch_IShellDispatch4_MinimizeAll,
    UndoMinimizeALL: CShellDispatch_IShellDispatch4_UndoMinimizeALL,
    FileRun: CShellDispatch_IShellDispatch4_FileRun,
    CascadeWindows: CShellDispatch_IShellDispatch4_CascadeWindows,
    TileVertically: CShellDispatch_IShellDispatch4_TileVertically,
    TileHorizontally: CShellDispatch_IShellDispatch4_TileHorizontally,
    ShutdownWindows: CShellDispatch_IShellDispatch4_ShutdownWindows,
    Suspend: CShellDispatch_IShellDispatch4_Suspend,
    EjectPC: CShellDispatch_IShellDispatch4_EjectPC,
    SetTime: CShellDispatch_IShellDispatch4_SetTime,
    TrayProperties: CShellDispatch_IShellDispatch4_TrayProperties,
    Help: CShellDispatch_IShellDispatch4_Help,
    FindFiles: CShellDispatch_IShellDispatch4_FindFiles,
    FindComputer: CShellDispatch_IShellDispatch4_FindComputer,
    RefreshMenu: CShellDispatch_IShellDispatch4_RefreshMenu,
    ControlPanelItem: CShellDispatch_IShellDispatch4_ControlPanelItem,
    IsRestricted: CShellDispatch_IShellDispatch4_IsRestricted,
    ShellExecute: CShellDispatch_IShellDispatch4_ShellExecute,
    FindPrinter: CShellDispatch_IShellDispatch4_FindPrinter,
    GetSystemInformation: CShellDispatch_IShellDispatch4_GetSystemInformation,
    ServiceStart: CShellDispatch_IShellDispatch4_ServiceStart,
    ServiceStop: CShellDispatch_IShellDispatch4_ServiceStop,
    IsServiceRunning: CShellDispatch_IShellDispatch4_IsServiceRunning,
    CanStartStopService: CShellDispatch_IShellDispatch4_CanStartStopService,
    ShowBrowserBar: CShellDispatch_IShellDispatch4_ShowBrowserBar,
    AddToRecent: CShellDispatch_IShellDispatch4_AddToRecent,
    WindowsSecurity: CShellDispatch_IShellDispatch4_WindowsSecurity,
    ToggleDesktop: CShellDispatch_IShellDispatch4_ToggleDesktop,
    ExplorerPolicy: CShellDispatch_IShellDispatch4_ExplorerPolicy,
    GetSetting: CShellDispatch_IShellDispatch4_GetSetting,
};

unsafe extern "system" fn CShellDispatch_IObjectSafety_QueryInterface(This: *mut IObjectSafety, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectSafety_iface)) as *mut CShellDispatch;
    CShellDispatch_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellDispatch_IObjectSafety_AddRef(This: *mut IObjectSafety) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectSafety_iface)) as *mut CShellDispatch;
    CShellDispatch_AddRef(this)
}
unsafe extern "system" fn CShellDispatch_IObjectSafety_Release(This: *mut IObjectSafety) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectSafety_iface)) as *mut CShellDispatch;
    CShellDispatch_Release(this)
}
unsafe extern "system" fn CShellDispatch_IObjectSafety_GetInterfaceSafetyOptions(This: *mut IObjectSafety, riid: REFIID, pdwSupportedOptions: *mut DWORD, pdwEnabledOptions: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectSafety_iface)) as *mut CShellDispatch;
    CShellDispatch_GetInterfaceSafetyOptions_body(this, riid, pdwSupportedOptions, pdwEnabledOptions)
}
unsafe extern "system" fn CShellDispatch_IObjectSafety_SetInterfaceSafetyOptions(This: *mut IObjectSafety, riid: REFIID, dwOptionSetMask: DWORD, dwEnabledOptions: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectSafety_iface)) as *mut CShellDispatch;
    CShellDispatch_SetInterfaceSafetyOptions_body(this, riid, dwOptionSetMask, dwEnabledOptions)
}

pub static CSHELLDISPATCH_IOBJECTSAFETY_VTBL: IObjectSafetyVtbl = IObjectSafetyVtbl {
    QueryInterface: CShellDispatch_IObjectSafety_QueryInterface,
    AddRef: CShellDispatch_IObjectSafety_AddRef,
    Release: CShellDispatch_IObjectSafety_Release,
    GetInterfaceSafetyOptions: CShellDispatch_IObjectSafety_GetInterfaceSafetyOptions,
    SetInterfaceSafetyOptions: CShellDispatch_IObjectSafety_SetInterfaceSafetyOptions,
};

unsafe extern "system" fn CShellDispatch_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectWithSite_iface)) as *mut CShellDispatch;
    CShellDispatch_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellDispatch_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectWithSite_iface)) as *mut CShellDispatch;
    CShellDispatch_AddRef(this)
}
unsafe extern "system" fn CShellDispatch_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectWithSite_iface)) as *mut CShellDispatch;
    CShellDispatch_Release(this)
}
unsafe extern "system" fn CShellDispatch_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectWithSite_iface)) as *mut CShellDispatch;
    CShellDispatch_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CShellDispatch_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellDispatch, IObjectWithSite_iface)) as *mut CShellDispatch;
    CShellDispatch_GetSite_body(this, riid, ppvSite)
}

pub static CSHELLDISPATCH_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CShellDispatch_IObjectWithSite_QueryInterface,
    AddRef: CShellDispatch_IObjectWithSite_AddRef,
    Release: CShellDispatch_IObjectWithSite_Release,
    SetSite: CShellDispatch_IObjectWithSite_SetSite,
    GetSite: CShellDispatch_IObjectWithSite_GetSite,
};

unsafe fn CShellDispatch_QueryInterface(
    this: *mut CShellDispatch, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellDispatch4)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellDispatch4_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellDispatch3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellDispatch4_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellDispatch2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellDispatch4_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellDispatch4_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDispatch)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellDispatch4_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectSafety)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectSafety_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CShellDispatch_AddRef(this);
    S_OK
}

unsafe fn CShellDispatch_AddRef(this: *mut CShellDispatch) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CShellDispatch_Release(this: *mut CShellDispatch) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CShellDispatch_dtor_body(this);
        CShellDispatch_cpp_com_assign_m_pTypeInfo(this, core::ptr::null_mut());
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_Shell currently maps to CShellDispatch in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CShellDispatch_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CShellDispatch>()) as *mut CShellDispatch;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CShellDispatch {
        IShellDispatch4_iface: IShellDispatch4 { lpVtbl: core::ptr::addr_of!(CSHELLDISPATCH_ISHELLDISPATCH4_VTBL) },
        IObjectSafety_iface: IObjectSafety { lpVtbl: core::ptr::addr_of!(CSHELLDISPATCH_IOBJECTSAFETY_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CSHELLDISPATCH_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pTypeInfo: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CShellDispatch_ctor_body(this);
    let hr = CShellDispatch_QueryInterface(this, riid, ppv);
    CShellDispatch_Release(this);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn CShellDispatch_Constructor(
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CShellDispatch_CreateInstance(core::ptr::null_mut(), riid, ppvOut.cast())
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDM_SECURITY: UINT = 5001;

pub const IDR_SHELL: UINT = 156;

pub const SSF_SERVERADMINUI: UINT = 4;

pub const SEE_MASK_IDLIST: ULONG = 0x0000_0004;

pub const BIF_NEWDIALOGSTYLE: DWORD = 0x0040;

pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4;

pub const ERROR_SERVICE_ALREADY_RUNNING: DWORD = 1056;

pub const ERROR_SERVICE_NOT_ACTIVE: DWORD = 1062;

pub const SC_MANAGER_CONNECT: DWORD = 1;

pub const SERVICE_CHANGE_CONFIG: DWORD = 2;

pub const SERVICE_CONTROL_STOP: DWORD = 1;

pub const SERVICE_NO_CHANGE: DWORD = 0xffffffff;

pub const SERVICE_QUERY_STATUS: DWORD = 4;

pub const SERVICE_RUNNING: DWORD = 4;

pub const SERVICE_START: DWORD = 16;

pub const SERVICE_STOP: DWORD = 32;

pub const SHARD_PIDL: DWORD = 0x00000001;

pub const SSF_DONTPRETTYPATH: DWORD = 0x00000800;

pub const SSF_NOCONFIRMRECYCLE: DWORD = 0x00008000;

pub const SSF_SEPPROCESS: DWORD = 0x00080000;

pub const SSF_SHOWALLOBJECTS: DWORD = 0x00000001;

pub const SSF_SHOWEXTENSIONS: DWORD = 0x00000002;

pub const SSF_SHOWSUPERHIDDEN: DWORD = 0x00040000;

pub const SSF_SHOWSYSFILES: DWORD = 0x00000020;

pub const SSF_STARTPANELON: DWORD = 0x00200000;

pub const SW_SHOW: INT = 5;

pub const TRAYCMD_CASCADE: DWORD = 403;

pub const TRAYCMD_DATE_AND_TIME: DWORD = 408;

pub const TRAYCMD_HELP_AND_SUPPORT: DWORD = 503;

pub const TRAYCMD_MINIMIZE_ALL: DWORD = 415;

pub const TRAYCMD_RESTORE_ALL: DWORD = 416;

pub const TRAYCMD_RUN_DIALOG: DWORD = 401;

pub const TRAYCMD_SEARCH_COMPUTERS: DWORD = 41094;

pub const TRAYCMD_SEARCH_FILES: DWORD = 41093;

pub const TRAYCMD_TASKBAR_PROPERTIES: DWORD = 413;

pub const TRAYCMD_TILE_H: DWORD = 404;

pub const TRAYCMD_TILE_V: DWORD = 405;

pub const TRAYCMD_TOGGLE_DESKTOP: DWORD = 407;

pub const VT_BOOL: VARTYPE = 11;

pub const VT_BSTR: VARTYPE = 8;

pub const VT_ERROR: VARTYPE = 10;

pub const VT_I2: VARTYPE = 2;

pub const VT_I4: VARTYPE = 3;

pub const WM_COMMAND: DWORD = 273;

#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANTARG,
    pub rgdispidNamedArgs: *mut DISPID,
    pub cArgs: UINT,
    pub cNamedArgs: UINT,
}

#[repr(C)]
pub struct IDLDESC {
    pub dwReserved: ULONG_PTR,
    pub wIDLFlags: USHORT,
}

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
}

#[repr(C)]
pub struct SERVICE_STATUS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
}

#[repr(C)]
pub struct SERVICE_STATUS_PROCESS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
    pub dwProcessId: DWORD,
    pub dwServiceFlags: DWORD,
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct Folder {
    pub lpVtbl: *const FolderVtbl,
}
#[repr(C)]
pub struct FolderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut Folder) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut Folder, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut Folder, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut Folder, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut Folder, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Title: unsafe extern "system" fn(This: *mut Folder, pbs: *mut BSTR) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut IDispatch) -> HRESULT,
    pub get_ParentFolder: unsafe extern "system" fn(This: *mut Folder, ppsf: *mut *mut Folder) -> HRESULT,
    pub Items: unsafe extern "system" fn(This: *mut Folder, ppid: *mut *mut FolderItems) -> HRESULT,
    pub ParseName: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, ppid: *mut *mut FolderItem) -> HRESULT,
    pub NewFolder: unsafe extern "system" fn(This: *mut Folder, bName: BSTR, vOptions: VARIANT) -> HRESULT,
    pub MoveHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub CopyHere: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, vOptions: VARIANT) -> HRESULT,
    pub GetDetailsOf: unsafe extern "system" fn(This: *mut Folder, vItem: VARIANT, iColumn: i32, pbs: *mut BSTR) -> HRESULT,
}
extern "C" {
    pub static IID_Folder: IID;
}

#[repr(C)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

extern "system" {
    fn GetLastError() -> DWORD;
    fn CoTaskMemFree(memory: LPVOID);
}

#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub const fn as_ptr(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Default for CComHeapPtr<T> {
    fn default() -> Self { Self::empty() }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

extern "C" {
    // CLSID_NULL is a guiddef.h macro alias, not a separate data symbol.
    #[link_name = "GUID_NULL"]
    static CLSID_NULL: GUID;
    static CLSID_Shell: GUID;
    static CLSID_ShellWindows: GUID;
}

pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_31AF3113D116: [WCHAR; 8] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_63ED30E64A87: [WCHAR; 14] = [0x0049, 0x0073, 0x004F, 0x0053, 0x005F, 0x0050, 0x0065, 0x0072, 0x0073, 0x006F, 0x006E, 0x0061, 0x006C, 0x0000];
pub static CPP_COM_WSTR_71AED3336CBB: [WCHAR; 18] = [0x0049, 0x0073, 0x004F, 0x0053, 0x005F, 0x0050, 0x0072, 0x006F, 0x0066, 0x0065, 0x0073, 0x0073, 0x0069, 0x006F, 0x006E, 0x0061, 0x006C, 0x0000];
pub static CPP_COM_WSTR_BB39C012AB62: [WCHAR; 14] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x005F, 0x0054, 0x0072, 0x0061, 0x0079, 0x0057, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_D53D8378BAB8: [WCHAR; 18] = [0x0049, 0x0073, 0x004F, 0x0053, 0x005F, 0x0044, 0x006F, 0x006D, 0x0061, 0x0069, 0x006E, 0x004D, 0x0065, 0x006D, 0x0062, 0x0065, 0x0072, 0x0000];
pub static CPP_COM_WSTR_FF4270D7D5C0: [WCHAR; 22] = [0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x006F, 0x0072, 0x0041, 0x0072, 0x0063, 0x0068, 0x0069, 0x0074, 0x0065, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::wine::shellord::SHGetSetSettings;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
use crate::wine::pidl::SHILCreateFromPathW;
use crate::wine::shellord::SHRunControlPanel;
use crate::shell32::shlexec::ShellExecuteExW;
use crate::shell32::shlexec::ShellExecuteW;
// One declaration per external symbol: island.rs.
use crate::island::CoCreateInstance;
use crate::island::FindWindowW;
use crate::island::GetSystemInfo;
use crate::island::OutputDebugStringA;
use crate::island::PostMessageW;
use crate::shellcom::*;
use crate::types::*;
use com_sys::{
    BINDPTR, BSTR, DESCKIND, DISPID, EXCEPINFO, FUNCDESC, HREFTYPE, INVOKEKIND,
    LCID, LPPARAMDESCEX, MEMBERID, TLIBATTR, TYPEATTR, TYPEKIND, VARDESC,
    VARIANTARG, VARIANT_BOOL,
};

pub type SC_HANDLE = HANDLE;
pub type BFFCALLBACK = Option<unsafe extern "system" fn(HWND, UINT, LPARAM, LPARAM) -> INT>;

#[repr(C)]
pub struct FolderItem { _opaque: [u8; 0] }
#[repr(C)]
pub struct FolderItems { _opaque: [u8; 0] }

pub use crate::island::BROWSEINFOW;

#[repr(C)]
pub struct SYSTEM_INFO {
    pub wProcessorArchitecture: USHORT,
    pub wReserved: USHORT,
    pub dwPageSize: DWORD,
    pub lpMinimumApplicationAddress: LPVOID,
    pub lpMaximumApplicationAddress: LPVOID,
    pub dwActiveProcessorMask: ULONG_PTR,
    pub dwNumberOfProcessors: DWORD,
    pub dwProcessorType: DWORD,
    pub dwAllocationGranularity: DWORD,
    pub wProcessorLevel: WORD,
    pub wProcessorRevision: WORD,
}

#[repr(C)]
pub struct SHELLSTATE {
    pub w0: u32,
    pub dwWin95Unused: DWORD,
    pub uWin95Unused: UINT,
    pub lParamSort: LONG,
    pub iSortDirection: INT,
    pub version: UINT,
    pub uNotUsed: UINT,
    pub w1: u32,
}

pub const VARIANT_TRUE: VARIANT_BOOL = -1;
pub const VARIANT_FALSE: VARIANT_BOOL = 0;
pub const DISP_E_BADINDEX: HRESULT = 0x8002_000B_u32 as HRESULT;
pub const DISP_E_PARAMNOTFOUND: HRESULT = 0x8002_0004_u32 as HRESULT;
pub const SEE_MASK_FLAG_DDEWAIT: ULONG = 0x0000_0100;
pub const OS_HOME: UINT = 19;
pub const OS_PROFESSIONAL: UINT = 20;
pub const OS_DOMAINMEMBER: UINT = 28;
pub const OS_SERVERADMINUI: UINT = 34;
pub const SERVICE_AUTO_START: DWORD = 2;
pub const SERVICE_DEMAND_START: DWORD = 3;
pub const SC_STATUS_PROCESS_INFO: INT = 0;

extern "C" {
    static LIBID_Shell32: GUID;
}

extern "system" {
    fn LoadRegTypeLib(libid: REFGUID, major: WORD, minor: WORD, lcid: LCID, out: *mut *mut ITypeLib) -> HRESULT;
    fn VariantChangeType(dst: *mut VARIANT, src: *const VARIANT, flags: USHORT, vt: VARTYPE) -> HRESULT;
    fn VariantInit(value: *mut VARIANT);
    fn VariantClear(value: *mut VARIANT) -> HRESULT;
    fn SHBrowseForFolderW(info: *mut BROWSEINFOW) -> LPITEMIDLIST;
    fn CFolder_CreateFromPidl(pidl: LPCITEMIDLIST, riid: REFIID, out: *mut *mut c_void) -> HRESULT;
    fn ExitWindowsDialog(hwnd: HWND);
    fn SHGetRestriction(subkey: LPCWSTR, subname: LPCWSTR, value: LPCWSTR) -> DWORD;
    fn lstrcmpiW(left: LPCWSTR, right: LPCWSTR) -> INT;
    fn IsOS(os: UINT) -> BOOL;
    fn OpenSCManagerW(machine: LPCWSTR, database: LPCWSTR, access: DWORD) -> SC_HANDLE;
    fn OpenServiceW(manager: SC_HANDLE, name: LPCWSTR, access: DWORD) -> SC_HANDLE;
    fn CloseServiceHandle(handle: SC_HANDLE) -> BOOL;
    fn ControlService(service: SC_HANDLE, control: DWORD, status: *mut SERVICE_STATUS) -> BOOL;
    fn StartServiceW(service: SC_HANDLE, argc: DWORD, argv: *const LPCWSTR) -> BOOL;
    fn ChangeServiceConfigW(service: SC_HANDLE, service_type: DWORD, start_type: DWORD, error_control: DWORD, binary_path: LPCWSTR, load_group: LPCWSTR, tag: LPDWORD, dependencies: LPCWSTR, account: LPCWSTR, password: LPCWSTR, display_name: LPCWSTR) -> BOOL;
    fn QueryServiceStatusEx(service: SC_HANDLE, info_level: INT, buffer: *mut BYTE, size: DWORD, needed: LPDWORD) -> BOOL;
    fn SHAddToRecentDocs(flags: UINT, item: *const c_void);
}

// oaidl.h's V_* macros. VARIANT is the island's `tagVARIANT`, so every member sits under
// the anonymous-union nest `n1.n2` / `n1.n2.n3` -- not a flat `vt`/`val` pair.
#[inline] unsafe fn V_VT(value: *const VARIANT) -> VARTYPE { (*value).n1.n2.vt }
#[inline] pub unsafe fn V_I4(value: *const VARIANT) -> LONG { (*value).n1.n2.n3.lVal }
#[inline] unsafe fn V_UI4(value: *const VARIANT) -> ULONG { (*value).n1.n2.n3.ulVal }
#[inline] unsafe fn V_ERROR(value: *const VARIANT) -> HRESULT { (*value).n1.n2.n3.scode }
#[inline] unsafe fn V_BOOL(value: *const VARIANT) -> VARIANT_BOOL { (*value).n1.n2.n3.boolVal }
#[inline] unsafe fn V_BSTR(value: *const VARIANT) -> BSTR { (*value).n1.n2.n3.bstrVal }

unsafe fn VariantToIdlist(value: *const VARIANT, idlist: *mut LPITEMIDLIST) -> HRESULT {
    if V_VT(value) == VT_I4 {
        SHGetSpecialFolderLocation(core::ptr::null_mut(), V_I4(value), idlist)
    } else if V_VT(value) == VT_BSTR {
        SHILCreateFromPathW(V_BSTR(value), idlist, core::ptr::null_mut())
    } else {
        E_FAIL
    }
}

#[inline] fn HResultFromWin32(error: DWORD) -> HRESULT { HRESULT_FROM_WIN32(error as LONG) }
#[inline] fn LongToHandle(value: LONG) -> HANDLE { value as isize as HANDLE }

pub unsafe fn CShellDispatch_EnsureTILoaded_body(this: *mut CShellDispatch, lcid: LCID) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    if !(*this).m_pTypeInfo.is_null() {
        return hr;
    }

    if cpp_com_iid_equal(&CLSID_NULL, &LIBID_Shell32) {
        OutputDebugStringA(
            (b"IDispatchImpl: plibid is CLSID_NULL!\r\n\0".as_ptr() as *const u8).cast(),
        );
    }

    let mut spTypeLib: *mut ITypeLib = core::ptr::null_mut();
    let _spTypeLib_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(spTypeLib));
    hr = LoadRegTypeLib(
        &LIBID_Shell32,
        1,
        0,
        lcid,
        &mut spTypeLib,
    );

    if hr >= 0 {
        hr = ((*(*spTypeLib).lpVtbl).GetTypeInfoOfGuid)(
            spTypeLib,
            &IID_IShellDispatch4,
            &mut (*this).m_pTypeInfo,
        );
    }

    if !spTypeLib.is_null() {
        ((*(*spTypeLib).lpVtbl).Release)(spTypeLib);
        spTypeLib = core::ptr::null_mut();
    }

    hr
}

pub unsafe fn PostTrayCommand_body(cmd: UINT) -> HRESULT {
    let hTrayWnd = FindWindowW(CPP_COM_WSTR_BB39C012AB62.as_ptr(), core::ptr::null());
    if !hTrayWnd.is_null() && PostMessageW(hTrayWnd, WM_COMMAND, cmd as WPARAM, 0) != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CShellDispatch_ctor_body(this: *mut CShellDispatch) -> () {}

pub unsafe fn CShellDispatch_dtor_body(this: *mut CShellDispatch) -> () {}

pub unsafe fn CShellDispatch_Initialize_body(this: *mut CShellDispatch) -> HRESULT { S_OK }

pub unsafe fn CShellDispatch_GetTypeInfoCount_v1_body(this: *mut CShellDispatch, pctinfo: *mut UINT) -> HRESULT {
    if pctinfo == core::ptr::null_mut() {
        return E_POINTER;
    }
    *pctinfo = 1;
    S_OK
}

pub unsafe fn CShellDispatch_get_Application_body(this: *mut CShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT {
    if ppid.is_null() {
        return E_INVALIDARG;
    }
    *ppid = this as *mut IDispatch;
    CShellDispatch_AddRef(this);
    S_OK
}

pub unsafe fn CShellDispatch_GetTypeInfo_v1_body(this: *mut CShellDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT {
    if iTInfo != 0 {
        return DISP_E_BADINDEX;
    }
    if ppTInfo.is_null() {
        return E_POINTER;
    }

    let hr = CShellDispatch_EnsureTILoaded_body(this, lcid);
    *ppTInfo = (*this).m_pTypeInfo;
    let pti = *ppTInfo;
    if !pti.is_null() {
        ((*(*pti).lpVtbl).AddRef)(pti);
    }

    hr
}

pub unsafe fn CShellDispatch_get_Parent_body(this: *mut CShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT {
    if !ppid.is_null() {
        *ppid = this as *mut IDispatch;
        CShellDispatch_AddRef(this);
    }
    S_OK
}

pub unsafe fn CShellDispatch_GetIDsOfNames_v1_body(this: *mut CShellDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT {
    let _: REFIID = riid;
    let hr = CShellDispatch_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        ((*(*(*this).m_pTypeInfo).lpVtbl).GetIDsOfNames)((*this).m_pTypeInfo, rgszNames, cNames, rgDispId)
    } else {
        hr
    }
}

pub unsafe fn CShellDispatch_Invoke_v1_body(this: *mut CShellDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT {
    let mut hr = CShellDispatch_EnsureTILoaded_body(this, lcid);
    if hr >= 0 {
        let type_info = (*this).m_pTypeInfo;
        hr = ((*(*type_info).lpVtbl).Invoke)(
            type_info,
            this as *mut core::ffi::c_void,
            dispIdMember,
            wFlags,
            pDispParams,
            pVarResult,
            pExcepInfo,
            puArgErr,
        );
    }
    hr
}

pub unsafe fn CShellDispatch_NameSpace_body(this: *mut CShellDispatch, mut vDir: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT {
    if ppsdf.is_null() {
        return E_POINTER;
    }
    *ppsdf = core::ptr::null_mut();
    let mut hr: HRESULT;

    if V_VT(&vDir) == VT_I2 {
        hr = VariantChangeType(&mut vDir, &vDir, 0, VT_I4);
        if ((hr) < 0) {
            return hr;
        }
    }

    let mut idlist: CComHeapPtr<ITEMIDLIST> = Default::default();
    hr = VariantToIdlist(&vDir, idlist.as_mut_slot());
    if hr < 0 {
        return S_FALSE;
    }

    CFolder_CreateFromPidl(idlist.as_ptr(), &IID_Folder, ppsdf.cast())
}

pub unsafe fn is_optional_argument_body(arg: *const VARIANT) -> BOOL {
    let result = V_VT(arg) == VT_ERROR && V_ERROR(arg) == DISP_E_PARAMNOTFOUND;
    result as BOOL
}

pub unsafe fn CShellDispatch_BrowseForFolder_body(this: *mut CShellDispatch, Hwnd: LONG, Title: BSTR, Options: LONG, RootFolder: VARIANT, ppsdf: *mut *mut Folder) -> HRESULT {
    // *ppsdf = NULL
    *ppsdf = core::ptr::null_mut();

    // BROWSEINFOW bi = { 0 };
    let mut bi: BROWSEINFOW = core::mem::zeroed();
    bi.hwndOwner = LongToHandle(Hwnd) as HWND;
    bi.lpszTitle = Title;
    bi.ulFlags = Options as UINT | BIF_NEWDIALOGSTYLE;

    let mut idlist_raw: LPITEMIDLIST = core::ptr::null_mut();
    let idlist_needs_free = if is_optional_argument_body(&RootFolder) == FALSE
        && VariantToIdlist(&RootFolder, &mut idlist_raw) == S_OK
    {
        bi.pidlRoot = idlist_raw;
        true
    } else {
        false
    };

    let selection_raw = SHBrowseForFolderW(&mut bi);
    if selection_raw.is_null() {
        if idlist_needs_free {
            CoTaskMemFree(idlist_raw as _);
        }
        return S_FALSE;
    }
    let selection_needs_free = true;

    let result = CFolder_CreateFromPidl(selection_raw, &IID_Folder, ppsdf.cast());

    // Emulate RAII destruction order (selection first, then idlist)
    if selection_needs_free {
        CoTaskMemFree(selection_raw as _);
    }
    if idlist_needs_free {
        CoTaskMemFree(idlist_raw as _);
    }

    result
}

pub unsafe fn CShellDispatch_Windows_body(this: *mut CShellDispatch, ppid: *mut *mut IDispatch) -> HRESULT {
    CoCreateInstance(
        &CLSID_ShellWindows,
        core::ptr::null_mut(),
        CLSCTX_LOCAL_SERVER,
        &IID_IDispatch,
        ppid.cast(),
    )
}

pub unsafe fn SHELL_OpenFolder_body(pidl: LPCITEMIDLIST, verb: LPCWSTR) -> HRESULT {
    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_IDLIST | SEE_MASK_FLAG_DDEWAIT;
    sei.hwnd = core::ptr::null_mut();
    sei.lpVerb = verb;
    sei.lpFile = core::ptr::null_mut();
    sei.lpParameters = core::ptr::null_mut();
    sei.lpDirectory = core::ptr::null_mut();
    sei.nShow = SW_SHOW;
    sei.lpIDList = pidl as LPVOID;
    if ShellExecuteExW((&raw mut sei).cast()) != 0 {
        return S_OK;
    }
    let error = GetLastError();
    HRESULT_FROM_WIN32(error as LONG)
}

pub unsafe fn OpenFolder_body(vDir: VARIANT, verb: LPCWSTR) -> HRESULT {
    let mut idlist: *mut ITEMIDLIST = core::ptr::null_mut();
    let hr = VariantToIdlist(&vDir, &mut idlist);
    if hr == S_OK && SHELL_OpenFolder_body(idlist, verb) == S_OK {
        if !idlist.is_null() {
            CoTaskMemFree(idlist as *mut core::ffi::c_void);
        }
        return S_OK;
    }
    if !idlist.is_null() {
        CoTaskMemFree(idlist as *mut core::ffi::c_void);
    }
    S_FALSE
}

pub unsafe fn CShellDispatch_Open_body(this: *mut CShellDispatch, vDir: VARIANT) -> HRESULT {
    OpenFolder_body(vDir, core::ptr::null())
}

pub unsafe fn CShellDispatch_Explore_body(this: *mut CShellDispatch, vDir: VARIANT) -> HRESULT {
    OpenFolder_body(vDir, CPP_COM_WSTR_31AF3113D116.as_ptr())
}

pub unsafe fn CShellDispatch_MinimizeAll_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_MINIMIZE_ALL)
}

pub unsafe fn CShellDispatch_UndoMinimizeALL_body(this: *mut CShellDispatch) -> HRESULT {
    return PostTrayCommand_body(TRAYCMD_RESTORE_ALL);
}

pub unsafe fn CShellDispatch_FileRun_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_RUN_DIALOG)
}

pub unsafe fn CShellDispatch_CascadeWindows_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_CASCADE)
}

pub unsafe fn CShellDispatch_TileVertically_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_TILE_V)
}

pub unsafe fn CShellDispatch_TileHorizontally_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_TILE_H)
}

pub unsafe fn CShellDispatch_ShutdownWindows_body(this: *mut CShellDispatch) -> HRESULT {
    let _ = this;
    unsafe {
        ExitWindowsDialog(core::ptr::null_mut());
    }
    S_OK
}

pub unsafe fn CShellDispatch_Suspend_body(this: *mut CShellDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_EjectPC_body(this: *mut CShellDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_SetTime_body(this: *mut CShellDispatch) -> HRESULT {
    return PostTrayCommand_body(TRAYCMD_DATE_AND_TIME);
}

pub unsafe fn CShellDispatch_TrayProperties_body(this: *mut CShellDispatch) -> HRESULT {
    unsafe { PostTrayCommand_body(TRAYCMD_TASKBAR_PROPERTIES) }
}

pub unsafe fn CShellDispatch_Help_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_HELP_AND_SUPPORT)
}

pub unsafe fn CShellDispatch_FindFiles_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_SEARCH_FILES)
}

pub unsafe fn CShellDispatch_FindComputer_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_SEARCH_COMPUTERS)
}

pub unsafe fn CShellDispatch_RefreshMenu_body(this: *mut CShellDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_ControlPanelItem_body(this: *mut CShellDispatch, szDir: BSTR) -> HRESULT {
    if SHRunControlPanel(szDir, core::ptr::null_mut()) != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CShellDispatch_IsRestricted_body(this: *mut CShellDispatch, group: BSTR, restriction: BSTR, value: *mut LONG) -> HRESULT {
    if value.is_null() {
        return E_INVALIDARG;
    }
    *value = SHGetRestriction(core::ptr::null_mut(), group, restriction) as LONG;
    S_OK
}

pub unsafe fn CShellDispatch_ShellExecute_body(this: *mut CShellDispatch, file: BSTR, v_args: VARIANT, v_dir: VARIANT, v_op: VARIANT, v_show: VARIANT) -> HRESULT {
    let mut args_str: VARIANT = core::mem::zeroed();
    VariantInit(&mut args_str);
    let mut dir_str: VARIANT = core::mem::zeroed();
    VariantInit(&mut dir_str);
    let mut op_str: VARIANT = core::mem::zeroed();
    VariantInit(&mut op_str);
    let mut show_int: VARIANT = core::mem::zeroed();
    VariantInit(&mut show_int);

    let mut args: *mut u16 = core::ptr::null_mut();
    let mut dir: *mut u16 = core::ptr::null_mut();
    let mut op: *mut u16 = core::ptr::null_mut();
    let mut show: i32 = SW_SHOW;

    let _ = VariantChangeType(
        &mut args_str as *mut VARIANT,
        &v_args as *const VARIANT,
        0,
        VT_BSTR,
    );
    if V_VT(&args_str as *const VARIANT) == VT_BSTR {
        args = V_BSTR(&args_str as *const VARIANT);
    }

    let _ = VariantChangeType(
        &mut dir_str as *mut VARIANT,
        &v_dir as *const VARIANT,
        0,
        VT_BSTR,
    );
    if V_VT(&dir_str as *const VARIANT) == VT_BSTR {
        dir = V_BSTR(&dir_str as *const VARIANT);
    }

    let _ = VariantChangeType(
        &mut op_str as *mut VARIANT,
        &v_op as *const VARIANT,
        0,
        VT_BSTR,
    );
    if V_VT(&op_str as *const VARIANT) == VT_BSTR {
        op = V_BSTR(&op_str as *const VARIANT);
    }

    let _ = VariantChangeType(
        &mut show_int as *mut VARIANT,
        &v_show as *const VARIANT,
        0,
        VT_I4,
    );
    if V_VT(&show_int as *const VARIANT) == VT_I4 {
        show = V_I4(&show_int as *const VARIANT);
    }

    let ret = ShellExecuteW(
        core::ptr::null_mut(),
        op,
        file,
        args,
        dir,
        show,
    );

    VariantClear(&mut args_str);
    VariantClear(&mut dir_str);
    VariantClear(&mut op_str);
    VariantClear(&mut show_int);

    if (ret as usize) > 32 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CShellDispatch_FindPrinter_body(this: *mut CShellDispatch, name: BSTR, location: BSTR, model: BSTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_GetSystemInformation_body(this: *mut CShellDispatch, name: BSTR, ret: *mut VARIANT) -> HRESULT {
    if lstrcmpiW(name, CPP_COM_WSTR_FF4270D7D5C0.as_ptr()) == 0 {
        let mut si: SYSTEM_INFO = core::mem::zeroed();
        GetSystemInfo((&mut si as *mut SYSTEM_INFO).cast());
        (*ret).n1.n2.vt = VT_I4;
        (*ret).n1.n2.n3.lVal = si.wProcessorArchitecture as LONG;
        return S_OK;
    }

    let mut os: u32 = 0;
    if lstrcmpiW(name, CPP_COM_WSTR_71AED3336CBB.as_ptr()) == 0 {
        os = OS_PROFESSIONAL;
    } else if lstrcmpiW(name, CPP_COM_WSTR_63ED30E64A87.as_ptr()) == 0 {
        os = OS_HOME;
    } else if lstrcmpiW(name, CPP_COM_WSTR_D53D8378BAB8.as_ptr()) == 0 {
        os = OS_DOMAINMEMBER;
    }

    if os != 0 {
        (*ret).n1.n2.vt = VT_BOOL;
        (*ret).n1.n2.n3.iVal = if IsOS(os) != 0 {
            VARIANT_TRUE
        } else {
            VARIANT_FALSE
        };
        return S_OK;
    }

    E_NOTIMPL
}

pub unsafe fn OpenServiceHelper_body(name: LPCWSTR, access: DWORD, hSvc: &mut SC_HANDLE) -> HRESULT {
    *hSvc = core::ptr::null_mut();
    let hScm: SC_HANDLE = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_CONNECT,
    );
    if hScm.is_null() {
        return HResultFromWin32(GetLastError());
    }
    let mut hr: HRESULT = S_OK;
    *hSvc = OpenServiceW(hScm, name, access);
    if (*hSvc).is_null() {
        hr = HResultFromWin32(GetLastError());
    }
    CloseServiceHandle(hScm);
    hr
}

pub unsafe fn SHELL32_ControlService_body(name: BSTR, control: DWORD, persistent: &mut VARIANT) -> HRESULT {
    let persist = V_VT(persistent as *const _) == VT_BOOL
        && V_BOOL(persistent as *mut _) != 0;
    let mut access: DWORD = if persist {
        SERVICE_CHANGE_CONFIG
    } else {
        0
    };
    match control {
        0 => access |= SERVICE_START,
        SERVICE_CONTROL_STOP => access |= SERVICE_STOP,
        _ => {}
    }
    let mut hSvc: SC_HANDLE = core::ptr::null_mut();
    let mut hr: HRESULT =
        OpenServiceHelper_body(name as *const u16, access, &mut hSvc);
    if hr >= 0 {
        let success: BOOL;
        let error: DWORD;
        let already: DWORD;
        if control != 0 {
            let mut ss: SERVICE_STATUS = core::mem::zeroed();
            success = ControlService(hSvc, control, &mut ss);
            error = GetLastError();
            already = ERROR_SERVICE_NOT_ACTIVE;
        } else {
            success = StartServiceW(hSvc, 0, core::ptr::null());
            error = GetLastError();
            already = ERROR_SERVICE_ALREADY_RUNNING;
        }
        hr = if success != 0 {
            S_OK
        } else if error == already {
            S_FALSE
        } else {
            HRESULT_FROM_WIN32(error as LONG)
        };
        if hr >= 0 && persist {
            ChangeServiceConfigW(
                hSvc,
                SERVICE_NO_CHANGE,
                if control != 0 {
                    SERVICE_DEMAND_START
                } else {
                    SERVICE_AUTO_START
                },
                SERVICE_NO_CHANGE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        CloseServiceHandle(hSvc);
    }
    hr
}

pub unsafe fn CShellDispatch_ServiceStart_body(this: *mut CShellDispatch, service: BSTR, persistent: VARIANT, ret: *mut VARIANT) -> HRESULT {
    let mut persistent_mut = persistent;
    let hr = SHELL32_ControlService_body(service, 0, &mut persistent_mut);
    (*ret).n1.n2.vt = VT_BOOL;
    (*ret).n1.n2.n3.iVal = if hr == S_OK { VARIANT_TRUE } else { VARIANT_FALSE };
    if hr == S_OK { S_OK } else { S_FALSE }
}

pub unsafe fn CShellDispatch_ServiceStop_body(this: *mut CShellDispatch, service: BSTR, mut persistent: VARIANT, ret: *mut VARIANT) -> HRESULT {
    let hr = SHELL32_ControlService_body(service, SERVICE_CONTROL_STOP, &mut persistent);
    (*ret).n1.n2.vt = VT_BOOL;
    (*ret).n1.n2.n3.iVal = if hr == S_OK { VARIANT_TRUE } else { VARIANT_FALSE };
    if hr == S_OK { S_OK } else { S_FALSE }
}

pub unsafe fn CShellDispatch_IsServiceRunning_body(this: *mut CShellDispatch, name: BSTR, running: *mut VARIANT) -> HRESULT {
    let mut status: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut scm: SC_HANDLE;
    let mut service: SC_HANDLE;
    let mut dummy: DWORD = 0;

    (*running).n1.n2.vt = VT_BOOL;
    (*running).n1.n2.n3.iVal = VARIANT_FALSE;

    scm = OpenSCManagerW(core::ptr::null_mut(), core::ptr::null(), SC_MANAGER_CONNECT);
    if scm.is_null() {
        return S_OK;
    }

    service = OpenServiceW(scm, name, SERVICE_QUERY_STATUS);
    if service.is_null() {
        CloseServiceHandle(scm);
        return S_OK;
    }

    if QueryServiceStatusEx(
        service,
        SC_STATUS_PROCESS_INFO,
        &mut status as *mut SERVICE_STATUS_PROCESS as *mut BYTE,
        core::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
        &mut dummy,
    ) == 0
    {
        CloseServiceHandle(service);
        CloseServiceHandle(scm);
        return S_OK;
    }

    if status.dwCurrentState == SERVICE_RUNNING {
        (*running).n1.n2.n3.iVal = VARIANT_TRUE;
    }

    CloseServiceHandle(service);
    CloseServiceHandle(scm);

    S_OK
}

pub unsafe fn CShellDispatch_CanStartStopService_body(this: *mut CShellDispatch, service: BSTR, ret: *mut VARIANT) -> HRESULT {
    let mut hSvc: SC_HANDLE = 0 as SC_HANDLE;
    let hr = OpenServiceHelper_body(service, SERVICE_START | SERVICE_STOP, &mut hSvc);
    if hr >= 0 {
        CloseServiceHandle(hSvc);
    }
    (*ret).n1.n2.vt = VT_BOOL;
    (*ret).n1.n2.n3.iVal = if hr == S_OK { VARIANT_TRUE } else { VARIANT_FALSE };
    S_OK
}

pub unsafe fn CShellDispatch_ShowBrowserBar_body(this: *mut CShellDispatch, clsid: BSTR, show: VARIANT, ret: *mut VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_AddToRecent_body(this: *mut CShellDispatch, file: VARIANT, category: BSTR) -> HRESULT {
    let mut idlist: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut hr = VariantToIdlist(&file, &mut idlist);
    if hr == S_OK {
        SHAddToRecentDocs(SHARD_PIDL, idlist as *const _);
    } else {
        hr = S_FALSE;
    }
    if !idlist.is_null() {
        CoTaskMemFree(idlist as *mut _);
    }
    hr
}

pub unsafe fn CShellDispatch_WindowsSecurity_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(IDM_SECURITY)
}

pub unsafe fn CShellDispatch_ToggleDesktop_body(this: *mut CShellDispatch) -> HRESULT {
    PostTrayCommand_body(TRAYCMD_TOGGLE_DESKTOP)
}

pub unsafe fn CShellDispatch_ExplorerPolicy_body(this: *mut CShellDispatch, policy: BSTR, value: *mut VARIANT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_GetSetting_body(this: *mut CShellDispatch, setting: LONG, result: *mut VARIANT_BOOL) -> HRESULT {
    let mut flag: i32 = -1;
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(((&raw mut ss) as *mut SHELLSTATE).cast(), setting as DWORD, FALSE);
    match setting {
        x if x == SSF_SHOWALLOBJECTS as LONG => flag = ((ss.w0 >> 0) & 1) as i32,
        x if x == SSF_SHOWEXTENSIONS as LONG => flag = ((ss.w0 >> 1) & 1) as i32,
        x if x == SSF_SHOWSYSFILES as LONG => flag = ((ss.w0 >> 3) & 1) as i32,
        x if x == SSF_DONTPRETTYPATH as LONG => flag = ((ss.w0 >> 8) & 1) as i32,
        x if x == SSF_NOCONFIRMRECYCLE as LONG => flag = ((ss.w0 >> 2) & 1) as i32,
        x if x == SSF_SHOWSUPERHIDDEN as LONG => flag = ((ss.w0 >> 15) & 1) as i32,
        x if x == SSF_SEPPROCESS as LONG => flag = ((ss.w1 >> 0) & 1) as i32,
        x if x == SSF_STARTPANELON as LONG => flag = ((ss.w1 >> 1) & 1) as i32,
        x if x == SSF_SERVERADMINUI as LONG => flag = IsOS(OS_SERVERADMINUI) as i32,
        _ => {}
    }
    if flag >= 0 {
        *result = if flag != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
        return S_OK;
    }
    S_FALSE
}

pub unsafe fn CShellDispatch_GetInterfaceSafetyOptions_body(this: *mut CShellDispatch, riid: REFIID, pdwSupportedOptions: *mut DWORD, pdwEnabledOptions: *mut DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_SetInterfaceSafetyOptions_body(this: *mut CShellDispatch, riid: REFIID, dwOptionSetMask: DWORD, dwEnabledOptions: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_SetSite_body(this: *mut CShellDispatch, pUnkSite: *mut IUnknown) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellDispatch_GetSite_body(this: *mut CShellDispatch, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT { E_NOTIMPL }
