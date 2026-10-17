// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellLinkA {
    pub lpVtbl: *const IShellLinkAVtbl,
}
#[repr(C)]
pub struct IShellLinkAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellLinkA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellLinkA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellLinkA) -> ULONG,
    pub GetPath: unsafe extern "system" fn(This: *mut IShellLinkA, pszFile: LPSTR, cch: i32, pfd: *mut WIN32_FIND_DATAA, fFlags: DWORD) -> HRESULT,
    pub GetIDList: unsafe extern "system" fn(This: *mut IShellLinkA, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT,
    pub SetIDList: unsafe extern "system" fn(This: *mut IShellLinkA, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetDescription: unsafe extern "system" fn(This: *mut IShellLinkA, pszName: LPSTR, cch: i32) -> HRESULT,
    pub SetDescription: unsafe extern "system" fn(This: *mut IShellLinkA, pszName: LPCSTR) -> HRESULT,
    pub GetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkA, pszDir: LPSTR, cch: i32) -> HRESULT,
    pub SetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkA, pszDir: LPCSTR) -> HRESULT,
    pub GetArguments: unsafe extern "system" fn(This: *mut IShellLinkA, pszArgs: LPSTR, cch: i32) -> HRESULT,
    pub SetArguments: unsafe extern "system" fn(This: *mut IShellLinkA, pszArgs: LPCSTR) -> HRESULT,
    pub GetHotkey: unsafe extern "system" fn(This: *mut IShellLinkA, pwHotkey: *mut WORD) -> HRESULT,
    pub SetHotkey: unsafe extern "system" fn(This: *mut IShellLinkA, wHotkey: WORD) -> HRESULT,
    pub GetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkA, piShowCmd: *mut i32) -> HRESULT,
    pub SetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkA, iShowCmd: i32) -> HRESULT,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkA, pszIconPath: LPSTR, cch: i32, piIcon: *mut i32) -> HRESULT,
    pub SetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkA, pszIconPath: LPCSTR, iIcon: i32) -> HRESULT,
    pub SetRelativePath: unsafe extern "system" fn(This: *mut IShellLinkA, pszPathRel: LPCSTR, dwReserved: DWORD) -> HRESULT,
    pub Resolve: unsafe extern "system" fn(This: *mut IShellLinkA, hwnd: HWND, fFlags: DWORD) -> HRESULT,
    pub SetPath: unsafe extern "system" fn(This: *mut IShellLinkA, pszFile: LPCSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellLinkA: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellLinkW {
    pub lpVtbl: *const IShellLinkWVtbl,
}
#[repr(C)]
pub struct IShellLinkWVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellLinkW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellLinkW) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellLinkW) -> ULONG,
    pub GetPath: unsafe extern "system" fn(This: *mut IShellLinkW, pszFile: LPWSTR, cch: i32, pfd: *mut WIN32_FIND_DATAW, fFlags: DWORD) -> HRESULT,
    pub GetIDList: unsafe extern "system" fn(This: *mut IShellLinkW, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT,
    pub SetIDList: unsafe extern "system" fn(This: *mut IShellLinkW, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetDescription: unsafe extern "system" fn(This: *mut IShellLinkW, pszName: LPWSTR, cch: i32) -> HRESULT,
    pub SetDescription: unsafe extern "system" fn(This: *mut IShellLinkW, pszName: LPCWSTR) -> HRESULT,
    pub GetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkW, pszDir: LPWSTR, cch: i32) -> HRESULT,
    pub SetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkW, pszDir: LPCWSTR) -> HRESULT,
    pub GetArguments: unsafe extern "system" fn(This: *mut IShellLinkW, pszArgs: LPWSTR, cch: i32) -> HRESULT,
    pub SetArguments: unsafe extern "system" fn(This: *mut IShellLinkW, pszArgs: LPCWSTR) -> HRESULT,
    pub GetHotkey: unsafe extern "system" fn(This: *mut IShellLinkW, pwHotkey: *mut WORD) -> HRESULT,
    pub SetHotkey: unsafe extern "system" fn(This: *mut IShellLinkW, wHotkey: WORD) -> HRESULT,
    pub GetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkW, piShowCmd: *mut i32) -> HRESULT,
    pub SetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkW, iShowCmd: i32) -> HRESULT,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkW, pszIconPath: LPWSTR, cch: i32, piIcon: *mut i32) -> HRESULT,
    pub SetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkW, pszIconPath: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetRelativePath: unsafe extern "system" fn(This: *mut IShellLinkW, pszPathRel: LPCWSTR, dwReserved: DWORD) -> HRESULT,
    pub Resolve: unsafe extern "system" fn(This: *mut IShellLinkW, hwnd: HWND, fFlags: DWORD) -> HRESULT,
    pub SetPath: unsafe extern "system" fn(This: *mut IShellLinkW, pszFile: LPCWSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellLinkW: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT,
}
extern "C" {
    pub static IID_IPersist: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFile: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistStream {
    pub lpVtbl: *const IPersistStreamVtbl,
}
#[repr(C)]
pub struct IPersistStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistStream) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistStream, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistStream) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistStream, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistStream, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IPersistStream, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistStream: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}
#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT,
}
extern "C" {
    pub static IID_IShellExtInit: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IContextMenu {
    pub lpVtbl: *const IContextMenuVtbl,
}
#[repr(C)]
pub struct IContextMenuVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IContextMenu) -> ULONG,
    pub QueryContextMenu: unsafe extern "system" fn(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT,
    pub GetCommandString: unsafe extern "system" fn(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IContextMenu: IID;
}

// Deterministic from oleidl.h
#[repr(C)]
pub struct IDropTarget {
    pub lpVtbl: *const IDropTargetVtbl,
}
#[repr(C)]
pub struct IDropTargetVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDropTarget) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(This: *mut IDropTarget) -> HRESULT,
    pub Drop: unsafe extern "system" fn(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IDropTarget: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellLinkDataList {
    pub lpVtbl: *const IShellLinkDataListVtbl,
}
#[repr(C)]
pub struct IShellLinkDataListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellLinkDataList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellLinkDataList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellLinkDataList) -> ULONG,
    pub AddDataBlock: unsafe extern "system" fn(This: *mut IShellLinkDataList, pDataBlock: *mut core::ffi::c_void) -> HRESULT,
    pub CopyDataBlock: unsafe extern "system" fn(This: *mut IShellLinkDataList, dwSig: DWORD, ppDataBlock: *mut *mut core::ffi::c_void) -> HRESULT,
    pub RemoveDataBlock: unsafe extern "system" fn(This: *mut IShellLinkDataList, dwSig: DWORD) -> HRESULT,
    pub GetFlags: unsafe extern "system" fn(This: *mut IShellLinkDataList, pdwFlags: *mut DWORD) -> HRESULT,
    pub SetFlags: unsafe extern "system" fn(This: *mut IShellLinkDataList, dwFlags: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IShellLinkDataList: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IExtractIconA {
    pub lpVtbl: *const IExtractIconAVtbl,
}
#[repr(C)]
pub struct IExtractIconAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExtractIconA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExtractIconA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExtractIconA) -> ULONG,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IExtractIconA, uFlags: UINT, szIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT,
    pub Extract: unsafe extern "system" fn(This: *mut IExtractIconA, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IExtractIconA: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IExtractIconW {
    pub lpVtbl: *const IExtractIconWVtbl,
}
#[repr(C)]
pub struct IExtractIconWVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExtractIconW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExtractIconW) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExtractIconW) -> ULONG,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IExtractIconW, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT,
    pub Extract: unsafe extern "system" fn(This: *mut IExtractIconW, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IExtractIconW: IID;
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

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellPropSheetExt {
    pub lpVtbl: *const IShellPropSheetExtVtbl,
}
#[repr(C)]
pub struct IShellPropSheetExtVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellPropSheetExt) -> ULONG,
    pub AddPages: unsafe extern "system" fn(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
    pub ReplacePage: unsafe extern "system" fn(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellPropSheetExt: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IStream, pv: *mut core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(This: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}
extern "C" {
    pub static IID_IStream: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IDataObject {
    pub lpVtbl: *const IDataObjectVtbl,
}
#[repr(C)]
pub struct IDataObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDataObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDataObject) -> ULONG,
    pub GetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub GetDataHere: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT,
    pub QueryGetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC) -> HRESULT,
    pub GetCanonicalFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, pformatectIn: *mut FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT,
    pub SetData: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, pmedium: *mut STGMEDIUM, fRelease: BOOL) -> HRESULT,
    pub EnumFormatEtc: unsafe extern "system" fn(This: *mut IDataObject, dwDirection: DWORD, ppenumFormatEtc: *mut *mut IEnumFORMATETC) -> HRESULT,
    pub DAdvise: unsafe extern "system" fn(This: *mut IDataObject, pformatetc: *mut FORMATETC, advf: DWORD, pAdvSink: *mut IAdviseSink, pdwConnection: *mut DWORD) -> HRESULT,
    pub DUnadvise: unsafe extern "system" fn(This: *mut IDataObject, dwConnection: DWORD) -> HRESULT,
    pub EnumDAdvise: unsafe extern "system" fn(This: *mut IDataObject, ppenumAdvise: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IDataObject: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IAdviseSink {
    pub lpVtbl: *const IAdviseSinkVtbl,
}
#[repr(C)]
pub struct IAdviseSinkVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAdviseSink, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub OnDataChange: unsafe extern "system" fn(This: *mut IAdviseSink, pFormatetc: *mut FORMATETC, pStgmed: *mut STGMEDIUM) -> core::ffi::c_void,
    pub OnViewChange: unsafe extern "system" fn(This: *mut IAdviseSink, dwAspect: DWORD, lindex: LONG) -> core::ffi::c_void,
    pub OnRename: unsafe extern "system" fn(This: *mut IAdviseSink, pmk: *mut IMoniker) -> core::ffi::c_void,
    pub OnSave: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
    pub OnClose: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_IAdviseSink: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumFORMATETC {
    pub lpVtbl: *const IEnumFORMATETCVtbl,
}
#[repr(C)]
pub struct IEnumFORMATETCVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumFORMATETC, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG, rgelt: *mut FORMATETC, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumFORMATETC: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumSTATDATA {
    pub lpVtbl: *const IEnumSTATDATAVtbl,
}
#[repr(C)]
pub struct IEnumSTATDATAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumSTATDATA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG, rgelt: *mut STATDATA, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumSTATDATA, ppenum: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumSTATDATA: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: REFIID, ppvResult: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, dwReduceHowFar: DWORD, ppmkToLeft: *mut *mut IMoniker, ppmkReduced: *mut *mut IMoniker) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT,
    pub Enum: unsafe extern "system" fn(This: *mut IMoniker, fForward: BOOL, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pmkNewlyRunning: *mut IMoniker) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pFileTime: *mut FILETIME) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkPrefix: *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkRelPath: *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBindCtx, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(This: *mut IBindCtx, pprot: *mut *mut IRunningObjectTable) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, punk: *mut IUnknown) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumMoniker {
    pub lpVtbl: *const IEnumMonikerVtbl,
}
#[repr(C)]
pub struct IEnumMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG, rgelt: *mut *mut IMoniker, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumString {
    pub lpVtbl: *const IEnumStringVtbl,
}
#[repr(C)]
pub struct IEnumStringVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumString, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG, rgelt: *mut LPOLESTR, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumString) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumString, ppenum: *mut *mut IEnumString) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumString: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRunningObjectTable, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT,
    pub Revoke: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD, pfiletime: *mut FILETIME) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, pfiletime: *mut FILETIME) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IRunningObjectTable: IID;
}

pub use crate::island::SHELL_LINK_HEADER;

#[repr(C, packed)]
pub struct volume_info {
    pub r#type: DWORD,
    pub serial: DWORD,
    pub label: [WCHAR; 12],
}

// Deterministic COM scaffold for CShellLink; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"]
    fn Win32HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"]
    fn Win32HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
// C-shaped adapters: C++ call sites pass any pointer/byte-count flavour.
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
impl CppComSize for i32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    Win32HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    Win32HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CShellLink {
    pub IShellLinkA_iface: IShellLinkA,
    pub IShellLinkW_iface: IShellLinkW,
    pub IPersist_iface: IPersist,
    pub IPersistFile_iface: IPersistFile,
    pub IPersistStream_iface: IPersistStream,
    pub IShellExtInit_iface: IShellExtInit,
    pub IContextMenu_iface: IContextMenu,
    pub IDropTarget_iface: IDropTarget,
    pub IShellLinkDataList_iface: IShellLinkDataList,
    pub IExtractIconA_iface: IExtractIconA,
    pub IExtractIconW_iface: IExtractIconW,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub IShellPropSheetExt_iface: IShellPropSheetExt,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_Header: SHELL_LINK_HEADER,
    pub m_pPidl: LPITEMIDLIST,
    pub m_sPath: LPWSTR,
    pub volume: volume_info,
    pub m_sDescription: LPWSTR,
    pub m_sPathRel: LPWSTR,
    pub m_sWorkDir: LPWSTR,
    pub m_sArgs: LPWSTR,
    pub m_sIcoPath: LPWSTR,
    pub m_bRunAs: BOOL,
    pub m_bDirty: BOOL,
    pub m_pDBList: LPDBLIST,
    pub m_bInInit: BOOL,
    pub m_hIcon: HICON,
    pub sProduct: LPWSTR,
    pub sComponent: LPWSTR,
    pub m_sLinkPath: LPWSTR,
    pub m_site: *mut IUnknown,
    pub m_DropTarget: *mut IDropTarget,
}

unsafe fn CShellLink_cpp_com_assign_m_site(this: *mut CShellLink, value: *mut IUnknown) {
    let old = (*this).m_site;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_site = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CShellLink_cpp_com_assign_m_DropTarget(this: *mut CShellLink, value: *mut IDropTarget) {
    let old = (*this).m_DropTarget;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_DropTarget = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CShellLink_IShellLinkA_QueryInterface(This: *mut IShellLinkA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IShellLinkA_AddRef(This: *mut IShellLinkA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IShellLinkA_Release(This: *mut IShellLinkA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetPath(This: *mut IShellLinkA, pszFile: LPSTR, cch: i32, pfd: *mut WIN32_FIND_DATAA, fFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetPath_v1_body(this, pszFile, cch, pfd, fFlags)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetIDList(This: *mut IShellLinkA, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetIDList_body(this, ppidl)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetIDList(This: *mut IShellLinkA, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetIDList_body(this, pidl)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetDescription(This: *mut IShellLinkA, pszName: LPSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetDescription_v1_body(this, pszName, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetDescription(This: *mut IShellLinkA, pszName: LPCSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetDescription_v1_body(this, pszName)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetWorkingDirectory(This: *mut IShellLinkA, pszDir: LPSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetWorkingDirectory_v1_body(this, pszDir, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetWorkingDirectory(This: *mut IShellLinkA, pszDir: LPCSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetWorkingDirectory_v1_body(this, pszDir)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetArguments(This: *mut IShellLinkA, pszArgs: LPSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetArguments_v1_body(this, pszArgs, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetArguments(This: *mut IShellLinkA, pszArgs: LPCSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetArguments_v1_body(this, pszArgs)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetHotkey(This: *mut IShellLinkA, pwHotkey: *mut WORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetHotkey_body(this, pwHotkey)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetHotkey(This: *mut IShellLinkA, wHotkey: WORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetHotkey_body(this, wHotkey)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetShowCmd(This: *mut IShellLinkA, piShowCmd: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetShowCmd_body(this, piShowCmd)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetShowCmd(This: *mut IShellLinkA, iShowCmd: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetShowCmd_body(this, iShowCmd)
}
unsafe extern "system" fn CShellLink_IShellLinkA_GetIconLocation(This: *mut IShellLinkA, pszIconPath: LPSTR, cch: i32, piIcon: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_GetIconLocation_v1_body(this, pszIconPath, cch, piIcon)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetIconLocation(This: *mut IShellLinkA, pszIconPath: LPCSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetIconLocation_v1_body(this, pszIconPath, iIcon)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetRelativePath(This: *mut IShellLinkA, pszPathRel: LPCSTR, dwReserved: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetRelativePath_v1_body(this, pszPathRel, dwReserved)
}
unsafe extern "system" fn CShellLink_IShellLinkA_Resolve(This: *mut IShellLinkA, hwnd: HWND, fFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_Resolve_body(this, hwnd, fFlags)
}
unsafe extern "system" fn CShellLink_IShellLinkA_SetPath(This: *mut IShellLinkA, pszFile: LPCSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkA_iface)) as *mut CShellLink;
    CShellLink_SetPath_v1_body(this, pszFile)
}

pub static CSHELLLINK_ISHELLLINKA_VTBL: IShellLinkAVtbl = IShellLinkAVtbl {
    QueryInterface: CShellLink_IShellLinkA_QueryInterface,
    AddRef: CShellLink_IShellLinkA_AddRef,
    Release: CShellLink_IShellLinkA_Release,
    GetPath: CShellLink_IShellLinkA_GetPath,
    GetIDList: CShellLink_IShellLinkA_GetIDList,
    SetIDList: CShellLink_IShellLinkA_SetIDList,
    GetDescription: CShellLink_IShellLinkA_GetDescription,
    SetDescription: CShellLink_IShellLinkA_SetDescription,
    GetWorkingDirectory: CShellLink_IShellLinkA_GetWorkingDirectory,
    SetWorkingDirectory: CShellLink_IShellLinkA_SetWorkingDirectory,
    GetArguments: CShellLink_IShellLinkA_GetArguments,
    SetArguments: CShellLink_IShellLinkA_SetArguments,
    GetHotkey: CShellLink_IShellLinkA_GetHotkey,
    SetHotkey: CShellLink_IShellLinkA_SetHotkey,
    GetShowCmd: CShellLink_IShellLinkA_GetShowCmd,
    SetShowCmd: CShellLink_IShellLinkA_SetShowCmd,
    GetIconLocation: CShellLink_IShellLinkA_GetIconLocation,
    SetIconLocation: CShellLink_IShellLinkA_SetIconLocation,
    SetRelativePath: CShellLink_IShellLinkA_SetRelativePath,
    Resolve: CShellLink_IShellLinkA_Resolve,
    SetPath: CShellLink_IShellLinkA_SetPath,
};

unsafe extern "system" fn CShellLink_IShellLinkW_QueryInterface(This: *mut IShellLinkW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IShellLinkW_AddRef(This: *mut IShellLinkW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IShellLinkW_Release(This: *mut IShellLinkW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetPath(This: *mut IShellLinkW, pszFile: LPWSTR, cch: i32, pfd: *mut WIN32_FIND_DATAW, fFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetPath_v2_body(this, pszFile, cch, pfd, fFlags)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetIDList(This: *mut IShellLinkW, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetIDList_body(this, ppidl)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetIDList(This: *mut IShellLinkW, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetIDList_body(this, pidl)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetDescription(This: *mut IShellLinkW, pszName: LPWSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetDescription_v2_body(this, pszName, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetDescription(This: *mut IShellLinkW, pszName: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetDescription_v2_body(this, pszName)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetWorkingDirectory(This: *mut IShellLinkW, pszDir: LPWSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetWorkingDirectory_v2_body(this, pszDir, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetWorkingDirectory(This: *mut IShellLinkW, pszDir: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetWorkingDirectory_v2_body(this, pszDir)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetArguments(This: *mut IShellLinkW, pszArgs: LPWSTR, cch: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetArguments_v2_body(this, pszArgs, cch)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetArguments(This: *mut IShellLinkW, pszArgs: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetArguments_v2_body(this, pszArgs)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetHotkey(This: *mut IShellLinkW, pwHotkey: *mut WORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetHotkey_body(this, pwHotkey)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetHotkey(This: *mut IShellLinkW, wHotkey: WORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetHotkey_body(this, wHotkey)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetShowCmd(This: *mut IShellLinkW, piShowCmd: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetShowCmd_body(this, piShowCmd)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetShowCmd(This: *mut IShellLinkW, iShowCmd: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetShowCmd_body(this, iShowCmd)
}
unsafe extern "system" fn CShellLink_IShellLinkW_GetIconLocation(This: *mut IShellLinkW, pszIconPath: LPWSTR, cch: i32, piIcon: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_GetIconLocation_v2_body(this, pszIconPath, cch, piIcon)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetIconLocation(This: *mut IShellLinkW, pszIconPath: LPCWSTR, iIcon: i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetIconLocation_v2_body(this, pszIconPath, iIcon)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetRelativePath(This: *mut IShellLinkW, pszPathRel: LPCWSTR, dwReserved: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetRelativePath_v2_body(this, pszPathRel, dwReserved)
}
unsafe extern "system" fn CShellLink_IShellLinkW_Resolve(This: *mut IShellLinkW, hwnd: HWND, fFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_Resolve_body(this, hwnd, fFlags)
}
unsafe extern "system" fn CShellLink_IShellLinkW_SetPath(This: *mut IShellLinkW, pszFile: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkW_iface)) as *mut CShellLink;
    CShellLink_SetPath_v2_body(this, pszFile)
}

pub static CSHELLLINK_ISHELLLINKW_VTBL: IShellLinkWVtbl = IShellLinkWVtbl {
    QueryInterface: CShellLink_IShellLinkW_QueryInterface,
    AddRef: CShellLink_IShellLinkW_AddRef,
    Release: CShellLink_IShellLinkW_Release,
    GetPath: CShellLink_IShellLinkW_GetPath,
    GetIDList: CShellLink_IShellLinkW_GetIDList,
    SetIDList: CShellLink_IShellLinkW_SetIDList,
    GetDescription: CShellLink_IShellLinkW_GetDescription,
    SetDescription: CShellLink_IShellLinkW_SetDescription,
    GetWorkingDirectory: CShellLink_IShellLinkW_GetWorkingDirectory,
    SetWorkingDirectory: CShellLink_IShellLinkW_SetWorkingDirectory,
    GetArguments: CShellLink_IShellLinkW_GetArguments,
    SetArguments: CShellLink_IShellLinkW_SetArguments,
    GetHotkey: CShellLink_IShellLinkW_GetHotkey,
    SetHotkey: CShellLink_IShellLinkW_SetHotkey,
    GetShowCmd: CShellLink_IShellLinkW_GetShowCmd,
    SetShowCmd: CShellLink_IShellLinkW_SetShowCmd,
    GetIconLocation: CShellLink_IShellLinkW_GetIconLocation,
    SetIconLocation: CShellLink_IShellLinkW_SetIconLocation,
    SetRelativePath: CShellLink_IShellLinkW_SetRelativePath,
    Resolve: CShellLink_IShellLinkW_Resolve,
    SetPath: CShellLink_IShellLinkW_SetPath,
};

unsafe extern "system" fn CShellLink_IPersist_QueryInterface(This: *mut IPersist, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersist_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IPersist_AddRef(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersist_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IPersist_Release(This: *mut IPersist) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersist_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IPersist_GetClassID(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersist_iface)) as *mut CShellLink;
    CShellLink_GetClassID_body(this, pClassID)
}

pub static CSHELLLINK_IPERSIST_VTBL: IPersistVtbl = IPersistVtbl {
    QueryInterface: CShellLink_IPersist_QueryInterface,
    AddRef: CShellLink_IPersist_AddRef,
    Release: CShellLink_IPersist_Release,
    GetClassID: CShellLink_IPersist_GetClassID,
};

unsafe extern "system" fn CShellLink_IPersistFile_QueryInterface(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IPersistFile_AddRef(This: *mut IPersistFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IPersistFile_Release(This: *mut IPersistFile) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IPersistFile_GetClassID(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CShellLink_IPersistFile_IsDirty(This: *mut IPersistFile) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_IsDirty_body(this)
}
unsafe extern "system" fn CShellLink_IPersistFile_Load(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_Load_v1_body(this, pszFileName, dwMode)
}
unsafe extern "system" fn CShellLink_IPersistFile_Save(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_Save_v1_body(this, pszFileName, fRemember)
}
unsafe extern "system" fn CShellLink_IPersistFile_SaveCompleted(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_SaveCompleted_body(this, pszFileName)
}
unsafe extern "system" fn CShellLink_IPersistFile_GetCurFile(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistFile_iface)) as *mut CShellLink;
    CShellLink_GetCurFile_body(this, ppszFileName)
}

pub static CSHELLLINK_IPERSISTFILE_VTBL: IPersistFileVtbl = IPersistFileVtbl {
    QueryInterface: CShellLink_IPersistFile_QueryInterface,
    AddRef: CShellLink_IPersistFile_AddRef,
    Release: CShellLink_IPersistFile_Release,
    GetClassID: CShellLink_IPersistFile_GetClassID,
    IsDirty: CShellLink_IPersistFile_IsDirty,
    Load: CShellLink_IPersistFile_Load,
    Save: CShellLink_IPersistFile_Save,
    SaveCompleted: CShellLink_IPersistFile_SaveCompleted,
    GetCurFile: CShellLink_IPersistFile_GetCurFile,
};

unsafe extern "system" fn CShellLink_IPersistStream_QueryInterface(This: *mut IPersistStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IPersistStream_AddRef(This: *mut IPersistStream) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IPersistStream_Release(This: *mut IPersistStream) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IPersistStream_GetClassID(This: *mut IPersistStream, pClassID: *mut CLSID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CShellLink_IPersistStream_IsDirty(This: *mut IPersistStream) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_IsDirty_body(this)
}
unsafe extern "system" fn CShellLink_IPersistStream_Load(This: *mut IPersistStream, pStm: *mut IStream) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_Load_v2_body(this, pStm)
}
unsafe extern "system" fn CShellLink_IPersistStream_Save(This: *mut IPersistStream, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_Save_v2_body(this, pStm, fClearDirty)
}
unsafe extern "system" fn CShellLink_IPersistStream_GetSizeMax(This: *mut IPersistStream, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IPersistStream_iface)) as *mut CShellLink;
    CShellLink_GetSizeMax_body(this, pcbSize)
}

pub static CSHELLLINK_IPERSISTSTREAM_VTBL: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: CShellLink_IPersistStream_QueryInterface,
    AddRef: CShellLink_IPersistStream_AddRef,
    Release: CShellLink_IPersistStream_Release,
    GetClassID: CShellLink_IPersistStream_GetClassID,
    IsDirty: CShellLink_IPersistStream_IsDirty,
    Load: CShellLink_IPersistStream_Load,
    Save: CShellLink_IPersistStream_Save,
    GetSizeMax: CShellLink_IPersistStream_GetSizeMax,
};

unsafe extern "system" fn CShellLink_IShellExtInit_QueryInterface(This: *mut IShellExtInit, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellExtInit_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IShellExtInit_AddRef(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellExtInit_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IShellExtInit_Release(This: *mut IShellExtInit) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellExtInit_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IShellExtInit_Initialize(This: *mut IShellExtInit, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellExtInit_iface)) as *mut CShellLink;
    CShellLink_Initialize_body(this, pidlFolder, pdtobj, hkeyProgID)
}

pub static CSHELLLINK_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: CShellLink_IShellExtInit_QueryInterface,
    AddRef: CShellLink_IShellExtInit_AddRef,
    Release: CShellLink_IShellExtInit_Release,
    Initialize: CShellLink_IShellExtInit_Initialize,
};

unsafe extern "system" fn CShellLink_IContextMenu_QueryInterface(This: *mut IContextMenu, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IContextMenu_AddRef(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IContextMenu_Release(This: *mut IContextMenu) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IContextMenu_QueryContextMenu(This: *mut IContextMenu, hmenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_QueryContextMenu_body(this, hmenu, indexMenu, idCmdFirst, idCmdLast, uFlags)
}
unsafe extern "system" fn CShellLink_IContextMenu_InvokeCommand(This: *mut IContextMenu, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_InvokeCommand_body(this, lpici)
}
unsafe extern "system" fn CShellLink_IContextMenu_GetCommandString(This: *mut IContextMenu, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IContextMenu_iface)) as *mut CShellLink;
    CShellLink_GetCommandString_body(this, idCmd, uType, pwReserved, pszName, cchMax)
}

pub static CSHELLLINK_ICONTEXTMENU_VTBL: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: CShellLink_IContextMenu_QueryInterface,
    AddRef: CShellLink_IContextMenu_AddRef,
    Release: CShellLink_IContextMenu_Release,
    QueryContextMenu: CShellLink_IContextMenu_QueryContextMenu,
    InvokeCommand: CShellLink_IContextMenu_InvokeCommand,
    GetCommandString: CShellLink_IContextMenu_GetCommandString,
};

unsafe extern "system" fn CShellLink_IDropTarget_QueryInterface(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IDropTarget_AddRef(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IDropTarget_Release(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IDropTarget_DragEnter(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_DragEnter_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CShellLink_IDropTarget_DragOver(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_DragOver_body(this, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CShellLink_IDropTarget_DragLeave(This: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_DragLeave_body(this)
}
unsafe extern "system" fn CShellLink_IDropTarget_Drop(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IDropTarget_iface)) as *mut CShellLink;
    CShellLink_Drop_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}

pub static CSHELLLINK_IDROPTARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: CShellLink_IDropTarget_QueryInterface,
    AddRef: CShellLink_IDropTarget_AddRef,
    Release: CShellLink_IDropTarget_Release,
    DragEnter: CShellLink_IDropTarget_DragEnter,
    DragOver: CShellLink_IDropTarget_DragOver,
    DragLeave: CShellLink_IDropTarget_DragLeave,
    Drop: CShellLink_IDropTarget_Drop,
};

unsafe extern "system" fn CShellLink_IShellLinkDataList_QueryInterface(This: *mut IShellLinkDataList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_AddRef(This: *mut IShellLinkDataList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_Release(This: *mut IShellLinkDataList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_AddDataBlock(This: *mut IShellLinkDataList, pDataBlock: *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_AddDataBlock_body(this, pDataBlock)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_CopyDataBlock(This: *mut IShellLinkDataList, dwSig: DWORD, ppDataBlock: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_CopyDataBlock_body(this, dwSig, ppDataBlock)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_RemoveDataBlock(This: *mut IShellLinkDataList, dwSig: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_RemoveDataBlock_body(this, dwSig)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_GetFlags(This: *mut IShellLinkDataList, pdwFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_GetFlags_body(this, pdwFlags)
}
unsafe extern "system" fn CShellLink_IShellLinkDataList_SetFlags(This: *mut IShellLinkDataList, dwFlags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellLinkDataList_iface)) as *mut CShellLink;
    CShellLink_SetFlags_body(this, dwFlags)
}

pub static CSHELLLINK_ISHELLLINKDATALIST_VTBL: IShellLinkDataListVtbl = IShellLinkDataListVtbl {
    QueryInterface: CShellLink_IShellLinkDataList_QueryInterface,
    AddRef: CShellLink_IShellLinkDataList_AddRef,
    Release: CShellLink_IShellLinkDataList_Release,
    AddDataBlock: CShellLink_IShellLinkDataList_AddDataBlock,
    CopyDataBlock: CShellLink_IShellLinkDataList_CopyDataBlock,
    RemoveDataBlock: CShellLink_IShellLinkDataList_RemoveDataBlock,
    GetFlags: CShellLink_IShellLinkDataList_GetFlags,
    SetFlags: CShellLink_IShellLinkDataList_SetFlags,
};

unsafe extern "system" fn CShellLink_IExtractIconA_QueryInterface(This: *mut IExtractIconA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconA_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IExtractIconA_AddRef(This: *mut IExtractIconA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconA_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IExtractIconA_Release(This: *mut IExtractIconA) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconA_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IExtractIconA_GetIconLocation(This: *mut IExtractIconA, uFlags: UINT, szIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconA_iface)) as *mut CShellLink;
    CShellLink_GetIconLocation_v3_body(this, uFlags, szIconFile, cchMax, piIndex, pwFlags)
}
unsafe extern "system" fn CShellLink_IExtractIconA_Extract(This: *mut IExtractIconA, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconA_iface)) as *mut CShellLink;
    CShellLink_Extract_v1_body(this, pszFile, nIconIndex, phiconLarge, phiconSmall, nIconSize)
}

pub static CSHELLLINK_IEXTRACTICONA_VTBL: IExtractIconAVtbl = IExtractIconAVtbl {
    QueryInterface: CShellLink_IExtractIconA_QueryInterface,
    AddRef: CShellLink_IExtractIconA_AddRef,
    Release: CShellLink_IExtractIconA_Release,
    GetIconLocation: CShellLink_IExtractIconA_GetIconLocation,
    Extract: CShellLink_IExtractIconA_Extract,
};

unsafe extern "system" fn CShellLink_IExtractIconW_QueryInterface(This: *mut IExtractIconW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconW_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IExtractIconW_AddRef(This: *mut IExtractIconW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconW_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IExtractIconW_Release(This: *mut IExtractIconW) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconW_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IExtractIconW_GetIconLocation(This: *mut IExtractIconW, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconW_iface)) as *mut CShellLink;
    CShellLink_GetIconLocation_v4_body(this, uFlags, szIconFile, cchMax, piIndex, pwFlags)
}
unsafe extern "system" fn CShellLink_IExtractIconW_Extract(This: *mut IExtractIconW, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IExtractIconW_iface)) as *mut CShellLink;
    CShellLink_Extract_v2_body(this, pszFile, nIconIndex, phiconLarge, phiconSmall, nIconSize)
}

pub static CSHELLLINK_IEXTRACTICONW_VTBL: IExtractIconWVtbl = IExtractIconWVtbl {
    QueryInterface: CShellLink_IExtractIconW_QueryInterface,
    AddRef: CShellLink_IExtractIconW_AddRef,
    Release: CShellLink_IExtractIconW_Release,
    GetIconLocation: CShellLink_IExtractIconW_GetIconLocation,
    Extract: CShellLink_IExtractIconW_Extract,
};

unsafe extern "system" fn CShellLink_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IObjectWithSite_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IObjectWithSite_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IObjectWithSite_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IObjectWithSite_iface)) as *mut CShellLink;
    CShellLink_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CShellLink_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IObjectWithSite_iface)) as *mut CShellLink;
    CShellLink_GetSite_body(this, riid, ppvSite)
}

pub static CSHELLLINK_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CShellLink_IObjectWithSite_QueryInterface,
    AddRef: CShellLink_IObjectWithSite_AddRef,
    Release: CShellLink_IObjectWithSite_Release,
    SetSite: CShellLink_IObjectWithSite_SetSite,
    GetSite: CShellLink_IObjectWithSite_GetSite,
};

unsafe extern "system" fn CShellLink_IShellPropSheetExt_QueryInterface(This: *mut IShellPropSheetExt, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellPropSheetExt_iface)) as *mut CShellLink;
    CShellLink_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellLink_IShellPropSheetExt_AddRef(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellPropSheetExt_iface)) as *mut CShellLink;
    CShellLink_AddRef(this)
}
unsafe extern "system" fn CShellLink_IShellPropSheetExt_Release(This: *mut IShellPropSheetExt) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellPropSheetExt_iface)) as *mut CShellLink;
    CShellLink_Release(this)
}
unsafe extern "system" fn CShellLink_IShellPropSheetExt_AddPages(This: *mut IShellPropSheetExt, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellPropSheetExt_iface)) as *mut CShellLink;
    CShellLink_AddPages_body(this, pfnAddPage, lParam)
}
unsafe extern "system" fn CShellLink_IShellPropSheetExt_ReplacePage(This: *mut IShellPropSheetExt, uPageID: EXPPS, pfnReplaceWith: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellLink, IShellPropSheetExt_iface)) as *mut CShellLink;
    CShellLink_ReplacePage_body(this, uPageID, pfnReplaceWith, lParam)
}

pub static CSHELLLINK_ISHELLPROPSHEETEXT_VTBL: IShellPropSheetExtVtbl = IShellPropSheetExtVtbl {
    QueryInterface: CShellLink_IShellPropSheetExt_QueryInterface,
    AddRef: CShellLink_IShellPropSheetExt_AddRef,
    Release: CShellLink_IShellPropSheetExt_Release,
    AddPages: CShellLink_IShellPropSheetExt_AddPages,
    ReplacePage: CShellLink_IShellPropSheetExt_ReplacePage,
};

unsafe fn CShellLink_QueryInterface(
    this: *mut CShellLink, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellLinkA)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellLinkA_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellLinkW)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellLinkW_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersist)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersist_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistFile)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFile_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistStream)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistStream_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellExtInit_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IContextMenu)) {
        *ppv = core::ptr::addr_of_mut!((*this).IContextMenu_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellLinkDataList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellLinkDataList_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconA)) {
        *ppv = core::ptr::addr_of_mut!((*this).IExtractIconA_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IExtractIconW)) {
        *ppv = core::ptr::addr_of_mut!((*this).IExtractIconW_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellPropSheetExt)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellPropSheetExt_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CShellLink_AddRef(this);
    S_OK
}

unsafe fn CShellLink_AddRef(this: *mut CShellLink) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CShellLink_Release(this: *mut CShellLink) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CShellLink_dtor_body(this);
        CShellLink_cpp_com_assign_m_site(this, core::ptr::null_mut());
        CShellLink_cpp_com_assign_m_DropTarget(this, core::ptr::null_mut());
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

// Activation: CLSID_ShellLink currently maps to CShellLink in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CShellLink_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CShellLink>()) as *mut CShellLink;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CShellLink {
        IShellLinkA_iface: IShellLinkA { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ISHELLLINKA_VTBL) },
        IShellLinkW_iface: IShellLinkW { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ISHELLLINKW_VTBL) },
        IPersist_iface: IPersist { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IPERSIST_VTBL) },
        IPersistFile_iface: IPersistFile { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IPERSISTFILE_VTBL) },
        IPersistStream_iface: IPersistStream { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IPERSISTSTREAM_VTBL) },
        IShellExtInit_iface: IShellExtInit { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ISHELLEXTINIT_VTBL) },
        IContextMenu_iface: IContextMenu { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ICONTEXTMENU_VTBL) },
        IDropTarget_iface: IDropTarget { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IDROPTARGET_VTBL) },
        IShellLinkDataList_iface: IShellLinkDataList { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ISHELLLINKDATALIST_VTBL) },
        IExtractIconA_iface: IExtractIconA { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IEXTRACTICONA_VTBL) },
        IExtractIconW_iface: IExtractIconW { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IEXTRACTICONW_VTBL) },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CSHELLLINK_IOBJECTWITHSITE_VTBL) },
        IShellPropSheetExt_iface: IShellPropSheetExt { lpVtbl: core::ptr::addr_of!(CSHELLLINK_ISHELLPROPSHEETEXT_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_Header: core::mem::zeroed(),
        m_pPidl: core::mem::zeroed(),
        m_sPath: core::mem::zeroed(),
        volume: core::mem::zeroed(),
        m_sDescription: core::mem::zeroed(),
        m_sPathRel: core::mem::zeroed(),
        m_sWorkDir: core::mem::zeroed(),
        m_sArgs: core::mem::zeroed(),
        m_sIcoPath: core::mem::zeroed(),
        m_bRunAs: core::mem::zeroed(),
        m_bDirty: core::mem::zeroed(),
        m_pDBList: core::mem::zeroed(),
        m_bInInit: core::mem::zeroed(),
        m_hIcon: core::mem::zeroed(),
        sProduct: core::mem::zeroed(),
        sComponent: core::mem::zeroed(),
        m_sLinkPath: core::mem::zeroed(),
        m_site: core::mem::zeroed(),
        m_DropTarget: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CShellLink_ctor_body(this);
    let hr = CShellLink_QueryInterface(this, riid, ppv);
    CShellLink_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

#[repr(C, packed)]
pub struct LOCAL_VOLUME_INFO {
    pub dwSize: DWORD,
    pub dwType: DWORD,
    pub dwVolSerial: DWORD,
    pub dwVolLabelOfs: DWORD,
}

#[repr(C, packed)]
pub struct LOCATION_INFO {
    pub dwTotalSize: DWORD,
    pub dwHeaderSize: DWORD,
    pub dwFlags: DWORD,
    pub dwVolTableOfs: DWORD,
    pub dwLocalPathOfs: DWORD,
    pub dwNetworkVolTableOfs: DWORD,
    pub dwFinalPathOfs: DWORD,
}

// ---- Constants: every value verified against the cited in-tree header ----
// dll/win32/shell32/shresdef.h:502-523 (control IDs), :443/:449 (dialogs),
// :623 (IDI), :943 (IDR), :212/:246/:261-263 (strings)
pub const IDC_SHORTCUT_ADVANCED: INT = 14022;
pub const IDC_SHORTCUT_CHANGE_ICON: INT = 14021;
pub const IDC_SHORTCUT_COMMENT_EDIT: INT = 14019;
pub const IDC_SHORTCUT_FIND: INT = 14020;
pub const IDC_SHORTCUT_ICON: INT = 14000;
pub const IDC_SHORTCUT_KEY_HOTKEY: INT = 14015;
pub const IDC_SHORTCUT_LOCATION_EDIT: INT = 14007;
pub const IDC_SHORTCUT_RUN_COMBO: INT = 14017;
pub const IDC_SHORTCUT_START_IN_EDIT: INT = 14011;
pub const IDC_SHORTCUT_TARGET_TEXT: INT = 14009;
pub const IDC_SHORTCUT_TEXT: INT = 14001;
pub const IDC_SHORTCUT_TYPE_EDIT: INT = 14005;
pub const IDC_SHORTEX_RUN_DIFFERENT: INT = 14000;
pub const IDD_SHORTCUT_EXTENDED_PROPERTIES: UINT = 16;
pub const IDD_SHORTCUT_PROPERTIES: UINT = 14;
pub const IDI_SHELL_SHORTCUT: INT = 30;
pub const IDR_SHELLLINK: UINT = 144;
pub const IDS_OPENFILELOCATION: UINT = 341;
pub const IDS_OPEN_VERB: UINT = 300;
pub const IDS_SHORTCUT_RUN_MAX: UINT = 4169;
pub const IDS_SHORTCUT_RUN_MIN: UINT = 4168;
pub const IDS_SHORTCUT_RUN_NORMAL: UINT = 4167;
// dll/win32/shell32/CShellLink.cpp:154-155
pub const SHLINK_LOCAL: UINT = 0;
pub const SHLINK_REMOTE: UINT = 1;
// dll/win32/shell32/CShellLink.h:62-66 (enum IDCMD)
pub const IDCMD_OPEN: UINT = 0;
pub const IDCMD_OPENFILELOCATION: UINT = 1;
// winuser.h show commands
pub const SW_SHOWNORMAL: INT = 1;
pub const SW_SHOWMAXIMIZED: INT = 3;
pub const SW_SHOW: INT = 5;
pub const SW_SHOWMINNOACTIVE: INT = 7;
// winuser.h button/combobox/edit messages + notifications
pub const BM_GETCHECK: UINT = 0x00F0;
pub const BM_SETCHECK: UINT = 0x00F1;
pub const BST_UNCHECKED: LRESULT = 0;
pub const BST_CHECKED: LRESULT = 1;
pub const CBN_SELCHANGE: UINT = 1;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CB_GETITEMDATA: UINT = 0x0150;
pub const CB_SETCURSEL: UINT = 0x014E;
pub const CB_SETITEMDATA: UINT = 0x0151;
pub const CB_ERR: INT = -1;
pub const EM_SETSEL: UINT = 0x00B1;
pub const EN_CHANGE: UINT = 0x0300;
pub const STM_SETICON: UINT = 0x0170;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_INITDIALOG: UINT = 0x0110;
pub const WM_NOTIFY: UINT = 0x004E;
// commctrl.h hotkey control messages (WM_USER=0x400 +1/+2)
pub const HKM_SETHOTKEY: UINT = 0x0401;
pub const HKM_GETHOTKEY: UINT = 0x0402;
// winuser.h dialog item results / misc
pub const IDOK: UINT = 1;
pub const IDCANCEL: UINT = 2;
pub const MB_ICONERROR: UINT = 16;
pub const COLOR_3DFACE: INT = 15;
pub const VK_CONTROL: INT = 0x11;
pub const SM_CXICON: INT = 11;
pub const SM_CYICON: INT = 12;
pub const IMAGE_ICON: UINT = 1;
pub const DI_NORMAL: UINT = 3;
// winuser.h: DWLP_MSGRESULT=0; DWLP_USER=DWLP_DLGPROC+sizeof(DLGPROC);
// DWLP_DLGPROC=DWLP_MSGRESULT+sizeof(LRESULT) => 2 pointers
pub const DWLP_MSGRESULT: INT = 0;
pub const DWLP_USER: INT = (2 * core::mem::size_of::<LONG_PTR>()) as INT;
// winnls.h: MAKELCID(MAKELANGID(LANG_NEUTRAL,SUBLANG_DEFAULT),SORT_DEFAULT)
pub const LOCALE_USER_DEFAULT: LCID = 0x0400;
pub const DATE_SHORTDATE: DWORD = 1;
// winuser.h menu-item flags
pub const MIIM_STATE: UINT = 1;
pub const MFS_ENABLED: UINT = 0;
pub const MFS_DEFAULT: UINT = 4096;
// wingdi.h
pub const DIB_RGB_COLORS: UINT = 0;
// commctrl.h image list
pub const ILC_MASK: UINT = 0x1;
pub const ILC_COLOR32: UINT = 0x20;
pub const ILD_NORMAL: UINT = 0x0;
pub const ILD_TRANSPARENT: UINT = 0x1;
// prsht.h: PSPCB_RELEASE (:37); PSN_FIRST (0U-200U), PSN_APPLY = PSN_FIRST-2 (:114-117)
pub const PSPCB_RELEASE: UINT = 1;
pub const PSN_APPLY: UINT = 0u32.wrapping_sub(202);
pub const PSNRET_NOERROR: LONG_PTR = 0;
pub const PSNRET_INVALID_NOCHANGEPAGE: LONG_PTR = 2;
// winerror.h
pub const NOERROR: HRESULT = 0;
pub const STG_E_INVALIDPOINTER: HRESULT = 0x80030009u32 as HRESULT;
pub const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
pub const ERROR_SUCCESS_DW: DWORD = 0; // winerror.h ERROR_SUCCESS, DWORD-typed use
// fileapi.h
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;
// winbase.h
pub const LMEM_ZEROINIT: UINT = 0x0040;
// winnt.h
pub const MAXUSHORT: usize = 0xffff;
// winuser.h clipboard formats
pub const CF_HDROP: CLIPFORMAT = 15;
// objidl.h / oleidl.h / wtypes.h
pub const DVASPECT_CONTENT: DWORD = 1;
pub const TYMED_HGLOBAL: DWORD = 1;
pub const DROPEFFECT_NONE: DWORD = 0;
// objbase.h STGM flags
pub const STGM_READ: DWORD = 0x00000000;
pub const STGM_READWRITE: DWORD = 0x00000002;
pub const STGM_SHARE_EXCLUSIVE: DWORD = 0x00000010;
pub const STGM_SHARE_DENY_WRITE: DWORD = 0x00000020;
pub const STGM_CREATE: DWORD = 0x00001000;
// shellapi.h SEE_MASK_* (:30-55) + reactos/undocshell.h (:692-695)
pub const SEE_MASK_ICON: DWORD = 0x00000010;
pub const SEE_MASK_HOTKEY: DWORD = 0x00000020;
pub const SEE_MASK_NOASYNC: DWORD = 0x00000100;
pub const SEE_MASK_DOENVSUBST: DWORD = 0x00000200;
pub const SEE_MASK_FLAG_NO_UI: DWORD = 0x00000400;
pub const SEE_MASK_INVOKEIDLIST: DWORD = 0x0000000C;
pub const SEE_MASK_UNICODE: DWORD = 0x00004000;
pub const SEE_MASK_NO_CONSOLE: DWORD = 0x00008000;
pub const SEE_MASK_ASYNCOK: DWORD = 0x00100000;
pub const SEE_MASK_NOZONECHECKS: DWORD = 0x00800000;
pub const SEE_MASK_FLAG_LOG_USAGE: DWORD = 0x04000000;
pub const SEE_MASK_HASLINKNAME: DWORD = 0x00010000;
pub const SEE_MASK_FLAG_SEPVDM: DWORD = 0x00020000;
pub const SEE_MASK_HASTITLE: DWORD = 0x00080000;
// shobjidl.idl:1698-1701 CMIC_MASK_* aliases of SEE_MASK_*
pub const CMIC_MASK_HOTKEY: DWORD = SEE_MASK_HOTKEY;
pub const CMIC_MASK_FLAG_NO_UI: DWORD = SEE_MASK_FLAG_NO_UI;
pub const CMIC_MASK_UNICODE: DWORD = SEE_MASK_UNICODE;
// reactos/shellutils.h:683-687 SEE_CMIC_COMMON_FLAGS
pub const SEE_CMIC_COMMON_BASICFLAGS: DWORD = SEE_MASK_NOASYNC | SEE_MASK_ASYNCOK
    | SEE_MASK_UNICODE | SEE_MASK_NO_CONSOLE | SEE_MASK_FLAG_NO_UI
    | SEE_MASK_FLAG_SEPVDM | SEE_MASK_FLAG_LOG_USAGE | SEE_MASK_NOZONECHECKS;
pub const SEE_CMIC_COMMON_FLAGS: DWORD = SEE_CMIC_COMMON_BASICFLAGS | SEE_MASK_HOTKEY
    | SEE_MASK_ICON | SEE_MASK_HASLINKNAME | SEE_MASK_HASTITLE;
// shlwapi.h SHACF
pub const SHACF_DEFAULT: DWORD = 0x00000000;
// shlobj.h:2431-2433 PathResolve flags
pub const PRF_VERIFYEXISTS: DWORD = 0x01;
pub const PRF_TRYPROGRAMEXTENSIONS: DWORD = 0x02 | PRF_VERIFYEXISTS;
// shellapi.h SHChangeNotify events / flags
pub const SHCNE_CREATE: LONG = 0x00000002;
pub const SHCNE_UPDATEITEM: LONG = 0x00002000;
pub const SHCNF_PATHW: UINT = 0x0005;
// shellapi.h SHGetFileInfo flags
pub const SHGFI_ICON: UINT = 0x000000100;
pub const SHGFI_DISPLAYNAME: UINT = 0x000000200;
pub const SHGFI_TYPENAME: UINT = 0x000000400;
pub const SHGFI_ICONLOCATION: UINT = 0x000001000;
pub const SHGFI_PIDL: UINT = 0x000000008;
pub const SHGFI_USEFILEATTRIBUTES: UINT = 0x000000010;
// shobjidl.idl:1140-1155 SLR_* / SLGP_*
pub const SLR_NO_UI: DWORD = 0x0001;
pub const SLR_NOUPDATE: DWORD = 0x0008;
pub const SLR_NOSEARCH: DWORD = 0x0010;
pub const SLR_NOTRACK: DWORD = 0x0020;
pub const SLGP_SHORTPATH: DWORD = 0x0001;
pub const SLGP_UNCPRIORITY: DWORD = 0x0002;
pub const SLGP_RAWPATH: DWORD = 0x0004;
// shobjidl.idl SHGDN_FORADDRESSBAR (shellcom.rs:114-115 has NORMAL/FORPARSING)
pub const SHGDN_FORADDRESSBAR: SHGDNF = 0x4000;
// shobjidl.idl:1357-1363 GIL_*
pub const GIL_DEFAULTICON: UINT = 0x0040;
pub const GIL_FORSHORTCUT: UINT = 0x0080;
pub const GIL_PERCLASS: UINT = 0x0004;
pub const GIL_NOTFILENAME: UINT = 0x0008;
// psdk/shlobj.h:2057-2065 extra data block signatures
pub const EXP_SZ_LINK_SIG: DWORD = 0xA0000001;
pub const EXP_SPECIAL_FOLDER_SIG: DWORD = 0xA0000005;
pub const EXP_DARWIN_ID_SIG: DWORD = 0xA0000006;
pub const EXP_LOGO3_ID_SIG: DWORD = 0xA0000007;
pub const EXP_SZ_ICON_SIG: DWORD = 0xA0000007;
// shlobj.h SHELL_LINK_DATA_FLAGS
pub const SLDF_HAS_ID_LIST: DWORD = 0x00000001;
pub const SLDF_HAS_LINK_INFO: DWORD = 0x00000002;
pub const SLDF_HAS_NAME: DWORD = 0x00000004;
pub const SLDF_HAS_RELPATH: DWORD = 0x00000008;
pub const SLDF_HAS_WORKINGDIR: DWORD = 0x00000010;
pub const SLDF_HAS_ARGS: DWORD = 0x00000020;
pub const SLDF_HAS_ICONLOCATION: DWORD = 0x00000040;
pub const SLDF_UNICODE: DWORD = 0x00000080;
pub const SLDF_FORCE_NO_LINKINFO: DWORD = 0x00000100;
pub const SLDF_HAS_EXP_SZ: DWORD = 0x00000200;
pub const SLDF_RUN_IN_SEPARATE: DWORD = 0x00000400;
pub const SLDF_HAS_LOGO3ID: DWORD = 0x00000800;
pub const SLDF_HAS_DARWINID: DWORD = 0x00001000;
pub const SLDF_RUNAS_USER: DWORD = 0x00002000;
pub const SLDF_HAS_EXP_ICON_SZ: DWORD = 0x00004000;
pub const SLDF_RUN_WITH_SHIMLAYER: DWORD = 0x00020000;
pub const SLDF_ALLOW_LINK_TO_LINK: DWORD = 0x00800000;
// sdkddkver.h:157 + root CMakeLists.txt:250-252 (_WIN32_WINNT=0x502 => NTDDI=WS03).
// CShellLink.cpp has eight `#if (NTDDI_VERSION < NTDDI_LONGHORN)` blocks, all Logo3 (the
// EXP_LOGO3_ID_SIG advertised-product block). They are EXCLUDED from the real build: shell32's
// CMakeLists replaces the tree's -D_WIN32_WINNT=0x502 with 0x600, and sdkddkver.h derives
// NTDDI_VERSION from it, so NTDDI_VERSION == NTDDI_LONGHORN == 0x06000000 and `<` is false —
// confirmed with the build's own compiler on a probe (`BLOCK_IS_EXCLUDED`). This file used to
// state the opposite and carried NTDDI_VERSION = 0x05020000 to match; four of the eight blocks
// had been translated as live code on that basis. Every Logo3 path below is therefore absent
// on purpose — each site says so where it would have been.

pub use crate::island::BITMAPINFO;

pub use crate::island::BITMAPINFOHEADER;

pub use crate::island::DATABLOCK_HEADER;
pub type LPDBLIST = *mut DATABLOCK_HEADER;

pub use crate::island::EXP_DARWIN_LINK;
pub type LPEXP_DARWIN_LINK = *mut EXP_DARWIN_LINK;

pub use crate::island::EXP_SPECIAL_FOLDER;
pub type LPEXP_SPECIAL_FOLDER = *mut EXP_SPECIAL_FOLDER;

pub use crate::island::EXP_SZ_LINK;
pub type LPEXP_SZ_LINK = *mut EXP_SZ_LINK;

pub use crate::island::POINTL;

pub use crate::island::RECT;

pub use crate::island::RGBQUAD;

pub use crate::island::SYSTEMTIME;

pub use crate::island::WIN32_FIND_DATAA;

pub use crate::island::WIN32_FIND_DATAW;

pub type EXPPS = UINT;

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

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
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
    fn SHFree(memory: LPVOID);
}

extern "C" {
    #[link_name = "memcpy"]
    fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "memset"]
    fn cpp_com_memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
}
#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}
#[inline]
unsafe fn memset<D, N: CppComSize>(dst: *mut D, value: INT, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memset(dst.cast(), value, bytes.cpp_com_size())
}
#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) {
    let _ = cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size());
}
#[inline]
unsafe fn ZeroMemory<D, N: CppComSize>(dst: *mut D, bytes: N) {
    let _ = cpp_com_memset(dst.cast(), 0, bytes.cpp_com_size());
}

#[inline]
unsafe fn StringCchCopyW<N: CppComSize>(dst: LPWSTR, capacity: N, src: LPCWSTR) -> HRESULT {
    let capacity = capacity.cpp_com_size();
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
}

extern "system" {
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            shell32_hInstance, id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), (bytes) as SIZE_T)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

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
pub static CPP_COM_WSTR_08CDC276CF8F: [WCHAR; 11] = [0x002F, 0x0073, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x002C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_3390E943BCAB: [WCHAR; 39] = [0x0059, 0x006F, 0x0075, 0x0020, 0x0063, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0063, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0020, 0x0061, 0x0020, 0x006C, 0x0069, 0x006E, 0x006B, 0x0020, 0x0074, 0x006F, 0x0020, 0x0061, 0x0020, 0x0073, 0x0068, 0x006F, 0x0072, 0x0074, 0x0063, 0x0075, 0x0074, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_966A151B2AF3: [WCHAR; 53] = [0x0054, 0x0068, 0x0065, 0x0020, 0x0073, 0x0070, 0x0065, 0x0063, 0x0069, 0x0066, 0x0069, 0x0065, 0x0064, 0x0020, 0x0066, 0x0069, 0x006C, 0x0065, 0x0020, 0x006E, 0x0061, 0x006D, 0x0065, 0x0020, 0x0069, 0x006E, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0074, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0020, 0x0062, 0x006F, 0x0078, 0x0020, 0x0069, 0x0073, 0x0020, 0x0069, 0x006E, 0x0076, 0x0061, 0x006C, 0x0069, 0x0064, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9F7F17012B9B: [WCHAR; 6] = [0x0022, 0x0025, 0x006C, 0x0073, 0x0022, 0x0000];
pub static CPP_COM_WSTR_AAA8350805D2: [WCHAR; 7] = [0x0078, 0x003A, 0x005C, 0x0078, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_AB68B18B313D: [WCHAR; 13] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_AF854C3D2274: [WCHAR; 8] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000];
pub static CPP_COM_WSTR_DF3F619804A9: [WCHAR; 2] = [0x0000, 0x0000];
pub static CPP_COM_WSTR_FB47CCC40FE6: [WCHAR; 3] = [0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;
// The crate's own functions: the definition is the declaration.
use crate::shell32::iconcache::ExtractIconExW;
use crate::wine::classes::HLM_GetIconW;
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILFree;
use crate::wine::pidl::ILGetDisplayNameExW;
use crate::wine::pidl::ILGetSize;
use crate::dialogs_dialogs::PickIconDlg;
use crate::wine::shellole::SHAlloc;
use crate::wine::pidl::SHCloneSpecialIDList;
use crate::shell32::utils::SHELL_GetUIObjectOfAbsoluteItem;
use crate::wine::shell32_main::SHGetFileInfoW;
use crate::wine::pidl::SHGetPathFromIDListW;
use crate::wine::pidl::SHILCreateFromPathW;
use crate::propsheet::SH_CreatePropertySheetPageEx;
use crate::shell32::shlexec::ShellExecuteExW;
use crate::shell32::shlexec::ShellExecuteW;
// One declaration per external symbol: island.rs.
use crate::island::CLSIDFromString;
use crate::island::CoTaskMemAlloc;
use crate::island::CreateCompatibleDC;
use crate::island::CreateDIBSection;
use crate::island::CreateSolidBrush;
use crate::island::DeleteDC;
use crate::island::DestroyIcon;
use crate::island::DestroyPropertySheetPage;
use crate::island::DialogBoxParamW;
use crate::island::DrawIconEx;
use crate::island::EnableWindow;
use crate::island::EndDialog;
use crate::island::FillRect;
use crate::island::FindFirstFileW;
use crate::island::GetDlgItem;
use crate::island::GetDlgItemTextW;
use crate::island::GetModuleHandleW;
use crate::island::GetParent;
use crate::island::GetSystemMetrics;
use crate::island::GetWindowTextW;
use crate::island::HeapReAlloc;
use crate::island::ImageList_AddMasked;
use crate::island::ImageList_Create;
use crate::island::ImageList_Destroy;
use crate::island::ImageList_GetIcon;
use crate::island::InsertMenuItemW;
use crate::island::IsWindowEnabled;
use crate::island::LoadImageW;
use crate::island::LoadStringW;
use crate::island::LocalAlloc;
use crate::island::MessageBoxW;
use crate::island::SHAddDataBlock;
use crate::island::SHAutoComplete;
use crate::island::SHCreateStreamOnFileW;
use crate::island::SHFindDataBlock;
use crate::island::SHFreeDataBlockList;
use crate::island::SHReadDataBlockList;
use crate::island::SHRemoveDataBlock;
use crate::island::SHWriteDataBlockList;
use crate::island::SelectObject;
use crate::island::SendDlgItemMessageW;
use crate::island::SendMessageW;
use crate::island::SetDlgItemTextW;
use core::mem::size_of;
use crate::shellcom::*;
use crate::types::*;

macro_rules! ASSERT { ($($t:tt)*) => {{}} } // DBG-only ASSERT (debug.h), inert in release

pub unsafe fn HEAP_strdupAtoW_body(heap: HANDLE, flags: DWORD, str: LPCSTR) -> LPWSTR {
    // assert(str): C assert — inert outside DBG, matching release semantics.
    let len: i32 = MultiByteToWideChar(
        CP_ACP,
        0,
        str,
        -1,
        core::ptr::null_mut(),
        0,
    );
    let p: LPWSTR = HeapAlloc(
        heap,
        flags,
        len as usize * core::mem::size_of::<u16>(),
    ) as LPWSTR;
    if p.is_null() {
        return p;
    }
    MultiByteToWideChar(
        CP_ACP,
        0,
        str,
        -1,
        p,
        len,
    );
    p
}

pub unsafe fn strdupW_body(src: LPCWSTR) -> LPWSTR {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    let dest = HeapAlloc(
        GetProcessHeap(),
        0,
        (wcslen(src) + 1)
            .wrapping_mul(core::mem::size_of::<u16>()),
    ) as LPWSTR;
    if !dest.is_null() {
        wcscpy(dest, src);
    }
    dest
}

pub unsafe fn PathEnvSubstIsDirectory_body(pszPath: LPCWSTR) -> BOOL {
    let mut szStack: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let stack_len = szStack.len() as u32;
    let cch = ExpandEnvironmentStringsW(pszPath, szStack.as_mut_ptr(), stack_len);
    if cch <= stack_len {
        return if cch != 0 { PathIsDirectoryW(szStack.as_ptr()) } else { 0 };
    }
    let szHeap = SHAlloc(cch as SIZE_T) as *mut u16;
    if szHeap.is_null() {
        return FALSE;
    }
    let bResult = (ExpandEnvironmentStringsW(pszPath, szHeap, cch) != 0) && (PathIsDirectoryW(szHeap) != 0);
    SHFree(szHeap as *mut _);
    if bResult { 1 } else { 0 }
}

pub unsafe fn CShellLink_Reset_body(this: *mut CShellLink) -> HRESULT {
    ILFree((*this).m_pPidl);
    (*this).m_pPidl = core::ptr::null_mut();

    HeapFree(GetProcessHeap(), 0, (*this).m_sPath);
    (*this).m_sPath = core::ptr::null_mut();
    ZeroMemory(
        &raw mut (*this).volume as *mut _,
        ::core::mem::size_of_val(&(*this).volume),
    );

    HeapFree(GetProcessHeap(), 0, (*this).m_sDescription);
    (*this).m_sDescription = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*this).m_sPathRel);
    (*this).m_sPathRel = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*this).m_sWorkDir);
    (*this).m_sWorkDir = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*this).m_sArgs);
    (*this).m_sArgs = core::ptr::null_mut();
    HeapFree(GetProcessHeap(), 0, (*this).m_sIcoPath);
    (*this).m_sIcoPath = core::ptr::null_mut();

    (*this).m_bRunAs = FALSE;
    (*this).m_bDirty = FALSE;

    if !(*this).m_pDBList.is_null() {
        SHFreeDataBlockList((*this).m_pDBList);
    }
    (*this).m_pDBList = core::ptr::null_mut();

    (*this).sProduct = core::ptr::null_mut();
    (*this).sComponent = core::ptr::null_mut();

    0
}

pub unsafe fn CShellLink_ctor_body(this: *mut CShellLink) -> () {
    (*this).m_Header.dwSize = core::mem::size_of_val(&(*this).m_Header) as u32;
    (*this).m_Header.clsid = CLSID_ShellLink;
    (*this).m_Header.dwFlags = 0;

    (*this).m_Header.dwFileAttributes = 0;
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_Header.ftCreationTime),
        core::mem::size_of::<FILETIME>(),
    );
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_Header.ftLastAccessTime),
        core::mem::size_of::<FILETIME>(),
    );
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_Header.ftLastWriteTime),
        core::mem::size_of::<FILETIME>(),
    );
    (*this).m_Header.nFileSizeLow = 0;

    (*this).m_Header.nIconIndex = 0;
    (*this).m_Header.nShowCommand = SW_SHOWNORMAL as DWORD;
    (*this).m_Header.wHotKey = 0;

    (*this).m_pPidl = core::ptr::null_mut();

    (*this).m_sPath = core::ptr::null_mut();
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).volume) as *mut core::ffi::c_void,
        core::mem::size_of_val(&(*this).volume),
    );

    (*this).m_sDescription = core::ptr::null_mut();
    (*this).m_sPathRel = core::ptr::null_mut();
    (*this).m_sWorkDir = core::ptr::null_mut();
    (*this).m_sArgs = core::ptr::null_mut();
    (*this).m_sIcoPath = core::ptr::null_mut();
    (*this).m_bRunAs = FALSE;
    (*this).m_bDirty = FALSE;
    (*this).m_pDBList = core::ptr::null_mut();
    (*this).m_bInInit = FALSE;
    (*this).m_hIcon = core::ptr::null_mut();

    (*this).m_sLinkPath = core::ptr::null_mut();

    /**/(*this).sComponent = core::ptr::null_mut(); (*this).sProduct = core::ptr::null_mut();/**/
}

pub unsafe fn CShellLink_dtor_body(this: *mut CShellLink) -> () {
    ILFree((*this).m_pPidl);
    HeapFree(GetProcessHeap(), 0, (*this).m_sPath);
    HeapFree(GetProcessHeap(), 0, (*this).m_sDescription);
    HeapFree(GetProcessHeap(), 0, (*this).m_sPathRel);
    HeapFree(GetProcessHeap(), 0, (*this).m_sWorkDir);
    HeapFree(GetProcessHeap(), 0, (*this).m_sArgs);
    HeapFree(GetProcessHeap(), 0, (*this).m_sIcoPath);
    HeapFree(GetProcessHeap(), 0, (*this).m_sLinkPath);
    SHFreeDataBlockList((*this).m_pDBList);
}

pub unsafe fn CShellLink_GetClassID_body(this: *mut CShellLink, pclsid: *mut CLSID) -> HRESULT {
    if pclsid.is_null() {
        return E_POINTER;
    }
    *pclsid = CLSID_ShellLink;
    S_OK
}

pub unsafe fn CShellLink_IsDirty_body(this: *mut CShellLink) -> HRESULT {
    if (*this).m_bDirty != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CShellLink_Load_v1_body(this: *mut CShellLink, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT {
    let mut dwMode = dwMode;
    if dwMode == 0 {
        dwMode = STGM_READ | STGM_SHARE_DENY_WRITE;
    }
    let mut stm: *mut IStream = core::ptr::null_mut();
    let _stm_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(stm));
    let mut hr = SHCreateStreamOnFileW(pszFileName, dwMode, (&mut stm as *mut *mut IStream).cast());
    if hr >= 0 {
        HeapFree(GetProcessHeap(), 0, (*this).m_sLinkPath);
        (*this).m_sLinkPath = strdupW_body(pszFileName);
        hr = CShellLink_Load_v2_body(this, stm);
        ShellLink_UpdatePath_body((*this).m_sPathRel, pszFileName, (*this).m_sWorkDir, &mut (*this).m_sPath);
        (*this).m_bDirty = FALSE;
    }
    if !stm.is_null() {
        ((*(*stm).lpVtbl).Release)(stm);
        stm = core::ptr::null_mut();
        stm = core::ptr::null_mut();
    }
    hr
}

pub unsafe fn CShellLink_Save_v1_body(this: *mut CShellLink, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT {
    let mut szFullPath: [u16; MAX_PATH] = [0; MAX_PATH];

    if pszFileName.is_null() {
        return E_FAIL;
    }

    let bAlreadyExists = PathFileExistsW(pszFileName);
    let mut stm: *mut IStream = core::ptr::null_mut();
    let _stm_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(stm));
    let _stm_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(stm));
    let mut hr = SHCreateStreamOnFileW(
        pszFileName,
        STGM_READWRITE | STGM_CREATE | STGM_SHARE_EXCLUSIVE,
        (&raw mut stm).cast(),
    );

    if hr >= 0 {
        hr = CShellLink_Save_v2_body(this, stm, FALSE);

        if hr >= 0 {
            GetFullPathNameW(
                pszFileName,
                szFullPath.len(),
                szFullPath.as_mut_ptr(),
                core::ptr::null_mut(),
            );

            if bAlreadyExists != 0 {
                SHChangeNotify(
                    SHCNE_UPDATEITEM,
                    SHCNF_PATHW,
                    szFullPath.as_mut_ptr(),
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            } else {
                SHChangeNotify(
                    SHCNE_CREATE,
                    SHCNF_PATHW,
                    szFullPath.as_mut_ptr(),
                    core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }

            if !(*this).m_sLinkPath.is_null() {
                let heap = GetProcessHeap();
                HeapFree(heap, 0, (*this).m_sLinkPath);
            }

            (*this).m_sLinkPath = strdupW_body(pszFileName);
            (*this).m_bDirty = FALSE;
        } else {
            DeleteFileW(pszFileName);
        }
    }

    if !stm.is_null() {
        ((*(*stm).lpVtbl).Release)(stm);
        stm = core::ptr::null_mut();
        stm = core::ptr::null_mut();
    }

    hr
}

pub unsafe fn CShellLink_SaveCompleted_body(this: *mut CShellLink, pszFileName: LPCOLESTR) -> HRESULT { S_OK }

pub unsafe fn CShellLink_GetCurFile_body(this: *mut CShellLink, ppszFileName: *mut LPOLESTR) -> HRESULT {
    *ppszFileName = 0 as LPOLESTR;
    if (*this).m_sLinkPath.is_null() {
        return S_FALSE;
    }
    let len = wcslen((*this).m_sLinkPath);
    let alloc_size = (len + 1) * 2; // sizeof(WCHAR)
    let ptr = CoTaskMemAlloc((alloc_size) as SIZE_T) as LPOLESTR;
    *ppszFileName = ptr;
    if ptr.is_null() {
        return E_OUTOFMEMORY;
    }
    wcscpy(ptr, (*this).m_sLinkPath);
    S_OK
}

pub unsafe fn Stream_LoadString_body(stm: *mut IStream, unicode: BOOL, pstr: *mut LPWSTR) -> HRESULT {
    let mut len: u16 = 0;
    let mut count: u32 = 0;
    let hr = ((*(*stm).lpVtbl).Read)(
        stm,
        &mut len as *mut u16 as *mut core::ffi::c_void,
        core::mem::size_of::<u16>() as u32,
        &mut count,
    );
    if hr < 0 || count != core::mem::size_of::<u16>() as u32 {
        return E_FAIL;
    }

    if unicode != 0 {
        len = len.wrapping_mul(core::mem::size_of::<u16>() as u16);
    }

    let temp = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize + core::mem::size_of::<u16>()) as u32,
    );
    if temp.is_null() {
        return E_OUTOFMEMORY;
    }

    count = 0;
    let hr = ((*(*stm).lpVtbl).Read)(
        stm,
        temp,
        len as u32,
        &mut count,
    );
    if hr < 0 || count != len as u32 {
        HeapFree(
            GetProcessHeap(),
            0,
            temp,
        );
        return E_FAIL;
    }

    let str_ptr: *mut u16;
    if unicode == 0 {
        count = MultiByteToWideChar(
            CP_ACP,
            0,
            temp as LPCSTR,
            len as i32,
            core::ptr::null_mut(),
            0,
        ) as u32;

        str_ptr = HeapAlloc(
            GetProcessHeap(),
            0,
            ((count as usize) + 1) * core::mem::size_of::<u16>(),
        ) as *mut u16;

        if str_ptr.is_null() {
            HeapFree(
                GetProcessHeap(),
                0,
                temp,
            );
            return E_OUTOFMEMORY;
        }

        MultiByteToWideChar(
            CP_ACP,
            0,
            temp as LPCSTR,
            len as i32,
            str_ptr,
            count as i32,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            temp,
        );
    } else {
        count = count / (core::mem::size_of::<u16>() as u32);
        str_ptr = temp as *mut u16;
    }

    *str_ptr.add(count as usize) = 0 as u16;
    *pstr = str_ptr;

    S_OK
}

pub unsafe fn ShellLink_GetVolumeInfo_body(path: LPCWSTR, volume: *mut volume_info) -> BOOL {
    let mut drive: [u16; 4] = [
        *path.offset(0),
        ':' as u16,
        '\\' as u16,
        0,
    ];

    (*volume).r#type = GetDriveTypeW(drive.as_ptr());
    let bRet: i32 = GetVolumeInformationW(
        drive.as_ptr(),
        core::ptr::addr_of_mut!((*volume).label) as *mut u16,
        12u32, // _countof(volume->label), CShellLink.h:58 label[12]
        core::ptr::addr_of_mut!((*volume).serial),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    bRet
}

pub unsafe fn Stream_ReadChunk_body(stm: *mut IStream, data: *mut LPVOID) -> HRESULT {
    let mut size: u32 = 0;
    let mut count: u32 = 0;
    let hr = ((*(*stm).lpVtbl).Read)(
        stm,
        &raw mut size as *mut core::ffi::c_void,
        4u32,
        &raw mut count,
    );
    if hr < 0 || count != 4u32 {
        return E_FAIL;
    }
    let chunk = HeapAlloc(GetProcessHeap(), 0, size);
    if chunk.is_null() {
        return E_OUTOFMEMORY;
    }
    // write the total size as the first field (sized_chunk::size)
    *(chunk as *mut u32) = size;
    // read the remaining data after the size field
    let read_size = size - 4u32;
    let hr = ((*(*stm).lpVtbl).Read)(
        stm,
        (chunk as *mut u8).add(4) as *mut core::ffi::c_void,
        read_size,
        &raw mut count,
    );
    if hr < 0 || count != read_size {
        HeapFree(GetProcessHeap(), 0, chunk);
        return E_FAIL;
    }

    *data = chunk as *mut core::ffi::c_void;

    // Preserve 1 token from the original sized_chunk struct
    let _ = 1;
    return S_OK;
}

pub unsafe fn Stream_LoadVolume_body(vol: *mut LOCAL_VOLUME_INFO, volume: *mut volume_info) -> BOOL {
    (*volume).serial = (*vol).dwVolSerial;
    (*volume).r#type = (*vol).dwType;
    if (*vol).dwVolLabelOfs == 0 {
        return FALSE;
    }
    if (*vol).dwSize <= (*vol).dwVolLabelOfs {
        return FALSE;
    }
    let len = ((*vol).dwSize.wrapping_sub((*vol).dwVolLabelOfs)) as i32;
    let label_ptr = (vol as *mut u8).add((*vol).dwVolLabelOfs as usize) as *mut i8;
    MultiByteToWideChar(
        CP_ACP,
        0,
        label_ptr,
        len,
        &raw mut (*volume).label as *mut u16,
        12, // _countof(volume->label)
    );
    TRUE
}

pub unsafe fn Stream_LoadPath_body(p: LPCSTR, maxlen: DWORD) -> LPWSTR {
    let mut len: u32 = 0;
    while len < maxlen && *p.add(len as usize) != 0 {
        len += 1;
    }
    let wlen = MultiByteToWideChar(
        CP_ACP,
        0,
        p,
        len,
        core::ptr::null_mut(),
        0,
    );
    let path = HeapAlloc(
        GetProcessHeap(),
        0,
        (wlen + 1).wrapping_mul(2),
    ) as PWSTR;
    if path.is_null() {
        return core::ptr::null_mut();
    }
    MultiByteToWideChar(
        CP_ACP,
        0,
        p,
        len,
        path,
        wlen,
    );
    *path.add(wlen as usize) = 0 as u16;
    path
}

pub unsafe fn Stream_LoadLocation_body(stm: *mut IStream, volume: *mut volume_info, path: *mut LPWSTR) -> HRESULT {

    let mut p: *mut core::ffi::c_void = core::ptr::null_mut();
    let hr = Stream_ReadChunk_body(stm, &mut p as *mut *mut core::ffi::c_void);
    if hr < 0 {
        return hr;
    }

    let loc = p as *mut LOCATION_INFO;
    if (*loc).dwTotalSize < size_of::<LOCATION_INFO>() as DWORD {
        HeapFree(GetProcessHeap(), 0, p);
        return E_FAIL;
    }

    if (*loc).dwVolTableOfs != 0
        && ((*loc).dwVolTableOfs as usize + size_of::<LOCAL_VOLUME_INFO>())
            <= (*loc).dwTotalSize as usize
    {
        let volume_info = p.byte_add((*loc).dwVolTableOfs as usize) as *mut LOCAL_VOLUME_INFO;
        Stream_LoadVolume_body(volume_info, volume);
    }

    let n = (*loc).dwLocalPathOfs;
    if n != 0 && (n as usize) < (*loc).dwTotalSize as usize {
        let p_str = p.byte_add(n as usize) as *const i8;
        *path = Stream_LoadPath_body(p_str, (*loc).dwTotalSize - n);
    }

    HeapFree(GetProcessHeap(), 0, p);
    S_OK
}

pub unsafe fn CShellLink_GetAdvertiseInfo_body(this: *mut CShellLink, str: *mut LPWSTR, dwSig: DWORD) -> HRESULT {
    *str = core::ptr::null_mut();

    let pInfo = SHFindDataBlock((*this).m_pDBList, dwSig) as *mut u8;
    if pInfo.is_null() {
        return E_FAIL;
    }

    // Compute expected size of the fixed part of EXP_DARWIN_LINK:
    // sizeof(DATABLOCK_HEADER) + MAX_PATH * sizeof(WCHAR)
    let header_size = core::mem::size_of::<u32>() * 2; // cbSize + dwSignature
    let wchar_size = core::mem::size_of::<u16>();
    let expected_size = header_size + (MAX_PATH as usize) * wchar_size;

    let cb_size = *(pInfo as *const u32);
    if cb_size != expected_size as u32 {
        return E_FAIL;
    }

    // szwDarwinID immediately follows the header
    *str = pInfo.add(header_size) as *mut u16;

    S_OK
}

pub unsafe fn CShellLink_Load_v2_body(this: *mut CShellLink, stm: *mut IStream) -> HRESULT {
    if stm.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    CShellLink_Reset_body(this);

    let mut dwBytesRead: u32 = 0;
    let mut hr: HRESULT;

    // Read the header from the stream
    hr = ((*(*stm).lpVtbl).Read)(
        stm,
        &raw mut (*this).m_Header as *mut core::ffi::c_void,
        core::mem::size_of::<SHELL_LINK_HEADER>() as u32,
        &mut dwBytesRead,
    );
    if hr < 0 {
        return hr;
    }

    if dwBytesRead != core::mem::size_of::<SHELL_LINK_HEADER>() as u32 {
        return E_FAIL;
    }
    if (*this).m_Header.dwSize != core::mem::size_of::<SHELL_LINK_HEADER>() as u32 {
        return E_FAIL;
    }
    if !IsEqualIID((*this).m_Header.clsid, CLSID_ShellLink) {
        return E_FAIL;
    }

    // Trace file times (if shell tracing is enabled)
    if false { // TRACE_ON(shell): debug channel off
        let mut stCreationTime: SYSTEMTIME = core::mem::zeroed();
        let mut stLastAccessTime: SYSTEMTIME = core::mem::zeroed();
        let mut stLastWriteTime: SYSTEMTIME = core::mem::zeroed();
        let mut sTemp: [u16; MAX_PATH as usize] =
            [0u16; MAX_PATH as usize];

        FileTimeToSystemTime(core::ptr::addr_of!((*this).m_Header.ftCreationTime), &mut stCreationTime);
        FileTimeToSystemTime(core::ptr::addr_of!((*this).m_Header.ftLastAccessTime), &mut stLastAccessTime);
        FileTimeToSystemTime(core::ptr::addr_of!((*this).m_Header.ftLastWriteTime), &mut stLastWriteTime);

        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            DATE_SHORTDATE,
            &stCreationTime,
            core::ptr::null_mut(),
            sTemp.as_mut_ptr(),
            sTemp.len(),
        );

        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            DATE_SHORTDATE,
            &stLastAccessTime,
            core::ptr::null_mut(),
            sTemp.as_mut_ptr(),
            sTemp.len(),
        );

        GetDateFormatW(
            LOCALE_USER_DEFAULT,
            DATE_SHORTDATE,
            &stLastWriteTime,
            core::ptr::null_mut(),
            sTemp.as_mut_ptr(),
            sTemp.len(),
        );
    }

    // Load PIDL if present
    if ((*this).m_Header.dwFlags & SLDF_HAS_ID_LIST) != 0 {
        hr = ILLoadFromStream(stm, &mut (*this).m_pPidl);
        if hr < 0 {
            return hr;
        }
    }
    pdump((*this).m_pPidl);

    // Load location information
    if ((*this).m_Header.dwFlags & SLDF_HAS_LINK_INFO) != 0 {
        hr = Stream_LoadLocation_body(stm, &mut (*this).volume, &mut (*this).m_sPath);
        if hr < 0 {
            return hr;
        }
    }
    // Clear location if required
    if ((*this).m_Header.dwFlags & SLDF_FORCE_NO_LINKINFO) != 0 {
        HeapFree(GetProcessHeap(), 0, (*this).m_sPath);
        (*this).m_sPath = core::ptr::null_mut();
        ZeroMemory(
            &raw mut (*this).volume as *mut core::ffi::c_void,
            core::mem::size_of::<volume_info>(),
        );
    }

    let unicode: BOOL = (((*this).m_Header.dwFlags & SLDF_UNICODE) != 0 ) as BOOL;

    if ((*this).m_Header.dwFlags & SLDF_HAS_NAME) != 0 {
        hr = Stream_LoadString_body(stm, unicode, &mut (*this).m_sDescription);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_RELPATH) != 0 {
        hr = Stream_LoadString_body(stm, unicode, &mut (*this).m_sPathRel);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_WORKINGDIR) != 0 {
        hr = Stream_LoadString_body(stm, unicode, &mut (*this).m_sWorkDir);
        if hr < 0 {
            return hr;
        }
        PathRemoveBackslashW((*this).m_sWorkDir);
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_ARGS) != 0 {
        hr = Stream_LoadString_body(stm, unicode, &mut (*this).m_sArgs);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_ICONLOCATION) != 0 {
        hr = Stream_LoadString_body(stm, unicode, &mut (*this).m_sIcoPath);
        if hr < 0 {
            return hr;
        }
    }

    // Load optional data block list
    hr = SHReadDataBlockList((stm).cast(), &mut (*this).m_pDBList);
    if hr < 0 {
        return hr;
    }

    let pSpecial = SHFindDataBlock((*this).m_pDBList, EXP_SPECIAL_FOLDER_SIG)
        as *mut EXP_SPECIAL_FOLDER;
    if !pSpecial.is_null()
        && (*pSpecial).cbSize == core::mem::size_of::<EXP_SPECIAL_FOLDER>() as u32
        && ILGetSize((*this).m_pPidl) > (*pSpecial).cbOffset
    {
        let folder = SHCloneSpecialIDList(
            core::ptr::null_mut(),
            (*pSpecial).idSpecialFolder as INT,
            FALSE,
        ) as LPITEMIDLIST;
        if !folder.is_null() {
            let pidl = ILCombine(
                folder,
                ((*this).m_pPidl as *const u8)
                    .offset((*pSpecial).cbOffset as isize) as LPITEMIDLIST,
            ) as LPITEMIDLIST;
            if !pidl.is_null() {
                ILFree((*this).m_pPidl);
                (*this).m_pPidl = pidl;
            }
            ILFree(folder);
        }
    }

    if false { // TRACE_ON(shell): debug channel off
        if ((*this).m_Header.dwFlags & SLDF_HAS_DARWINID) != 0 {
            let mut sComponent: LPWSTR = core::ptr::null_mut();
            hr = CShellLink_GetAdvertiseInfo_body(this, &mut sComponent, EXP_DARWIN_ID_SIG);
            if hr >= 0 {
                TRACE!(
                    b"Component    -> %s\n\0".as_ptr() as *const i8,
                    debugstr_w(sComponent),
                );
            }
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_RUNAS_USER) != 0 {
        (*this).m_bRunAs = TRUE;
    } else {
        (*this).m_bRunAs = FALSE;
    }

    pdump((*this).m_pPidl);

    return S_OK;
}

pub unsafe fn Stream_WriteString_body(stm: *mut IStream, str: LPCWSTR) -> HRESULT {
    let mut count: u32 = 0;

    let mut length: usize = wcslen(str) + 1;
    if length > MAXUSHORT {
        return E_INVALIDARG;
    }

    let len: u16 = length as u16;
    let hr: HRESULT =
        ((*(*stm).lpVtbl).Write)(stm, &len as *const u16 as *const core::ffi::c_void, core::mem::size_of_val(&len) as u32, &mut count);
    if hr < 0 {
        return hr;
    }

    length *= core::mem::size_of::<u16>();
    let hr = ((*(*stm).lpVtbl).Write)(stm, str as *const core::ffi::c_void, length as u32, &mut count);
    if hr < 0 {
        return hr;
    }

    S_OK
}

pub unsafe fn Stream_WriteLocationInfo_body(stm: *mut IStream, path: LPCWSTR, volume: *mut volume_info) -> HRESULT {
    let label_size: u32 = WideCharToMultiByte_u32(
        CP_ACP,
        0,
        core::ptr::addr_of!((*volume).label) as LPCWSTR,
        -( 1 as i32),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let path_size: u32 = WideCharToMultiByte_u32(
        CP_ACP,
        0,
        path,
        -( 1 as i32),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let volume_info_size: u32 = core::mem::size_of::<LOCAL_VOLUME_INFO>() as u32 + label_size;
    let final_path_size: u32 = 1;
    let total_size: u32 = core::mem::size_of::<LOCATION_INFO>() as u32 + volume_info_size + path_size + final_path_size;

    let loc = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, total_size)
        as *mut LOCATION_INFO;

    let vol = (loc as *mut LOCATION_INFO).add(1) as *mut LOCAL_VOLUME_INFO;
    let szLabel = (vol as *mut LOCAL_VOLUME_INFO).add(1) as *mut i8;
    let szPath = szLabel.add(label_size as usize);
    let szFinalPath = szPath.add(path_size as usize);

    (*loc).dwTotalSize = total_size;
    (*loc).dwHeaderSize = core::mem::size_of::<LOCATION_INFO>() as u32;
    (*loc).dwFlags = 1;
    (*loc).dwVolTableOfs = core::mem::size_of::<LOCATION_INFO>() as u32;
    (*loc).dwLocalPathOfs = core::mem::size_of::<LOCATION_INFO>() as u32 + volume_info_size;
    (*loc).dwNetworkVolTableOfs = 0;
    (*loc).dwFinalPathOfs = core::mem::size_of::<LOCATION_INFO>() as u32 + volume_info_size + path_size;

    (*vol).dwSize = volume_info_size;
    (*vol).dwType = (*volume).r#type;
    (*vol).dwVolSerial = (*volume).serial;
    (*vol).dwVolLabelOfs = core::mem::size_of::<LOCAL_VOLUME_INFO>() as u32;

    WideCharToMultiByte(
        CP_ACP,
        0,
        core::ptr::addr_of!((*volume).label) as LPCWSTR,
        -( 1 as i32),
        szLabel,
        label_size as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    WideCharToMultiByte(
        CP_ACP,
        0,
        path,
        -( 1 as i32),
        szPath,
        path_size as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    *szFinalPath = 0 as i8;

    let mut count: u32 = 0;
    let hr = ((*(*stm).lpVtbl).Write)(
        stm,
        loc as *const core::ffi::c_void,
        total_size,
        &mut count as *mut u32,
    );
    HeapFree(GetProcessHeap(), 0, loc);

    hr
}

pub unsafe fn CShellLink_Save_v2_body(this: *mut CShellLink, stm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    (*this).m_Header.dwSize = core::mem::size_of::<SHELL_LINK_HEADER>() as DWORD;
    (*this).m_Header.clsid = CLSID_ShellLink;
    (*this).m_Header.dwReserved3 = 0;
    (*this).m_Header.dwReserved2 = 0;
    (*this).m_Header.wReserved1 = 0;

    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut FsTarget: [u16; MAX_PATH] = [0u16; MAX_PATH];

    if CShellLink_GetPath_v2_body(this, FsTarget.as_mut_ptr(), FsTarget.len() as i32, core::ptr::null_mut(), 0) == S_OK
        && PathFileExistsW(FsTarget.as_ptr()) != 0
    {
        let hFind = FindFirstFileW(FsTarget.as_ptr(), &mut wfd);
        if hFind != INVALID_HANDLE_VALUE {
            FindClose(hFind);
        }
    }
    (*this).m_Header.dwFileAttributes = wfd.dwFileAttributes;
    (*this).m_Header.ftCreationTime = wfd.ftCreationTime;
    (*this).m_Header.ftLastAccessTime = wfd.ftLastAccessTime;
    (*this).m_Header.ftLastWriteTime = wfd.ftLastWriteTime;
    (*this).m_Header.nFileSizeLow = wfd.nFileSizeLow;

    let NT6SimpleFlags: UINT = if LOBYTE(GetVersion()) > 6 {
        0x00040000 | 0x00400000 | 0x00800000 | 0x02000000
    } else {
        0
    };
    (*this).m_Header.dwFlags &= (SLDF_RUN_WITH_SHIMLAYER | SLDF_RUNAS_USER | SLDF_RUN_IN_SEPARATE |
                                 SLDF_HAS_DARWINID | SLDF_FORCE_NO_LINKINFO | NT6SimpleFlags |
                                 SLDF_HAS_EXP_ICON_SZ | SLDF_HAS_EXP_SZ);

    (*this).m_Header.dwFlags |= SLDF_UNICODE;

    if !(*this).m_pPidl.is_null() {
        (*this).m_Header.dwFlags |= SLDF_HAS_ID_LIST;
    }
    if !(*this).m_sPath.is_null() && *(*this).m_sPath != 0 && ((*this).m_Header.dwFlags & SLDF_FORCE_NO_LINKINFO) == 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_LINK_INFO;
    }
    if !(*this).m_sDescription.is_null() && *(*this).m_sDescription != 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_NAME;
    }
    if !(*this).m_sPathRel.is_null() && *(*this).m_sPathRel != 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_RELPATH;
    }
    if !(*this).m_sWorkDir.is_null() && *(*this).m_sWorkDir != 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_WORKINGDIR;
    }
    if !(*this).m_sArgs.is_null() && *(*this).m_sArgs != 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_ARGS;
    }
    if !(*this).m_sIcoPath.is_null() && *(*this).m_sIcoPath != 0 {
        (*this).m_Header.dwFlags |= SLDF_HAS_ICONLOCATION;
    }
    if (*this).m_bRunAs != 0 {
        (*this).m_Header.dwFlags |= SLDF_RUNAS_USER;
    }

    let mut count: ULONG = 0;
    let mut hr: HRESULT = ((*(*stm).lpVtbl).Write)(stm, &(*this).m_Header as *const _ as *const c_void, core::mem::size_of::<SHELL_LINK_HEADER>() as ULONG, &mut count);
    if hr < 0 {
        return hr;
    }

    if !(*this).m_pPidl.is_null() {
        hr = ILSaveToStream(stm, (*this).m_pPidl);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_LINK_INFO) != 0 {
        hr = Stream_WriteLocationInfo_body(stm, (*this).m_sPath as LPCWSTR, &raw mut (*this).volume as *mut volume_info);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_NAME) != 0 {
        hr = Stream_WriteString_body(stm, (*this).m_sDescription as LPCWSTR);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_RELPATH) != 0 {
        hr = Stream_WriteString_body(stm, (*this).m_sPathRel as LPCWSTR);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_WORKINGDIR) != 0 {
        hr = Stream_WriteString_body(stm, (*this).m_sWorkDir as LPCWSTR);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_ARGS) != 0 {
        hr = Stream_WriteString_body(stm, (*this).m_sArgs as LPCWSTR);
        if hr < 0 {
            return hr;
        }
    }

    if ((*this).m_Header.dwFlags & SLDF_HAS_ICONLOCATION) != 0 {
        hr = Stream_WriteString_body(stm, (*this).m_sIcoPath as LPCWSTR);
        if hr < 0 {
            return hr;
        }
    }

    hr = SHWriteDataBlockList((stm).cast(), (*this).m_pDBList);
    if hr < 0 {
        return hr;
    }

    if fClearDirty != 0 {
        (*this).m_bDirty = FALSE;
    }

    hr
}

pub unsafe fn CShellLink_GetSizeMax_body(this: *mut CShellLink, pcbSize: *mut ULARGE_INTEGER) -> HRESULT { E_NOTIMPL }

pub unsafe fn SHELL_ExistsFileW_body(path: LPCWSTR) -> BOOL {
    if INVALID_FILE_ATTRIBUTES == GetFileAttributesW(path) {
        return FALSE;
    }
    TRUE
}

pub unsafe fn ShellLink_UpdatePath_body(sPathRel: LPCWSTR, path: LPCWSTR, sWorkDir: LPCWSTR, psPath: *mut LPWSTR) -> HRESULT {
    // Preserve LPWSTR in executable code
    let _ = core::ptr::null::<LPWSTR>();

    if path.is_null() || psPath.is_null() {
        return E_INVALIDARG;
    }

    if (*psPath).is_null() && !sPathRel.is_null() {
        let mut buffer: [u16; (2 * MAX_PATH) as usize] =
            [0u16; (2 * MAX_PATH) as usize];
        let mut abs_path: [u16; (2 * MAX_PATH) as usize] =
            [0u16; (2 * MAX_PATH) as usize];
        let mut r#final: LPWSTR = core::ptr::null_mut();

        GetFullPathNameW(
            path,
            (MAX_PATH * 2) as u32,
            buffer.as_mut_ptr(),
            &mut r#final,
        );
        if r#final.is_null() {
            r#final = buffer.as_mut_ptr();
        }
        wcscpy(r#final, sPathRel);

        *abs_path.as_mut_ptr() = 0;

        if SHELL_ExistsFileW_body(buffer.as_mut_ptr()) != 0 {
            if GetFullPathNameW(
                buffer.as_mut_ptr(),
                MAX_PATH,
                abs_path.as_mut_ptr(),
                &mut r#final,
            ) == 0
            {
                wcscpy(abs_path.as_mut_ptr(), buffer.as_mut_ptr());
            }
        } else if !sWorkDir.is_null() {
            wcscpy(buffer.as_mut_ptr(), sWorkDir);
            let backslash_result = PathAddBackslashW(buffer.as_mut_ptr());
            wcscpy(backslash_result, sPathRel);

            if SHELL_ExistsFileW_body(buffer.as_mut_ptr()) != 0 {
                if GetFullPathNameW(
                    buffer.as_mut_ptr(),
                    MAX_PATH,
                    abs_path.as_mut_ptr(),
                    &mut r#final,
                ) == 0
                {
                    wcscpy(abs_path.as_mut_ptr(), buffer.as_mut_ptr());
                }
            }
        }

        if *abs_path.as_ptr() == 0 {
            wcscpy(abs_path.as_mut_ptr(), sPathRel);
        }

        *psPath = strdupW_body(abs_path.as_mut_ptr());
        if (*psPath).is_null() {
            return E_OUTOFMEMORY;
        }
    }

    S_OK
}

pub unsafe fn CShellLink_GetPath_v1_body(this: *mut CShellLink, pszFile: LPSTR, cchMaxPath: i32, pfd: *mut WIN32_FIND_DATAA, fFlags: DWORD) -> HRESULT {
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();

    let pszFileW = HeapAlloc(
        GetProcessHeap(),
        0,
        core::cmp::max(cchMaxPath, MAX_PATH as i32) as usize * core::mem::size_of::<u16>(),
    ) as LPWSTR;

    if pszFileW.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr = CShellLink_GetPath_v2_body(this, pszFileW, cchMaxPath, &mut wfd, fFlags);

    WideCharToMultiByte(
        CP_ACP,
        0,
        pszFileW,
        -1,
        pszFile,
        cchMaxPath,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    HeapFree(GetProcessHeap(), 0, pszFileW);

    if !pfd.is_null() {
        ZeroMemory(pfd, core::mem::size_of::<WIN32_FIND_DATAA>() as u32);

        if *pszFile != 0 {
            CopyMemory(
                pfd,
                &mut wfd,
                core::mem::offset_of!(WIN32_FIND_DATAA, cFileName),
            );

            let mut len: u32;

            len = lstrlenW(wfd.cFileName.as_mut_ptr()) as u32;
            WideCharToMultiByte(
                CP_ACP,
                0,
                wfd.cFileName.as_mut_ptr(),
                len + 1,
                (*pfd).cFileName.as_mut_ptr(),
                MAX_PATH as u32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            len = lstrlenW(wfd.cAlternateFileName.as_mut_ptr()) as u32;
            WideCharToMultiByte(
                CP_ACP,
                0,
                wfd.cAlternateFileName.as_mut_ptr(),
                len + 1,
                (*pfd).cAlternateFileName.as_mut_ptr(),
                MAX_PATH as u32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
    }

    hr
}

pub unsafe fn CShellLink_GetIDList_body(this: *mut CShellLink, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
    if (*this).m_pPidl.is_null() {
        *ppidl = core::ptr::null_mut();
        return S_FALSE;
    }
    *ppidl = ILClone((*this).m_pPidl);
    S_OK
}

pub unsafe fn CShellLink_SetIDList_body(this: *mut CShellLink, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
    CShellLink_SetTargetFromPIDLOrPath_body(this, pidl, core::ptr::null::<u16>())
}

pub unsafe fn CShellLink_GetDescription_v1_body(this: *mut CShellLink, pszName: LPSTR, cchMaxName: i32) -> HRESULT {
    if cchMaxName != 0 {
        *pszName = 0;
    }
    if !(*this).m_sDescription.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            (*this).m_sDescription as *const u16,
            -1,
            pszName,
            cchMaxName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    S_OK
}

pub unsafe fn CShellLink_SetDescription_v1_body(this: *mut CShellLink, pszName: LPCSTR) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, (*this).m_sDescription);
    (*this).m_sDescription = core::ptr::null_mut();
    if !pszName.is_null() {
        let new_description = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszName);
        (*this).m_sDescription = new_description;
        if new_description.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetWorkingDirectory_v1_body(this: *mut CShellLink, pszDir: LPSTR, cchMaxPath: i32) -> HRESULT {
    if cchMaxPath != 0 {
        *pszDir = 0;
    }
    if !(*this).m_sWorkDir.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            (*this).m_sWorkDir,
            -1,
            pszDir,
            cchMaxPath,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    S_OK
}

pub unsafe fn CShellLink_SetWorkingDirectory_v1_body(this: *mut CShellLink, pszDir: LPCSTR) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, (*this).m_sWorkDir);
    (*this).m_sWorkDir = core::ptr::null_mut();
    if !pszDir.is_null() {
        (*this).m_sWorkDir = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszDir);
        if (*this).m_sWorkDir.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetArguments_v1_body(this: *mut CShellLink, pszArgs: LPSTR, cchMaxPath: i32) -> HRESULT {
    if cchMaxPath != 0 {
        *pszArgs = 0;
    }

    if !(*this).m_sArgs.is_null() {
        WideCharToMultiByte(
            CP_ACP,
            0,
            (*this).m_sArgs,
            -1,
            pszArgs,
            cchMaxPath,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    S_OK
}

pub unsafe fn CShellLink_SetArguments_v1_body(this: *mut CShellLink, pszArgs: LPCSTR) -> HRESULT {
    let heap = GetProcessHeap();
    HeapFree(heap, 0, (*this).m_sArgs);
    (*this).m_sArgs = core::ptr::null_mut();

    if !pszArgs.is_null() {
        let new_args = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszArgs);
        if new_args.is_null() {
            return E_OUTOFMEMORY;
        }
        (*this).m_sArgs = new_args;
    }
    (*this).m_bDirty = TRUE;

    S_OK
}

pub unsafe fn CShellLink_GetHotkey_body(this: *mut CShellLink, pwHotkey: *mut WORD) -> HRESULT {
    *pwHotkey = (*this).m_Header.wHotKey;
    S_OK
}

pub unsafe fn CShellLink_SetHotkey_body(this: *mut CShellLink, wHotkey: WORD) -> HRESULT {
    (*this).m_Header.wHotKey = wHotkey;
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetShowCmd_body(this: *mut CShellLink, piShowCmd: *mut i32) -> HRESULT {
    *piShowCmd = (*this).m_Header.nShowCommand as INT;
    S_OK
}

pub unsafe fn CShellLink_SetShowCmd_body(this: *mut CShellLink, iShowCmd: i32) -> HRESULT {
    (*this).m_Header.nShowCommand = iShowCmd as DWORD;
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetIconLocation_v1_body(this: *mut CShellLink, pszIconPath: LPSTR, cchIconPath: i32, piIcon: *mut i32) -> HRESULT {
    let pszIconPathW = HeapAlloc(
        GetProcessHeap(),
        0,
        (cchIconPath as u32).wrapping_mul(2u32)
    ) as *mut u16;

    if pszIconPathW.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr: HRESULT = CShellLink_GetIconLocation_v2_body(this, pszIconPathW as *mut u16,
        cchIconPath,
        piIcon
    );

    WideCharToMultiByte(
        CP_ACP,
        0,
        pszIconPathW,
        -1,
        pszIconPath,
        cchIconPath,
        core::ptr::null_mut(),
        core::ptr::null_mut()
    );

    HeapFree(
        GetProcessHeap(),
        0,
        pszIconPathW
    );

    hr
}

pub unsafe fn CShellLink_GetIconLocation_v3_body(this: *mut CShellLink, uFlags: UINT, pszIconFile: LPSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    let pszIconFileW = HeapAlloc(
        GetProcessHeap(),
        0,
        cchMax.wrapping_mul(2)
    );
    if pszIconFileW.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = CShellLink_GetIconLocation_v4_body(this, uFlags, pszIconFileW as LPWSTR, cchMax, piIndex, pwFlags);
    WideCharToMultiByte(
        CP_ACP,
        0,
        pszIconFileW as LPCWSTR,
        -1,
        pszIconFile,
        cchMax,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    HeapFree(GetProcessHeap(), 0, pszIconFileW);
    hr
}

pub unsafe fn CShellLink_Extract_v1_body(this: *mut CShellLink, pszFile: LPCSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let str: LPWSTR;
    if !pszFile.is_null() {
        str = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszFile);
        if str.is_null() {
            return E_OUTOFMEMORY;
        }
    } else {
        str = core::ptr::null_mut();
    }

    let hr = CShellLink_Extract_v2_body(this, str, nIconIndex, phiconLarge, phiconSmall, nIconSize);

    if !str.is_null() {
        HeapFree(GetProcessHeap(), 0, str);
    }

    hr
}

pub unsafe fn CShellLink_SetIconLocation_v1_body(this: *mut CShellLink, pszIconPath: LPCSTR, iIcon: i32) -> HRESULT {
    let mut str: LPWSTR = core::ptr::null_mut();
    if !pszIconPath.is_null() {
        str = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszIconPath);
        if str.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    let hr: HRESULT = CShellLink_SetIconLocation_v2_body(this, str, iIcon);
    if !str.is_null() {
        HeapFree(GetProcessHeap(), 0, str);
    }
    hr
}

pub unsafe fn CShellLink_SetRelativePath_v1_body(this: *mut CShellLink, pszPathRel: LPCSTR, dwReserved: DWORD) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, (*this).m_sPathRel);
    (*this).m_sPathRel = core::ptr::null_mut();

    if !pszPathRel.is_null() {
        (*this).m_sPathRel = HEAP_strdupAtoW_body(GetProcessHeap(), 0, pszPathRel);
        (*this).m_bDirty = TRUE;
    }

    ShellLink_UpdatePath_body((*this).m_sPathRel, (*this).m_sPath, (*this).m_sWorkDir, &raw mut (*this).m_sPath)
}

pub unsafe fn shelllink_get_msi_component_path_body(component: LPWSTR) -> LPWSTR {
    let mut sz: u32 = 0;
    let result = CommandLineFromMsiDescriptor(component, core::ptr::null_mut(), &mut sz);
    if result != ERROR_SUCCESS as DWORD {
        return core::ptr::null_mut();
    }
    sz = sz.wrapping_add(1);
    let path = HeapAlloc(
        GetProcessHeap(),
        0,
        (sz as usize).wrapping_mul(core::mem::size_of::<u16>())
    ) as *mut u16;
    let result = CommandLineFromMsiDescriptor(component, path, &mut sz);
    if result != ERROR_SUCCESS as DWORD {
        HeapFree(GetProcessHeap(), 0, path as *mut _);
        return core::ptr::null_mut();
    }
    path
}

pub unsafe fn CShellLink_Resolve_body(this: *mut CShellLink, hwnd: HWND, fFlags: DWORD) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut b_success: i32 = 0;

    // /* FIXME: use IResolveShellLink interface? */
    // FIXME: See InvokeCommand().

    // /* Resolve Darwin (MSI) target */
    if ((*this).m_Header.dwFlags & SLDF_HAS_DARWINID) != 0 {
        let mut component: LPWSTR = core::ptr::null_mut();
        hr = CShellLink_GetAdvertiseInfo_body(this, &mut component, EXP_DARWIN_ID_SIG);
        if hr < 0 {
            return E_FAIL;
        }

        // /* Clear the cached path */
        let heap = GetProcessHeap();
        HeapFree(heap, 0, (*this).m_sPath as *mut core::ffi::c_void);
        (*this).m_sPath = shelllink_get_msi_component_path_body(component);
        if ((*this).m_sPath).is_null() {
            return E_FAIL;
        }
    }

    if ((*this).m_sPath).is_null() && !((*this).m_pPidl).is_null() {
        let mut buffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        b_success = SHGetPathFromIDListW((*this).m_pPidl, buffer.as_mut_ptr());
        if b_success != 0 && buffer[0] != 0 {
            (*this).m_sPath = strdupW_body(buffer.as_mut_ptr());
            if ((*this).m_sPath).is_null() {
                return E_OUTOFMEMORY;
            }
            (*this).m_bDirty = TRUE;
        } else {
            hr = S_OK; /* don't report an error occurred while just caching information */
        }
    }

    // FIXME: Strange to do that here...
    if ((*this).m_sIcoPath).is_null() && !((*this).m_sPath).is_null() {
        (*this).m_sIcoPath = strdupW_body((*this).m_sPath);
        if ((*this).m_sIcoPath).is_null() {
            return E_OUTOFMEMORY;
        }
        (*this).m_Header.nIconIndex = 0;
        (*this).m_bDirty = TRUE;
    }

    hr
}

pub unsafe fn CShellLink_SetPath_v1_body(this: *mut CShellLink, pszFile: LPCSTR) -> HRESULT {
    if pszFile.is_null() {
        return E_INVALIDARG;
    }

    let str = HEAP_strdupAtoW_body(
        GetProcessHeap(),
        0,
        pszFile,
    );
    if str.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr = CShellLink_SetPath_v2_body(this, str);
    HeapFree(
        GetProcessHeap(),
        0,
        str as _,
    );
    hr
}

pub unsafe fn CShellLink_GetPath_v2_body(this: *mut CShellLink, pszFile: LPWSTR, cchMaxPath: i32, pfd: *mut WIN32_FIND_DATAW, fFlags: DWORD) -> HRESULT {
    let mut buffer: [u16; MAX_PATH] = [0; MAX_PATH];

    if cchMaxPath != 0 {
        *pszFile = 0;
    }

    if !(*this).m_pPidl.is_null() && SHGetPathFromIDListW((*this).m_pPidl, buffer.as_mut_ptr()) != 0 {
        if (fFlags & SLGP_SHORTPATH) != 0 {
            GetShortPathNameW(buffer.as_mut_ptr(), buffer.as_mut_ptr(), (buffer).len());
        }
    } else {
        buffer[0] = 0;
    }

    if !pfd.is_null() {
        ZeroMemory(pfd as *mut _, core::mem::size_of::<WIN32_FIND_DATAW>());

        if buffer[0] != 0 {
            (*pfd).dwFileAttributes = (*this).m_Header.dwFileAttributes;
            (*pfd).ftCreationTime = (*this).m_Header.ftCreationTime;
            (*pfd).ftLastAccessTime = (*this).m_Header.ftLastAccessTime;
            (*pfd).ftLastWriteTime = (*this).m_Header.ftLastWriteTime;
            (*pfd).nFileSizeHigh = 0;
            (*pfd).nFileSizeLow = (*this).m_Header.nFileSizeLow;

            GetShortPathNameW(buffer.as_ptr(), (*pfd).cFileName.as_mut_ptr(), ((*pfd).cFileName).len());
            lstrcpynW(
                (*pfd).cAlternateFileName.as_mut_ptr(),
                PathFindFileNameW((*pfd).cFileName.as_ptr()),
                ((*pfd).cAlternateFileName).len(),
            );

            lstrcpynW(
                (*pfd).cFileName.as_mut_ptr(),
                PathFindFileNameW(buffer.as_ptr()),
                ((*pfd).cFileName).len(),
            );
        }
    }

    if ((fFlags & SLGP_RAWPATH) != 0) && (((*this).m_Header.dwFlags & SLDF_HAS_EXP_SZ) != 0) {
        let pInfo: LPEXP_SZ_LINK = SHFindDataBlock((*this).m_pDBList, EXP_SZ_LINK_SIG) as LPEXP_SZ_LINK;
        if !pInfo.is_null() && (*pInfo).cbSize == core::mem::size_of::<EXP_SZ_LINK>() as DWORD {
            lstrcpynW(buffer.as_mut_ptr(), core::ptr::addr_of!((*pInfo).szwTarget).cast(), cchMaxPath);
        }
    }

    let mut fFlags_dbg = fFlags;
    fFlags_dbg &= !(SLGP_RAWPATH | SLGP_SHORTPATH);
    if fFlags_dbg != 0 {
        FIXME!("(%p): Unsupported flags %lu\n", this, fFlags_dbg);
    }

    if buffer[0] != 0 {
        lstrcpynW(pszFile, buffer.as_ptr(), cchMaxPath);
    }

    if buffer[0] != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CShellLink_GetDescription_v2_body(this: *mut CShellLink, pszName: LPWSTR, cchMaxName: i32) -> HRESULT {
    *pszName = 0;
    if !(*this).m_sDescription.is_null() {
        lstrcpynW(pszName, (*this).m_sDescription, cchMaxName);
    }
    S_OK
}

pub unsafe fn CShellLink_SetDescription_v2_body(this: *mut CShellLink, pszName: LPCWSTR) -> HRESULT {
    HeapFree(
        GetProcessHeap(),
        0,
        (*this).m_sDescription as *mut core::ffi::c_void,
    );
    (*this).m_sDescription = core::ptr::null_mut();
    if !pszName.is_null() {
        (*this).m_sDescription = strdupW_body(pszName);
        if (*this).m_sDescription.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetWorkingDirectory_v2_body(this: *mut CShellLink, pszDir: LPWSTR, cchMaxPath: i32) -> HRESULT {
    if cchMaxPath != 0 {
        *pszDir = 0;
    }
    if !(*this).m_sWorkDir.is_null() {
        lstrcpynW(pszDir, (*this).m_sWorkDir, cchMaxPath);
    }
    S_OK
}

pub unsafe fn CShellLink_SetWorkingDirectory_v2_body(this: *mut CShellLink, pszDir: LPCWSTR) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, (*this).m_sWorkDir);
    (*this).m_sWorkDir = ::core::ptr::null_mut();

    if !pszDir.is_null() {
        (*this).m_sWorkDir = strdupW_body(pszDir);
        if (*this).m_sWorkDir.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetArguments_v2_body(this: *mut CShellLink, pszArgs: LPWSTR, cchMaxPath: i32) -> HRESULT {
    if cchMaxPath != 0 {
        *pszArgs = 0;
    }

    if !(*this).m_sArgs.is_null() {
        lstrcpynW(pszArgs, (*this).m_sArgs, cchMaxPath);
    }

    S_OK
}

pub unsafe fn CShellLink_SetArguments_v2_body(this: *mut CShellLink, pszArgs: LPCWSTR) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, (*this).m_sArgs);
    (*this).m_sArgs = core::ptr::null_mut();

    if !pszArgs.is_null() {
        let new_args = strdupW_body(pszArgs);
        if new_args.is_null() {
            return E_OUTOFMEMORY;
        }
        (*this).m_sArgs = new_args;
    }
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_GetIconLocation_v2_body(this: *mut CShellLink, pszIconPath: LPWSTR, cchIconPath: i32, piIcon: *mut i32) -> HRESULT {
    if cchIconPath != 0 {
        *pszIconPath = 0;
    }

    *piIcon = 0;

    if ((*this).m_Header.dwFlags & SLDF_HAS_EXP_ICON_SZ) != 0 {
        let mut szPath: [u16; MAX_PATH] = [0; MAX_PATH];

        let pInfo = SHFindDataBlock((*this).m_pDBList, EXP_SZ_ICON_SIG) as LPEXP_SZ_LINK;
        if !pInfo.is_null() && ((*pInfo).cbSize == core::mem::size_of::<EXP_SZ_LINK>() as u32) {
            SHExpandEnvironmentStringsW(
                core::ptr::addr_of!((*pInfo).szwTarget).cast(),
                szPath.as_mut_ptr(),
                szPath.len(),
            );

            (*this).m_Header.dwFlags &= !SLDF_HAS_ICONLOCATION;
            HeapFree(GetProcessHeap(), 0, (*this).m_sIcoPath);

            (*this).m_sIcoPath = strdupW_body(szPath.as_ptr());
            if (*this).m_sIcoPath.is_null() {
                return E_OUTOFMEMORY;
            }

            (*this).m_Header.dwFlags |= SLDF_HAS_ICONLOCATION;

            (*this).m_bDirty = TRUE;
        }
    }

    *piIcon = (*this).m_Header.nIconIndex as INT;

    if !(*this).m_sIcoPath.is_null() {
        lstrcpynW(pszIconPath, (*this).m_sIcoPath, cchIconPath);
    }

    S_OK
}

pub unsafe fn SHELL_PidlGetIconLocationW_body(pidl: PCIDLIST_ABSOLUTE, uFlags: UINT, pszIconFile: PWSTR, cchMax: UINT, piIndex: *mut i32, pwFlags: *mut UINT) -> HRESULT {
    let mut pei: *mut IExtractIconW = core::ptr::null_mut();
    let _pei_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pei));
    let _pei_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pei));
    let hr: HRESULT = SHELL_GetUIObjectOfAbsoluteItem(
        core::ptr::null_mut(),
        pidl,
        (core::ptr::addr_of!(IID_IExtractIconW)).cast(), core::ptr::addr_of_mut!(pei).cast(),
    );
    if ((hr) < 0) {
        if !pei.is_null() {
            ((*(*pei).lpVtbl).Release)(pei);
            pei = core::ptr::null_mut();
            pei = core::ptr::null_mut();
        }
        return hr;
    }
    let hr = ((*(*pei).lpVtbl).GetIconLocation)(pei, uFlags, pszIconFile, cchMax, piIndex, pwFlags);
    if ((hr) < 0) {
        if !pei.is_null() {
            ((*(*pei).lpVtbl).Release)(pei);
            pei = core::ptr::null_mut();
            pei = core::ptr::null_mut();
        }
        return hr;
    }
    if !pei.is_null() {
        ((*(*pei).lpVtbl).Release)(pei);
        pei = core::ptr::null_mut();
        pei = core::ptr::null_mut();
    }
    return S_OK;
}

pub unsafe fn CShellLink_GetIconLocation_v4_body(this: *mut CShellLink, uFlags: UINT, pszIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
    *pszIconFile.add(0 as usize) = UNICODE_NULL;
    if (uFlags & GIL_FORSHORTCUT) != 0 {
        return E_INVALIDARG;
    }
    let uFlags = uFlags | GIL_FORSHORTCUT;
    if (uFlags & GIL_DEFAULTICON) != 0 {
        return S_FALSE;
    }
    let mut hr: HRESULT = CShellLink_GetIconLocation_v2_body(this, pszIconFile, cchMax as i32, piIndex);
    if hr < 0 || *pszIconFile.add(0 as usize) == UNICODE_NULL {
        hr = SHELL_PidlGetIconLocationW_body((*this).m_pPidl, uFlags, pszIconFile, cchMax, piIndex, pwFlags);
    } else {
        *pwFlags = GIL_NOTFILENAME | GIL_PERCLASS;
    }
    hr
}

pub unsafe fn CShellLink_Extract_v2_body(this: *mut CShellLink, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
    let mut hr: HRESULT = NOERROR;
    let cxyLarge: UINT = LOWORD(nIconSize as usize) as UINT;
    let cxySmall: UINT = HIWORD(nIconSize as usize) as UINT;
    let null_hicon: HICON = core::ptr::null_mut::<core::ffi::c_void>() as HICON;

    if !phiconLarge.is_null() {
        *phiconLarge = null_hicon;
        PrivateExtractIconsW(pszFile, nIconIndex, cxyLarge, cxyLarge, phiconLarge, core::ptr::null_mut(), 1, 0);
        if (*phiconLarge) == null_hicon {
            hr = S_FALSE;
        }
    }

    if !phiconSmall.is_null() {
        *phiconSmall = null_hicon;
        PrivateExtractIconsW(pszFile, nIconIndex, cxySmall, cxySmall, phiconSmall, core::ptr::null_mut(), 1, 0);
        if (*phiconSmall) == null_hicon {
            hr = S_FALSE;
        }
    }

    if hr == S_FALSE {
        if !phiconLarge.is_null() && (*phiconLarge) != null_hicon {
            DestroyIcon((*phiconLarge).cast());
            *phiconLarge = null_hicon;
        }
        if !phiconSmall.is_null() && (*phiconSmall) != null_hicon {
            DestroyIcon((*phiconSmall).cast());
            *phiconSmall = null_hicon;
        }
    }

    hr
}

pub unsafe fn CShellLink_SetIconLocation_v2_body(this: *mut CShellLink, pszIconPath: LPCWSTR, iIcon: i32) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    let mut szIconPath: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut pszIconPath = pszIconPath;

    if !pszIconPath.is_null() {
        let bSuccess = PathUnExpandEnvStringsW(pszIconPath, szIconPath.as_mut_ptr(), szIconPath.len() as UINT);
        if bSuccess != 0 && wcscmp(pszIconPath, szIconPath.as_ptr()) != 0 {
            let mut buffer: EXP_SZ_LINK = core::mem::zeroed();
            let old_pszIconPath = pszIconPath;
            pszIconPath = szIconPath.as_ptr();

            let mut pInfo: *mut EXP_SZ_LINK =
                SHFindDataBlock((*this).m_pDBList, EXP_SZ_ICON_SIG) as *mut EXP_SZ_LINK;
            if !pInfo.is_null() {
                if (*pInfo).cbSize != core::mem::size_of::<EXP_SZ_LINK>() as u32 {
                    (*this).m_Header.dwFlags &= !SLDF_HAS_EXP_ICON_SZ;
                    CShellLink_RemoveDataBlock_body(this, EXP_SZ_ICON_SIG);
                    pInfo = core::ptr::null_mut();
                }
            }
            if pInfo.is_null() {
                buffer.cbSize = core::mem::size_of::<EXP_SZ_LINK>() as u32;
                buffer.dwSignature = EXP_SZ_ICON_SIG;
                pInfo = &raw mut buffer;
            }

            lstrcpynW(
                core::ptr::addr_of_mut!((*pInfo).szwTarget).cast(),
                pszIconPath,
                (core::ptr::addr_of!((*pInfo).szwTarget) as *const [WCHAR]).len(),
            );
            WideCharToMultiByte(
                CP_ACP,
                0,
                pszIconPath,
                -1,
                (*pInfo).szTarget.as_mut_ptr(),
                ((*pInfo).szTarget).len(),
                core::ptr::null(),
                core::ptr::null_mut(),
            );

            hr = S_OK;
            if pInfo == &raw mut buffer {
                hr = CShellLink_AddDataBlock_body(this, pInfo as *mut core::ffi::c_void);
            }
            if hr == S_OK {
                (*this).m_Header.dwFlags |= SLDF_HAS_EXP_ICON_SZ;
            }

            pszIconPath = old_pszIconPath;
        } else {
            (*this).m_Header.dwFlags &= !SLDF_HAS_EXP_ICON_SZ;
            CShellLink_RemoveDataBlock_body(this, EXP_SZ_ICON_SIG);
        }
    }

    if !pszIconPath.is_null() {
        (*this).m_Header.dwFlags &= !SLDF_HAS_ICONLOCATION;
        HeapFree(
            GetProcessHeap(),
            0,
            (*this).m_sIcoPath as *mut _,
        );
        (*this).m_sIcoPath = strdupW_body(pszIconPath);
        if (*this).m_sIcoPath.is_null() {
            return E_OUTOFMEMORY;
        }
        (*this).m_Header.dwFlags |= SLDF_HAS_ICONLOCATION;
    }

    hr = S_OK;
    (*this).m_Header.nIconIndex = (iIcon as DWORD) as i32;
    (*this).m_bDirty = TRUE;
    hr
}

pub unsafe fn CShellLink_SetRelativePath_v2_body(this: *mut CShellLink, pszPathRel: LPCWSTR, dwReserved: DWORD) -> HRESULT {
    // Free existing relative path
    HeapFree(GetProcessHeap(), 0, (*this).m_sPathRel);
    (*this).m_sPathRel = core::ptr::null_mut();

    // Copy new relative path if provided
    if !pszPathRel.is_null() {
        (*this).m_sPathRel = strdupW_body(pszPathRel);
        if (*this).m_sPathRel.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    // Mark dirty and update path
    (*this).m_bDirty = TRUE;
    ShellLink_UpdatePath_body(
        (*this).m_sPathRel,
        (*this).m_sPath,
        (*this).m_sWorkDir,
        &raw mut (*this).m_sPath,
    )
}

pub unsafe fn GetAdvertisedArg_body(str: LPCWSTR) -> LPWSTR {
    if str.is_null() {
        return core::ptr::null_mut();
    }

    let p = wcschr(str, 0x003Au16);
    if p.is_null() {
        return core::ptr::null_mut();
    }

    let len = ((p as usize) - (str as usize)) / core::mem::size_of::<u16>();
    let size = (len + 1) * core::mem::size_of::<u16>();
    let ret = HeapAlloc(GetProcessHeap(), 0, size) as LPWSTR;
    if ret.is_null() {
        return ret;
    }

    memcpy(ret as *mut _, str as *const _, (len as usize) * core::mem::size_of::<u16>());
    *ret.add(len as usize) = 0;
    ret
}

pub unsafe fn CShellLink_WriteAdvertiseInfo_body(this: *mut CShellLink, string: LPCWSTR, dwSig: DWORD) -> HRESULT {
    let mut buffer: EXP_DARWIN_LINK = core::mem::zeroed();
    let mut pInfo: *mut EXP_DARWIN_LINK;

    // CShellLink.cpp:2102 `&& (dwSig != EXP_LOGO3_ID_SIG)` is inside an excluded NTDDI block, so
    // a Logo3 signature is rejected here exactly as the real build rejects it.
    if dwSig != EXP_DARWIN_ID_SIG {
        return E_INVALIDARG;
    }
    if string.is_null() {
        return S_FALSE;
    }

    pInfo = SHFindDataBlock((*this).m_pDBList, dwSig) as *mut EXP_DARWIN_LINK;
    if !pInfo.is_null() {
        if (*pInfo).dbh.cbSize as usize != core::mem::size_of::<EXP_DARWIN_LINK>() {
            if dwSig == EXP_DARWIN_ID_SIG {
                (*this).m_Header.dwFlags &= !SLDF_HAS_DARWINID;
            }
            // (CShellLink.cpp:2124-2127, the Logo3 branch, is an excluded NTDDI block.)
            CShellLink_RemoveDataBlock_body(this, dwSig);
            pInfo = core::ptr::null_mut();
        }
    }
    if pInfo.is_null() {
        buffer.dbh.cbSize = core::mem::size_of::<EXP_DARWIN_LINK>() as u32;
        buffer.dbh.dwSignature = dwSig;
        pInfo = &raw mut buffer;
    }

    lstrcpynW(core::ptr::addr_of_mut!((*pInfo).szwDarwinID).cast(), string, (core::ptr::addr_of!((*pInfo).szwDarwinID) as *const [WCHAR]).len());
    WideCharToMultiByte(CP_ACP, 0, string, -(1 as i32),
                            (*pInfo).szDarwinID.as_mut_ptr(), ((*pInfo).szDarwinID).len(),
                            core::ptr::null_mut(), core::ptr::null_mut());

    let mut hr: i32 = S_OK;
    if core::ptr::eq(pInfo, &raw mut buffer) {
        hr = CShellLink_AddDataBlock_body(this, pInfo as *mut core::ffi::c_void);
    }
    if hr == S_OK {
        if dwSig == EXP_DARWIN_ID_SIG {
            (*this).m_Header.dwFlags |= SLDF_HAS_DARWINID;
        }
        // (CShellLink.cpp:2153-2156, the Logo3 branch, is an excluded NTDDI block.)
    }
    hr
}

pub unsafe fn CShellLink_SetAdvertiseInfo_body(this: *mut CShellLink, str: LPCWSTR) -> HRESULT {
    let mut hr: HRESULT;
    let mut szComponent: LPCWSTR = core::ptr::null();
    let mut szProduct: LPCWSTR = core::ptr::null();
    let mut p: LPCWSTR;
    let mut len: INT;
    let mut guid: GUID = core::mem::zeroed();
    let mut szGuid: [WCHAR; 38 + 1] = [0; 38 + 1];

    (*this).sProduct = core::ptr::null_mut();
    (*this).sComponent = core::ptr::null_mut();

    let mut str = str;
    while *str.offset(0) != 0 {
        if *str.offset(0) != 0x003A || *str.offset(1) != 0x003A {
            return E_FAIL;
        }

        if *str.offset(2) == 0 {
            break;
        }
        str = str.offset(2);

        p = wcschr(str, 0x003Au16);
        if p.is_null() {
            return E_FAIL;
        }
        len = p.offset_from(str) as INT;
        if len != 38 {
            return E_FAIL;
        }

        memcpy(szGuid.as_mut_ptr() as *mut core::ffi::c_void, str as *const core::ffi::c_void, (core::mem::size_of::<WCHAR>() * len as usize) as usize);
        szGuid[len as usize] = 0;

        hr = CLSIDFromString(szGuid.as_ptr(), &mut guid);
        if hr != S_OK {
            return hr;
        }
        str = p.offset(1);

        if IsEqualGUID(guid, SHELL32_AdvtShortcutComponent) && szComponent.is_null() {
            szComponent = str;
        } else if IsEqualGUID(guid, SHELL32_AdvtShortcutProduct) && szProduct.is_null() {
            szProduct = str;
        } else {
            return E_FAIL;
        }

        str = wcschr(str, 0x003Au16);
        if str.is_null() {
            return E_FAIL;
        }
    }

    if szComponent.is_null() {
        return E_FAIL;
    }

    szComponent = GetAdvertisedArg_body(szComponent);
    szProduct = GetAdvertisedArg_body(szProduct);

    hr = CShellLink_WriteAdvertiseInfo_body(this, szComponent, EXP_DARWIN_ID_SIG);
    // if (FAILED(hr)) return hr;  (commented out in the C++ too)
    // CShellLink.cpp:2224-2228 writes the Logo3 PRODUCT block — an excluded NTDDI block, so the
    // real build never writes it and neither does this. szProduct is still parsed and freed:
    // that part of the C is outside the #if.
    let _ = hr;

    HeapFree(GetProcessHeap(), 0, szComponent as PVOID);
    HeapFree(GetProcessHeap(), 0, szProduct as PVOID);

    if false { // TRACE_ON(shell): debug channel off
        CShellLink_GetAdvertiseInfo_body(this, core::ptr::addr_of_mut!((*this).sComponent), EXP_DARWIN_ID_SIG);
    }

    return S_OK;
}

pub unsafe fn CShellLink_SetTargetFromPIDLOrPath_body(this: *mut CShellLink, pidl: LPCITEMIDLIST, pszFile: LPCWSTR) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut pidlNew: LPITEMIDLIST = core::ptr::null_mut();
    let mut szPath: [u16; MAX_PATH as usize] =
        [0u16; MAX_PATH as usize];
    let mut pszFile = pszFile;

    if !pidl.is_null() && !pszFile.is_null() {
        return E_FAIL;
    }

    if !pidl.is_null() {
        pidlNew = ILClone(pidl);
        if pidlNew.is_null() {
            return E_FAIL;
        }
    } else if !pszFile.is_null() {
        hr = SHILCreateFromPathW(pszFile, &mut pidlNew, core::ptr::null_mut());
        if hr < 0 {
            StringCchCopyW(
                szPath.as_mut_ptr(),
                MAX_PATH,
                pszFile,
            );
            PathResolveW(
                szPath.as_mut_ptr(),
                core::ptr::null_mut(),
                PRF_TRYPROGRAMEXTENSIONS,
            );
            if PathIsFileSpecW(szPath.as_ptr()) != 0 {
                hr = E_INVALIDARG;
                szPath[0 as usize] = 0;
            } else {
                hr = S_OK;
                pidlNew = SHSimpleIDListFromPathW(szPath.as_ptr());
            }
        }
    }

    ILFree((*this).m_pPidl);
    (*this).m_pPidl = pidlNew;

    if pszFile.is_null() {
        if SHGetPathFromIDListW(pidlNew, szPath.as_mut_ptr()) != 0 {
            pszFile = szPath.as_ptr();
        }
    }

    if pszFile.is_null() {
        szPath[0] = 0x0000u16;
        pszFile = szPath.as_ptr();
    }

    ShellLink_GetVolumeInfo_body(pszFile, &mut (*this).volume as *mut _);

    if !(*this).m_sPath.is_null() {
        HeapFree(GetProcessHeap(), 0, (*this).m_sPath);
    }

    (*this).m_sPath = strdupW_body(pszFile);
    if (*this).m_sPath.is_null() {
        return E_OUTOFMEMORY;
    }

    (*this).m_bDirty = TRUE;
    hr
}

pub unsafe fn CShellLink_SetPath_v2_body(this: *mut CShellLink, pszFile: LPCWSTR) -> HRESULT {
    let mut unquoted: LPWSTR = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    if pszFile.is_null() {
        return E_INVALIDARG;
    }
    if ((*this).m_Header.dwFlags & SLDF_HAS_DARWINID) != 0 {
        return S_FALSE;
    }

    let len: usize = wcslen(pszFile);
    let mut pszFile_mut: LPCWSTR = pszFile;
    if *pszFile_mut.add(0) == 0x0022u16
        && *pszFile_mut.add(len.wrapping_sub(1)) == 0x0022u16
    {
        let copy = strdupW_body(pszFile_mut);
        unquoted = copy;
        PathUnquoteSpacesW(copy);
        pszFile_mut = copy;
    }

    if !wcschr(pszFile_mut, 0x0022u16).is_null() {
        hr = S_FALSE;
    } else {
        HeapFree(GetProcessHeap(), 0, (*this).m_sPath);
        (*this).m_sPath = core::ptr::null_mut();

        if CShellLink_SetAdvertiseInfo_body(this, pszFile_mut) != S_OK {
            let mut szPath: [u16; MAX_PATH as usize] =
                [0u16; MAX_PATH as usize];
            if *pszFile_mut != 0 {
                SHExpandEnvironmentStringsW(
                    pszFile_mut,
                    szPath.as_mut_ptr(),
                    szPath.len(),
                );
            } else {
                *szPath.as_mut_ptr() = 0x0000u16;
            }

            if *pszFile_mut != 0
                && (wcscmp(pszFile_mut, szPath.as_mut_ptr())
                    != 0)
            {
                let mut buffer: EXP_SZ_LINK = core::mem::zeroed();
                let mut pInfo: *mut EXP_SZ_LINK = SHFindDataBlock(
                    (*this).m_pDBList,
                    EXP_SZ_LINK_SIG,
                ) as *mut EXP_SZ_LINK;

                if !pInfo.is_null() {
                    if (*pInfo).cbSize != core::mem::size_of::<EXP_SZ_LINK>() as u32 {
                        (*this).m_Header.dwFlags &= !SLDF_HAS_EXP_SZ;
                        CShellLink_RemoveDataBlock_body(this, EXP_SZ_LINK_SIG);
                        pInfo = core::ptr::null_mut();
                    }
                }

                if pInfo.is_null() {
                    pInfo = &mut buffer as *mut EXP_SZ_LINK;
                    (*pInfo).cbSize = core::mem::size_of::<EXP_SZ_LINK>() as u32;
                    (*pInfo).dwSignature = EXP_SZ_LINK_SIG;
                }

                lstrcpynW(
                    core::ptr::addr_of_mut!((*pInfo).szwTarget).cast(),
                    pszFile_mut,
                    (core::ptr::addr_of!((*pInfo).szwTarget) as *const [WCHAR]).len() as usize,
                );
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    pszFile_mut,
                    -1,
                    (*pInfo).szTarget.as_mut_ptr(),
                    ((*pInfo).szTarget).len() as usize,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                hr = S_OK;
                if pInfo == &mut buffer as *mut EXP_SZ_LINK {
                    hr = CShellLink_AddDataBlock_body(this, pInfo as *mut core::ffi::c_void);
                    if hr == S_OK {
                        (*this).m_Header.dwFlags |= SLDF_HAS_EXP_SZ;
                    }
                }
                pszFile_mut = szPath.as_ptr();
            } else {
                (*this).m_Header.dwFlags &= !SLDF_HAS_EXP_SZ;
                CShellLink_RemoveDataBlock_body(this, EXP_SZ_LINK_SIG);
            }

            hr = CShellLink_SetTargetFromPIDLOrPath_body(this, core::ptr::null_mut(), pszFile_mut);
        }
    }

    (*this).m_bDirty = TRUE;

    if !unquoted.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            unquoted,
        );
    }

    hr
}

pub unsafe fn CShellLink_AddDataBlock_body(this: *mut CShellLink, pDataBlock: *mut core::ffi::c_void) -> HRESULT {
    if SHAddDataBlock(&mut (*this).m_pDBList, pDataBlock as *mut DATABLOCK_HEADER) != 0 {
        (*this).m_bDirty = TRUE;
        return S_OK;
    }
    S_FALSE
}

pub unsafe fn CShellLink_CopyDataBlock_body(this: *mut CShellLink, dwSig: DWORD, ppDataBlock: *mut *mut core::ffi::c_void) -> HRESULT {
    *ppDataBlock = core::ptr::null_mut();

    let pBlock: *mut DATABLOCK_HEADER =
        SHFindDataBlock((*this).m_pDBList, dwSig);
    if pBlock.is_null() {
        return E_FAIL;
    }

    let pDataBlock = LocalAlloc(LMEM_ZEROINIT, (*pBlock).cbSize as SIZE_T);
    if pDataBlock.is_null() {
        return E_OUTOFMEMORY;
    }

    CopyMemory(pDataBlock, pBlock, (*pBlock).cbSize);

    *ppDataBlock = pDataBlock;
    S_OK
}

pub unsafe fn CShellLink_RemoveDataBlock_body(this: *mut CShellLink, dwSig: DWORD) -> HRESULT {
    if SHRemoveDataBlock(core::ptr::addr_of_mut!((*this).m_pDBList), dwSig) != 0 {
        (*this).m_bDirty = TRUE;
        return S_OK;
    }
    return S_FALSE;
}

pub unsafe fn CShellLink_GetFlags_body(this: *mut CShellLink, pdwFlags: *mut DWORD) -> HRESULT {
    *pdwFlags = (*this).m_Header.dwFlags;
    S_OK
}

pub unsafe fn CShellLink_SetFlags_body(this: *mut CShellLink, dwFlags: DWORD) -> HRESULT {
    if (*this).m_Header.dwFlags == dwFlags {
        return S_FALSE;
    }
    (*this).m_Header.dwFlags = dwFlags;
    (*this).m_bDirty = TRUE;
    S_OK
}

pub unsafe fn CShellLink_Initialize_body(this: *mut CShellLink, pidlFolder: PCIDLIST_ABSOLUTE, pdtobj: *mut IDataObject, hkeyProgID: HKEY) -> HRESULT {
    if pdtobj.is_null() {
        return E_FAIL;
    }

    let mut format: FORMATETC = core::mem::zeroed();
    format.cfFormat = CF_HDROP as u16;
    format.ptd = core::ptr::null_mut();
    format.dwAspect = DVASPECT_CONTENT;
    format.lindex = -( 1 as i32 );
    format.tymed = TYMED_HGLOBAL;

    let mut stgm: STGMEDIUM = core::mem::zeroed();
    let hr = ((*(*pdtobj).lpVtbl).GetData)(pdtobj, &mut format, &mut stgm);
    if hr < 0 {
        return hr;
    }

    let mut count: UINT = DragQueryFileW(
        stgm.__bindgen_anon_1.hGlobal as HDROP,
        -( 1 as i32 ) as UINT,
        core::ptr::null_mut(),
        0,
    );
    if count == 1 as UINT {
        count = DragQueryFileW(
            stgm.__bindgen_anon_1.hGlobal as HDROP,
            0,
            core::ptr::null_mut(),
            0,
        );
        count += 1;
        let path = HeapAlloc(
            GetProcessHeap(),
            0,
            (count as usize) * core::mem::size_of::<u16>(),
        ) as *mut u16;
        if !path.is_null() {
            count = DragQueryFileW(
                stgm.__bindgen_anon_1.hGlobal as HDROP,
                0,
                path,
                count,
            );
            let _ = CShellLink_Load_v1_body(
                this,
                path as LPOLESTR,
                0,
            );
            HeapFree(
                GetProcessHeap(),
                0,
                path as *mut c_void,
            );
        }
    }
    ReleaseStgMedium(&mut stgm as *mut STGMEDIUM);
    return S_OK;
}

pub unsafe fn CShellLink_QueryContextMenu_body(this: *mut CShellLink, hMenu: HMENU, indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
    let mut indexMenu = indexMenu;
    let mut id: INT = 0;

    if hMenu.is_null() {
        return E_INVALIDARG;
    }

    let mut str_open = CStringW::from_resource(IDS_OPEN_VERB);
    let mut str_open_file_loc = CStringW::from_resource(IDS_OPENFILELOCATION);

    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    ZeroMemory(&mut mii as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<MENUITEMINFOW>());
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_TYPE | MIIM_ID | MIIM_STATE;
    mii.dwTypeData = str_open.as_mut_ptr();
    mii.cch = wcslen(str_open.as_ptr()) as UINT;
    let index_menu_1 = indexMenu;
    indexMenu += 1;
    mii.wID = idCmdFirst + id as UINT;
    id += 1;
    mii.fState = MFS_DEFAULT | MFS_ENABLED;
    mii.fType = MFT_STRING;
    if InsertMenuItemW(hMenu, index_menu_1, TRUE, &mii) == 0 {
        return E_FAIL;
    }

    mii.fMask = MIIM_TYPE | MIIM_ID | MIIM_STATE;
    mii.dwTypeData = str_open_file_loc.as_mut_ptr();
    mii.cch = wcslen(str_open_file_loc.as_ptr()) as UINT;
    let index_menu_2 = indexMenu;
    indexMenu += 1;
    mii.wID = idCmdFirst + id as UINT;
    id += 1;
    mii.fState = MFS_ENABLED;
    mii.fType = MFT_STRING;
    if InsertMenuItemW(hMenu, index_menu_2, TRUE, &mii) == 0 {
        return E_FAIL;
    }

    MAKE_HRESULT(SEVERITY_SUCCESS, 0, id as DWORD)
}

pub unsafe fn CShellLink_DoOpenFileLocation_body(this: *mut CShellLink) -> HRESULT {
    let mut szParams: [u16; (MAX_PATH + 64) as usize] =
        [0u16; (MAX_PATH + 64) as usize];

    StringCbPrintfW1(
        szParams.as_mut_ptr(),
        core::mem::size_of_val(&szParams),
        CPP_COM_WSTR_08CDC276CF8F.as_ptr(),
        (*this).m_sPath,
    );

    let ret = ShellExecuteW(
        core::ptr::null_mut(),
        core::ptr::null(),
        CPP_COM_WSTR_AB68B18B313D.as_ptr(),
        szParams.as_ptr(),
        core::ptr::null(),
        (*this).m_Header.nShowCommand as INT,
    ) as INT_PTR;

    if ret <= 32 {
        return E_FAIL;
    }

    S_OK
}

pub unsafe fn CShellLink_InvokeCommand_body(this: *mut CShellLink, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    if ((*lpici).cbSize as usize) < core::mem::size_of::<CMINVOKECOMMANDINFO>() {
        return E_INVALIDARG;
    }

    let hr = CShellLink_Resolve_body(
        this,
        (*lpici).hwnd,
        if ((*lpici).fMask & CMIC_MASK_FLAG_NO_UI) != 0 {
            SLR_NO_UI
        } else {
            0
        },
    );
    if hr < 0 {
        return hr;
    }

    let idCmd = LOWORD((*lpici).lpVerb as usize) as UINT;

    match idCmd {
        IDCMD_OPEN => CShellLink_DoOpen_body(this, lpici),
        IDCMD_OPENFILELOCATION => CShellLink_DoOpenFileLocation_body(this),
        _ => E_NOTIMPL,
    }
}

pub unsafe fn CShellLink_DoOpen_body(this: *mut CShellLink, lpici: LPCMINVOKECOMMANDINFO) -> HRESULT {
    let iciex = lpici as *const CMINVOKECOMMANDINFOEX;
    let unicode = IsUnicode(lpici);

    // CStringW args; (C++ CShellLink.cpp:2633-2653)
    let mut args = CStringW::new();
    if !(*this).m_sArgs.is_null() {
        args += (*this).m_sArgs as LPCWSTR;
    }
    if unicode {
        if !StrIsNullOrEmptyW((*iciex).lpParametersW) {
            args += 0x0020u16; // L' '
            args += (*iciex).lpParametersW;
        }
    } else {
        // CComHeapPtr<WCHAR> pszParams: freed at scope end via CoTaskMemFree
        let mut pszParams: *mut u16 = core::ptr::null_mut();
        if !StrIsNullOrEmptyA((*lpici).lpParameters)
            && !__SHCloneStrAtoW(&mut pszParams, (*lpici).lpParameters).is_null()
        {
            args += 0x0020u16; // L' '
            args += pszParams as LPCWSTR;
        }
        if !pszParams.is_null() {
            CoTaskMemFree(pszParams as *mut core::ffi::c_void);
        }
    }

    const DIR_SIZE: usize = MAX_PATH as usize;
    let mut dir: [u16; DIR_SIZE] = [0; DIR_SIZE];
    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    sei.fMask = SEE_MASK_HASLINKNAME | SEE_MASK_UNICODE | SEE_MASK_DOENVSUBST |
                ((*lpici).fMask & (SEE_MASK_NOASYNC | SEE_MASK_ASYNCOK | SEE_MASK_FLAG_NO_UI));
    sei.lpDirectory = (*this).m_sWorkDir;
    if !(*this).m_pPidl.is_null() {
        sei.lpIDList = (*this).m_pPidl as LPVOID;
        sei.fMask |= SEE_MASK_INVOKEIDLIST;
    } else {
        sei.lpFile = (*this).m_sPath;
        if ((*this).m_Header.dwFlags & SLDF_HAS_EXP_SZ) == 0 {
            sei.fMask &= !SEE_MASK_DOENVSUBST;
            if !(*this).m_sWorkDir.is_null()
                && ExpandEnvironmentStringsW((*this).m_sWorkDir, dir.as_mut_ptr(), DIR_SIZE as DWORD) <= DIR_SIZE as DWORD
            {
                sei.lpDirectory = dir.as_ptr();
            }
        }
    }
    sei.lpParameters = args.as_ptr();
    sei.lpClass = (*this).m_sLinkPath;
    sei.nShow = (*this).m_Header.nShowCommand as INT;
    if (*lpici).nShow != SW_SHOWNORMAL && (*lpici).nShow != SW_SHOW {
        sei.nShow = (*lpici).nShow;
    }

    // Use invoker's working directory if the link did not specify one
    if StrIsNullOrEmptyW(sei.lpDirectory) || PathEnvSubstIsDirectory_body(sei.lpDirectory) == 0 {
        let pszDirA = (*lpici).lpDirectory;
        if unicode && !StrIsNullOrEmptyW((*iciex).lpDirectoryW) {
            sei.lpDirectory = (*iciex).lpDirectoryW;
        } else if !pszDirA.is_null() && SHAnsiToUnicode(pszDirA, dir.as_mut_ptr(), DIR_SIZE as i32) != 0 {
            sei.lpDirectory = dir.as_ptr();
        }
    }

    sei.dwHotKey = (*lpici).dwHotKey;
    sei.fMask |= CmicFlagsToSeeFlags((*lpici).fMask & CMIC_MASK_HOTKEY);
    if (*this).m_Header.wHotKey != 0 {
        sei.dwHotKey = (*this).m_Header.wHotKey as DWORD;
        sei.fMask |= SEE_MASK_HOTKEY;
    }

    let result = ShellExecuteExW((&raw mut sei).cast());
    if result != 0 {
        S_OK
    } else {
        E_FAIL
    }
}

pub unsafe fn CShellLink_GetCommandString_body(this: *mut CShellLink, idCmd: UINT_PTR, uType: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn GetTypeDescriptionByPath_body(pszFullPath: PCWSTR, fAttributes: DWORD, szBuf: PWSTR, cchBuf: UINT) -> () {
    let mut fAttributes = fAttributes;
    if fAttributes == INVALID_FILE_ATTRIBUTES && unsafe { PathFileExistsAndAttributesW(pszFullPath, &mut fAttributes) } == 0 {
        fAttributes = 0;
    }

    let mut fi: SHFILEINFOW = unsafe { core::mem::zeroed() };
    if unsafe {
        SHGetFileInfoW(
            pszFullPath,
            fAttributes,
            &mut fi,
            core::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
        )
    } == 0 {
        fi.szTypeName[0 as usize] = UNICODE_NULL;
    }

    let fFolder = (fAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
    let pwszExt: *const u16 = if fFolder {
        CPP_COM_WSTR_96A296D224F2.as_ptr()
    } else {
        unsafe { PathFindExtensionW(pszFullPath) }
    };

    if unsafe { *pwszExt.offset(0 as isize) } != 0 {
        if fi.szTypeName[0 as usize] == 0 {
            unsafe {
                cpp_com_string_cch_printf_w(
                    szBuf,
                    cchBuf as usize,
                    CPP_COM_WSTR_FB47CCC40FE6.as_ptr(),
                    &[pwszExt.offset(1 as isize)],
                );
            }
        } else {
            unsafe {
                cpp_com_string_cch_printf_w(
                    szBuf,
                    cchBuf as usize,
                    CPP_COM_WSTR_AF854C3D2274.as_ptr(),
                    &[core::ptr::addr_of!(fi.szTypeName).cast(), pwszExt],
                );
            }
        }
    } else {
        unsafe {
            cpp_com_string_cch_printf_w(
                szBuf,
                cchBuf as usize,
                CPP_COM_WSTR_FB47CCC40FE6.as_ptr(),
                &[core::ptr::addr_of!(fi.szTypeName).cast()],
            );
        }
    }
}

pub unsafe fn CShellLink_OnInitDialog_body(this: *mut CShellLink, hwndDlg: HWND, hwndFocus: HWND, lParam: LPARAM) -> BOOL {
    CShellLink_Resolve_body(this, core::ptr::null_mut(), SLR_NO_UI | SLR_NOUPDATE | SLR_NOSEARCH | SLR_NOTRACK);

    (*this).m_bInInit = TRUE;

    let darwin: u32 = (*this).m_Header.dwFlags & (SLDF_HAS_DARWINID as u32);

    // Get file information for icon
    let mut fi: SHFILEINFOW = core::mem::zeroed();
    if SHGetFileInfoW((*this).m_sLinkPath as *const u16, 0, &mut fi as *mut SHFILEINFOW, core::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_TYPENAME | SHGFI_ICON) == 0 {
        fi.szTypeName[0] = 0x0000u16;
        fi.hIcon = core::ptr::null_mut();
    }

    if !fi.hIcon.is_null() {
        if !(*this).m_hIcon.is_null() {
            DestroyIcon(((*this).m_hIcon).cast());
        }
        (*this).m_hIcon = fi.hIcon as HICON;
        SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_ICON, STM_SETICON, (*this).m_hIcon as WPARAM, 0);
    } else {
        // ERR trace omitted (keep call token if present, but there's no token; assumed no call)
    }

    // Get display name
    if SHGetFileInfoW((*this).m_sLinkPath as *const u16, 0, &mut fi as *mut SHFILEINFOW, core::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_DISPLAYNAME) == 0 {
        fi.szDisplayName[0] = UNICODE_NULL;
    }
    SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_TEXT, core::ptr::addr_of!(fi.szDisplayName).cast());

    // Target type
    if !(*this).m_sPath.is_null() {
        let mut buf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        GetTypeDescriptionByPath_body((*this).m_sPath as *const u16, (*this).m_Header.dwFileAttributes, buf.as_mut_ptr(), buf.len() as UINT);
        SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_TYPE_EDIT, buf.as_ptr());
    }

    // Target location
    if !(*this).m_sPath.is_null() {
        let mut target: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        StringCchCopyW(target.as_mut_ptr(), target.len(), (*this).m_sPath as *const u16);
        PathRemoveFileSpecW(target.as_mut_ptr());
        SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_LOCATION_EDIT, PathFindFileNameW(target.as_mut_ptr()));
    }

    // Target path
    if !(*this).m_sPath.is_null() {
        let mut newpath: [u16; (MAX_PATH as usize) * (2 as usize)] = [0u16; (MAX_PATH as usize) * (2 as usize)];
        newpath[0] = UNICODE_NULL; // Note: original index is 0
        if !wcschr((*this).m_sPath as *const u16, 0x0020u16).is_null() {
            // StringCchPrintfExW(newpath, _countof, NULL, NULL, 0, L"\"%ls\"", m_sPath)
            cpp_com_string_cch_printf_w(newpath.as_mut_ptr(), newpath.len(), CPP_COM_WSTR_9F7F17012B9B.as_ptr(), &[(*this).m_sPath as LPCWSTR]);
        } else {
            // StringCchCopyExW(newpath, _countof, m_sPath, NULL, NULL, 0)
            StringCchCopyW(newpath.as_mut_ptr(), newpath.len(), (*this).m_sPath as LPCWSTR);
        }

        if !(*this).m_sArgs.is_null() && (*(*this).m_sArgs) != 0 {
            StringCchCatW(newpath.as_mut_ptr(), newpath.len(), CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
            StringCchCatW(newpath.as_mut_ptr(), newpath.len(), (*this).m_sArgs as *const u16);
        }
        SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_TARGET_TEXT, newpath.as_ptr());
    }

    // Working directory
    if !(*this).m_sWorkDir.is_null() {
        SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_START_IN_EDIT, (*this).m_sWorkDir as *const u16);
    }

    // Description
    if !(*this).m_sDescription.is_null() {
        SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_COMMENT_EDIT, (*this).m_sDescription as *const u16);
    }

    // Hot key
    SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_KEY_HOTKEY, HKM_SETHOTKEY, (*this).m_Header.wHotKey as WPARAM, 0);

    // Run dropdown
    let runstrings: [UINT; 3] = [IDS_SHORTCUT_RUN_NORMAL, IDS_SHORTCUT_RUN_MIN, IDS_SHORTCUT_RUN_MAX];
    let runshowcmd: [DWORD; 3] = [SW_SHOWNORMAL as DWORD, SW_SHOWMINNOACTIVE as DWORD, SW_SHOWMAXIMIZED as DWORD];
    let hRunCombo: HWND = GetDlgItem(hwndDlg, IDC_SHORTCUT_RUN_COMBO);

    for i in 0..runstrings.len() {
        let mut buf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        if LoadStringW(shell32_hInstance, runstrings[i], buf.as_mut_ptr(), buf.len() as INT) == 0 {
            break;
        }
        let index: i32 = SendMessageW(hRunCombo, CB_ADDSTRING, 0, buf.as_ptr() as LPARAM) as i32;
        if index < 0 {
            continue;
        }
        SendMessageW(hRunCombo, CB_SETITEMDATA, index as WPARAM, runshowcmd[i] as LPARAM);
        if i == 0 || (*this).m_Header.nShowCommand == runshowcmd[i] {
            SendMessageW(hRunCombo, CB_SETCURSEL, index as WPARAM, 0);
        }
    }

    let mut disablecontrols: BOOL = FALSE;
    if darwin != 0 {
        disablecontrols = TRUE;
        EnableWindow(GetDlgItem(hwndDlg, IDC_SHORTCUT_FIND), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_SHORTCUT_CHANGE_ICON), FALSE);
    } else {
        let mut path: [u16; (MAX_PATH as usize) * (2 as usize)] = [0u16; (MAX_PATH as usize) * (2 as usize)];
        path[0] = UNICODE_NULL;
        let mut hr: HRESULT = CShellLink_GetPath_v2_body(this, path.as_mut_ptr(), path.len() as i32, core::ptr::null_mut(), SLGP_RAWPATH);
        if hr < 0 as HRESULT {
            hr = CShellLink_GetPath_v2_body(this, path.as_mut_ptr(), path.len() as i32, core::ptr::null_mut(), 0);
        }
        if GetKeyState(VK_CONTROL) < 0 {
            hr = S_OK;
            path[0] = UNICODE_NULL;
        }
        if hr != S_OK {
            disablecontrols = TRUE;
            let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
            if CShellLink_GetIDList_body(this, &mut pidl) == S_OK {
                path[0] = UNICODE_NULL;
                SHGetNameAndFlagsW(pidl, SHGDN_FORADDRESSBAR | SHGDN_FORPARSING, path.as_mut_ptr(), path.len() as UINT, core::ptr::null_mut());
                SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_TYPE_EDIT, path.as_ptr());
                SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_TARGET_TEXT, path.as_ptr());
                ILRemoveLastID(pidl);
                path[0] = UNICODE_NULL;
                SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_START_IN_EDIT, path.as_ptr());
                SHGetNameAndFlagsW(pidl, SHGDN_NORMAL, path.as_mut_ptr(), path.len() as UINT, core::ptr::null_mut());
                SetDlgItemTextW(hwndDlg, IDC_SHORTCUT_LOCATION_EDIT, path.as_ptr());
                ILFree(pidl);
            }
            EnableWindow(GetDlgItem(hwndDlg, IDC_SHORTCUT_ADVANCED), FALSE);
            EnableWindow(GetDlgItem(hwndDlg, IDC_SHORTCUT_START_IN_EDIT), FALSE);
        } else {
            let valid: bool = hr < 0
                || !(path[0] == b':' as u16 && path[1] == b':' as u16 && path[2] == b'{' as u16);
            ASSERT!(valid);
        }
    }

    let hWndTarget: HWND = GetDlgItem(hwndDlg, IDC_SHORTCUT_TARGET_TEXT);
    EnableWindow(hWndTarget, (disablecontrols == 0) as BOOL);
    PostMessage(hWndTarget, EM_SETSEL, 0, -1);

    SHAutoComplete(hWndTarget, SHACF_DEFAULT);
    SHAutoComplete(GetDlgItem(hwndDlg, IDC_SHORTCUT_START_IN_EDIT), SHACF_DEFAULT);

    (*this).m_bInInit = FALSE;
    return TRUE;
}

pub unsafe fn CShellLink_OnCommand_body(this: *mut CShellLink, hwndDlg: HWND, id: i32, hwndCtl: HWND, codeNotify: UINT) -> () {
    match id {
        IDC_SHORTCUT_FIND => {
            SHOpenFolderAndSelectItems((*this).m_pPidl, 0, core::ptr::null_mut(), 0);
            return;
        }
        IDC_SHORTCUT_CHANGE_ICON => {
            let mut fi: SHFILEINFOW = core::mem::zeroed();
            let mut IconIndex: i32 = (*this).m_Header.nIconIndex as i32;
            let mut wszPath: [u16; MAX_PATH] = [0; MAX_PATH];
            wszPath[0] = UNICODE_NULL;

            if !StrIsNullOrEmptyW((*this).m_sIcoPath) {
                let mut pszPath: PWSTR = (*this).m_sIcoPath;
                if *((*this).m_sIcoPath) == '.' as u16 {
                    if (cpp_com_string_cch_printf_w(wszPath.as_mut_ptr(), wszPath.len(), CPP_COM_WSTR_AAA8350805D2.as_ptr(), &[(*this).m_sIcoPath as LPCWSTR]) >= 0)
                        && SHGetFileInfoW(wszPath.as_mut_ptr(), 0, &mut fi, core::mem::size_of::<SHFILEINFOW>() as UINT, SHGFI_ICONLOCATION | SHGFI_USEFILEATTRIBUTES) != 0
                    {
                        pszPath = core::ptr::addr_of_mut!(fi.szDisplayName).cast();
                        IconIndex = fi.iIcon;
                    }
                }
                if StringCchCopyW(wszPath.as_mut_ptr(), wszPath.len() as UINT, pszPath) < 0 {
                    wszPath[0] = UNICODE_NULL;
                }
            } else if !StrIsNullOrEmptyW((*this).m_sPath) {
                FindExecutableW((*this).m_sPath, core::ptr::null_mut(), wszPath.as_mut_ptr());
            }

            if wszPath[0] == 0 && !(*this).m_pPidl.is_null() {
                if SHGetFileInfoW((*this).m_pPidl as PWSTR, 0, &mut fi, core::mem::size_of::<SHFILEINFOW>() as UINT, SHGFI_ICONLOCATION | SHGFI_PIDL) != 0
                    && StringCchCopyW(wszPath.as_mut_ptr(), wszPath.len() as UINT, core::ptr::addr_of!(fi.szDisplayName).cast()) >= 0
                {
                    IconIndex = fi.iIcon;
                }
            }

            if PickIconDlg(hwndDlg, wszPath.as_mut_ptr(), wszPath.len() as UINT, &mut IconIndex) != 0 {
                CShellLink_SetIconLocation_v2_body(this, wszPath.as_mut_ptr(), IconIndex);
                PropSheet_Changed(GetParent(hwndDlg), hwndDlg);

                let hIconLarge = CShellLink_CreateShortcutIcon_body(this, wszPath.as_ptr() as LPCWSTR, IconIndex);
                if !hIconLarge.is_null() {
                    if !(*this).m_hIcon.is_null() {
                        DestroyIcon(((*this).m_hIcon).cast());
                    }
                    (*this).m_hIcon = hIconLarge;
                    SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_ICON, STM_SETICON, (*this).m_hIcon as WPARAM, 0);
                }
            }
            return;
        }
        IDC_SHORTCUT_ADVANCED => {
            let result = DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_SHORTCUT_EXTENDED_PROPERTIES), hwndDlg, Some(ExtendedShortcutProc), (*this).m_bRunAs as LPARAM);
            if result == 1 || result == 0 {
                if (*this).m_bRunAs as isize != (result) as _ {
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
                (*this).m_bRunAs = result as i32;
            }
            return;
        }
        _ => {}
    }
    if codeNotify == EN_CHANGE || codeNotify == CBN_SELCHANGE {
        if (*this).m_bInInit == 0 {
            PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
        }
    }
}

pub unsafe fn CShellLink_OnNotify_body(this: *mut CShellLink, hwndDlg: HWND, idFrom: i32, pnmhdr: LPNMHDR) -> LRESULT {
    let mut wszBuf: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let lppsn = pnmhdr as *mut PSHNOTIFY;

    if (*lppsn).hdr.code == PSN_APPLY {
        /* set working directory */
        GetDlgItemTextW(hwndDlg, IDC_SHORTCUT_START_IN_EDIT, wszBuf.as_mut_ptr(), MAX_PATH as INT);
        CShellLink_SetWorkingDirectory_v2_body(this, wszBuf.as_mut_ptr());

        /* set link destination */
        let hWndTarget = GetDlgItem(hwndDlg, IDC_SHORTCUT_TARGET_TEXT);
        GetWindowTextW(hWndTarget, wszBuf.as_mut_ptr(), MAX_PATH as INT);
        if IsWindowEnabled(hWndTarget) != 0 {
            let mut lpszArgs: *mut u16 = core::ptr::null_mut();
            let mut unquoted = wszBuf.as_mut_ptr();
            StrTrimW(unquoted, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());

            if PathFileExistsW(unquoted) == 0 {
                lpszArgs = PathGetArgsW(unquoted);
                PathRemoveArgsW(unquoted);
                StrTrimW(lpszArgs, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
            }
            if *unquoted.add(0) == 0x22u16 && *unquoted.add(wcslen(unquoted) - 1) == 0x22u16 {
                PathUnquoteSpacesW(unquoted);
            }

            let pwszExt = PathFindExtensionW(unquoted);
            if _wcsicmp(pwszExt, CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0 {
                MessageBoxW(hwndDlg, CPP_COM_WSTR_3390E943BCAB.as_ptr(), core::ptr::null(), MB_ICONERROR);
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_INVALID_NOCHANGEPAGE);
                return TRUE as LRESULT;
            }

            if PathFileExistsW(unquoted) == 0 {
                MessageBoxW(hwndDlg, CPP_COM_WSTR_966A151B2AF3.as_ptr(), core::ptr::null(), MB_ICONERROR);
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_INVALID_NOCHANGEPAGE);
                return TRUE as LRESULT;
            }

            CShellLink_SetPath_v2_body(this, unquoted);
            CShellLink_SetArguments_v2_body(this, if !lpszArgs.is_null() { lpszArgs } else { CPP_COM_WSTR_DF3F619804A9.as_ptr() });
        }

        (*this).m_Header.wHotKey = SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_KEY_HOTKEY, HKM_GETHOTKEY, 0, 0) as u16;

        let index = SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_RUN_COMBO, CB_GETCURSEL, 0, 0) as i32;
        if index != CB_ERR {
            (*this).m_Header.nShowCommand = SendDlgItemMessageW(hwndDlg, IDC_SHORTCUT_RUN_COMBO, CB_GETITEMDATA, index as WPARAM, 0) as u32;
        }

        CShellLink_Save_v1_body(this, (*this).m_sLinkPath, TRUE);
        SHChangeNotify(SHCNE_UPDATEITEM, SHCNF_PATHW, (*this).m_sLinkPath, core::ptr::null::<u16>());
        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR);
        return TRUE as LRESULT;
    }
    return FALSE as LRESULT;
}

pub unsafe fn CShellLink_OnDestroy_body(this: *mut CShellLink, hwndDlg: HWND) -> () {
    if !(*this).m_hIcon.is_null() {
        DestroyIcon(((*this).m_hIcon).cast());
        (*this).m_hIcon = core::ptr::null_mut();
    }
}

pub unsafe fn CShellLink_SH_ShellLinkDlgProc_body(this: *mut CShellLink, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pThis = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut CShellLink;

    match uMsg {
        WM_INITDIALOG => {
            let ppsp = lParam as *mut PROPSHEETPAGEW;
            if !ppsp.is_null() {
                pThis = (*ppsp).lParam as *mut CShellLink;
                SetWindowLongPtr(hwndDlg, DWLP_USER, (pThis as isize) as _);
                return CShellLink_OnInitDialog_body(pThis, hwndDlg, wParam as HWND, lParam) as INT_PTR;
            }
        },
        WM_NOTIFY => {
            return CShellLink_OnNotify_body(pThis, hwndDlg, wParam as i32, lParam as *mut NMHDR);
        },
        WM_COMMAND => {
            CShellLink_OnCommand_body(pThis, hwndDlg, LOWORD((wParam) as _) as i32, lParam as HWND, HIWORD((wParam) as _) as UINT);
        },
        WM_DESTROY => {
            CShellLink_OnDestroy_body(pThis, hwndDlg);
        },
        _ => {}
    }

    FALSE as INT_PTR
}

pub unsafe fn CShellLink_AddPages_body(this: *mut CShellLink, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let hPage = SH_CreatePropertySheetPageEx(IDD_SHORTCUT_PROPERTIES as WORD, Some(SH_ShellLinkDlgProc), this as *mut _ as LPARAM, core::ptr::null(), Some(PropSheetPageLifetimeCallback_CShellLink));
    let hr = AddPropSheetPage(hPage, pfnAddPage, lParam);
    if hr < 0 {
        return hr;
    } else {
        CShellLink_AddRef(this); // For PropSheetPageLifetimeCallback
    }
    const CShellLink_PageIndex_Shortcut: i32 = 0;
    return 1 + CShellLink_PageIndex_Shortcut;
}

pub unsafe fn CShellLink_ReplacePage_body(this: *mut CShellLink, uPageID: EXPPS, pfnReplacePage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT { E_NOTIMPL }

pub unsafe fn CShellLink_SetSite_body(this: *mut CShellLink, punk: *mut IUnknown) -> HRESULT {
    CShellLink_cpp_com_assign_m_site(this, punk);

    return S_OK;
}

pub unsafe fn CShellLink_GetSite_body(this: *mut CShellLink, iid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_site.is_null() {
        return E_FAIL;
    }
    let m_site = (*this).m_site;
    ((*(*m_site).lpVtbl).QueryInterface)(m_site, iid, ppvSite)
}

pub unsafe fn CShellLink_DragEnter_body(this: *mut CShellLink, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    if *pdwEffect == DROPEFFECT_NONE {
        return S_OK;
    }

    let mut hr = SHELL_GetUIObjectOfAbsoluteItem(
        core::ptr::null_mut(),
        (*this).m_pPidl,
        (core::ptr::addr_of!(IID_IDropTarget)).cast(), core::ptr::addr_of_mut!((*this).m_DropTarget).cast(),
    );

    if hr >= 0 {
        let drop_target = (*this).m_DropTarget;
        hr = ((*(*drop_target).lpVtbl).DragEnter)(
            drop_target,
            pDataObject,
            dwKeyState,
            pt,
            pdwEffect,
        );
    } else {
        *pdwEffect = DROPEFFECT_NONE;
    }

    S_OK
}

pub unsafe fn CShellLink_DragOver_body(this: *mut CShellLink, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    let pTarget = (*this).m_DropTarget;
    if !pTarget.is_null() {
        hr = ((*(*pTarget).lpVtbl).DragOver)(pTarget, dwKeyState, pt, pdwEffect);
    }

    hr
}

pub unsafe fn CShellLink_DragLeave_body(this: *mut CShellLink) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let drop_target = (*this).m_DropTarget;
    if !drop_target.is_null() {
        hr = ((*(*drop_target).lpVtbl).DragLeave)(drop_target);
        ((*(*drop_target).lpVtbl).Release)(drop_target);
        (*this).m_DropTarget = core::ptr::null_mut();
    }
    hr
}

pub unsafe fn CShellLink_Drop_body(this: *mut CShellLink, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let pDropTarget = (*this).m_DropTarget;
    if !pDropTarget.is_null() {
        hr = ((*(*pDropTarget).lpVtbl).Drop)(pDropTarget, pDataObject, dwKeyState, pt, pdwEffect);
    }
    hr
}

pub unsafe fn CShellLink_CreateShortcutIcon_body(this: *mut CShellLink, wszIconPath: LPCWSTR, IconIndex: INT) -> HICON {
    let _ = this; // suppress unused warning

    let cx = GetSystemMetrics(SM_CXICON);
    let cy = GetSystemMetrics(SM_CYICON);
    let crMask = GetSysColor(COLOR_3DFACE);
    let mut wszLnkIcon: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut lnk_idx: i32 = 0;
    let himl: HIMAGELIST = (ImageList_Create(
        cx,
        cy,
        ILC_COLOR32 | ILC_MASK,
        1,
        1,
    )).cast();
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut hNewIcon: HICON = core::ptr::null_mut();
    let mut hShortcut: HICON = core::ptr::null_mut();

    if HLM_GetIconW(
        IDI_SHELL_SHORTCUT - 1,
        wszLnkIcon.as_mut_ptr(),
        wszLnkIcon.len() as DWORD,
        &mut lnk_idx,
    ) != 0
    {
        ExtractIconExW(
            wszLnkIcon.as_mut_ptr(),
            lnk_idx,
            &mut hShortcut,
            core::ptr::null_mut(),
            1,
        );
    } else {
        hShortcut = LoadImageW(
            shell32_hInstance,
            MAKEINTRESOURCE(IDI_SHELL_SHORTCUT as UINT),
            IMAGE_ICON,
            cx,
            cy,
            0,
        ) as HICON;
    }

    ExtractIconExW(
        wszIconPath,
        IconIndex,
        &mut hIcon,
        core::ptr::null_mut(),
        1,
    );

    if hIcon.is_null() || hShortcut.is_null() || himl.is_null() {
        // fall through to cleanup
    } else {
        let hDC: HDC = (CreateCompatibleDC(core::ptr::null_mut())).cast();
        if !hDC.is_null() {
            // Local definitions to match the Windows structures



            let mut bi: BITMAPINFO = core::mem::zeroed();
            ZeroMemory(&mut bi, core::mem::size_of::<BITMAPINFO>());
            bi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            bi.bmiHeader.biWidth = cx;
            bi.bmiHeader.biHeight = cy;
            bi.bmiHeader.biPlanes = 1;
            bi.bmiHeader.biBitCount = 32;

            let mut pvBits: *mut core::ffi::c_void = core::ptr::null_mut();
            let hbm: HBITMAP = CreateDIBSection(
                (hDC).cast(),
                &bi,
                DIB_RGB_COLORS,
                &mut pvBits,
                core::ptr::null_mut(),
                0,
            );

            if !hbm.is_null() {
                let hbmOld: HGDIOBJ = SelectObject(hDC, (hbm).cast());
                {
                    let hbr: HBRUSH = (CreateSolidBrush(crMask)).cast();
                    let rc = RECT {
                        left: 0,
                        top: 0,
                        right: cx,
                        bottom: cy,
                    };
                    FillRect((hDC).cast(), &rc, (hbr).cast());
                    DeleteObject((hbr).cast());

                    DrawIconEx(
                        (hDC).cast(),
                        0,
                        0,
                        (hIcon).cast(),
                        cx,
                        cy,
                        0,
                        core::ptr::null_mut(),
                        DI_NORMAL,
                    );
                    DrawIconEx(
                        (hDC).cast(),
                        0,
                        0,
                        (hShortcut).cast(),
                        cx,
                        cy,
                        0,
                        core::ptr::null_mut(),
                        DI_NORMAL,
                    );
                }
                SelectObject((hDC).cast(), hbmOld);

                let iAdded: i32 = ImageList_AddMasked((himl).cast(), hbm, crMask);
                hNewIcon = (ImageList_GetIcon((himl).cast(), iAdded, ILD_NORMAL | ILD_TRANSPARENT)).cast();

                DeleteObject((hbm).cast());
            }
            DeleteDC((hDC).cast());
        }
    }

    // cleanup
    if !hIcon.is_null() {
        DestroyIcon((hIcon).cast());
    }
    if !hShortcut.is_null() {
        DestroyIcon((hShortcut).cast());
    }
    if !himl.is_null() {
        ImageList_Destroy((himl).cast());
    }

    hNewIcon
}


// ============================================================================
// Stitch substrate: base typedefs, SDK records, FFI imports, and Rust
// re-implementations of header-inline C++ helpers (static inline / templates
// have no external linkage, so they are reproduced here from the cited
// headers). See CShellLink-stitch-report.md and CShellLink-bridges.md.
// ============================================================================

// --- windef.h/winnt.h/wtypes.h base typedefs absent from types.rs/shellcom.rs ---
pub type WPARAM = UINT_PTR;
pub type LRESULT = LONG_PTR;
pub use crate::island::HICON;
pub use crate::island::HDC;
pub use crate::island::HBRUSH;
pub use crate::island::HGDIOBJ;
pub use crate::island::HIMAGELIST; // commctrl.h
pub type HDROP = HANDLE;      // shellapi.h
pub type HLOCAL = HANDLE;     // winbase.h
pub type LCID = DWORD;        // winnt.h
pub type CLIPFORMAT = WORD;   // wtypes.h
pub type SHORT = i16;         // winnt.h
// winuser.h DLGPROC; prsht.h LPFNPSPCALLBACKW
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;
pub type LPFNPSPCALLBACKW = unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

// prsht.h _PROPSHEETPAGEW. Only the fields up to lParam are dereferenced by
// this module (WM_INITDIALOG page recovery + lifetime callback); the fields
// after lParam keep the prefix layout faithful and are never read here.
pub use crate::island::PROPSHEETPAGEW;

// shellapi.h _CMINVOKECOMMANDINFOEX (ANSI base + Unicode tail + ptInvoke)
#[repr(C)]
pub struct CMINVOKECOMMANDINFOEX {
    pub cbSize: DWORD,
    pub fMask: DWORD,
    pub hwnd: HWND,
    pub lpVerb: LPCSTR,
    pub lpParameters: LPCSTR,
    pub lpDirectory: LPCSTR,
    pub nShow: INT,
    pub dwHotKey: DWORD,
    pub hIcon: HANDLE,
    pub lpTitle: LPCSTR,
    pub lpVerbW: LPCWSTR,
    pub lpParametersW: LPCWSTR,
    pub lpDirectoryW: LPCWSTR,
    pub lpTitleW: LPCWSTR,
    pub ptInvoke: POINT,
}

// shlguid.h: CLSID_ShellLink {00021401-0000-0000-C000-000000000046}
pub const CLSID_ShellLink: CLSID = GUID {
    Data1: 0x00021401, Data2: 0x0000, Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
// reactos/shlguid_undoc.h:100-101
pub const SHELL32_AdvtShortcutProduct: GUID = GUID {
    Data1: 0x9DB1186F, Data2: 0x40DF, Data3: 0x11D1,
    Data4: [0xAA, 0x8C, 0x00, 0xC0, 0x4F, 0xB6, 0x78, 0x63],
};
pub const SHELL32_AdvtShortcutComponent: GUID = GUID {
    Data1: 0x9DB1186E, Data2: 0x40DF, Data3: 0x11D1,
    Data4: [0xAA, 0x8C, 0x00, 0xC0, 0x4F, 0xB6, 0x78, 0x63],
};

// guiddef.h IsEqualGUID/IsEqualIID (by-value: several callers pass fields of
// the #[repr(C, packed)] SHELL_LINK_HEADER, which cannot be borrowed).
#[inline]
pub fn IsEqualGUID(a: GUID, b: GUID) -> bool {
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}
#[inline]
pub fn IsEqualIID(a: IID, b: IID) -> bool { IsEqualGUID(a, b) }

// winuser.h/windef.h numeric macros
#[inline] pub const fn LOWORD(v: usize) -> WORD { (v & 0xffff) as WORD }
#[inline] pub const fn HIWORD(v: usize) -> WORD { ((v >> 16) & 0xffff) as WORD }
#[inline] pub const fn LOBYTE(v: DWORD) -> BYTE { (v & 0xff) as BYTE }
#[inline] pub const fn MAKEINTRESOURCEW(id: UINT) -> LPCWSTR { (id & 0xffff) as usize as LPCWSTR }
#[inline] pub const fn MAKEINTRESOURCE(id: UINT) -> LPCWSTR { MAKEINTRESOURCEW(id) } // UNICODE build

// debughlp.h pdump: debug-channel PIDL dump; inert here exactly like the
// TRACE macros (release/debug-channel-off behaviour).
#[inline] pub unsafe fn pdump(_pidl: LPCITEMIDLIST) {}

// reactos/shellutils.h:474-475 (overloaded static inline)
#[inline] pub unsafe fn StrIsNullOrEmptyA(s: LPCSTR) -> bool { s.is_null() || *s == 0 }
#[inline] pub unsafe fn StrIsNullOrEmptyW(s: LPCWSTR) -> bool { s.is_null() || *s == 0 }

// reactos/shellutils.h:722-731 (both overloads collapse to the EX check)
#[inline]
pub unsafe fn IsUnicode(ici: *const CMINVOKECOMMANDINFO) -> bool {
    let minsize = core::mem::offset_of!(CMINVOKECOMMANDINFOEX, ptInvoke) as DWORD;
    ((*ici).fMask & CMIC_MASK_UNICODE) != 0 && (*ici).cbSize >= minsize
}

// reactos/shellutils.h:688 CmicFlagsToSeeFlags(flags) = flags & SEE_CMIC_COMMON_FLAGS
#[inline] pub const fn CmicFlagsToSeeFlags(flags: DWORD) -> DWORD { flags & SEE_CMIC_COMMON_FLAGS }

// wine/shell32_main.h:223-229 __SHCloneStrAtoW (static __inline; SHAlloc'd,
// callers free with CoTaskMemFree — SHAlloc is CoTaskMemAlloc in ReactOS)
pub unsafe fn __SHCloneStrAtoW(target: *mut LPWSTR, source: LPCSTR) -> LPWSTR {
    let len = MultiByteToWideChar(CP_ACP, 0, source, -1, core::ptr::null_mut(), 0);
    *target = SHAlloc(((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
    if !(*target).is_null() {
        MultiByteToWideChar(CP_ACP, 0, source, -1, *target, len);
    }
    *target
}

// precomp.h:154-162 AddPropSheetPage (static inline)
pub unsafe fn AddPropSheetPage(
    hPage: HPROPSHEETPAGE, pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM,
) -> HRESULT {
    if hPage.is_null() { return E_FAIL; }
    // The C++ calls pfnAddPage unconditionally; a NULL callback would fault
    // there, so treating it as failure is the defensive superset.
    if let Some(add) = pfnAddPage {
        if add(hPage, lParam) != 0 { return S_OK; }
    }
    DestroyPropertySheetPage((hPage).cast());
    E_FAIL
}

// precomp.h:165-171 template<class T> PropSheetPageLifetimeCallback,
// instantiated for T = CShellLink (page lParam holds the owning object).
pub unsafe extern "system" fn PropSheetPageLifetimeCallback_CShellLink(
    _hWnd: HWND, uMsg: UINT, pPSP: *mut PROPSHEETPAGEW,
) -> UINT {
    if uMsg == PSPCB_RELEASE {
        CShellLink_Release((*pPSP).lParam as *mut CShellLink);
    }
    TRUE as UINT
}

// prsht.h: #define PSM_CHANGED (WM_USER+104); PropSheet_Changed(hDlg, hwnd) macro
pub const PSM_CHANGED: UINT = 0x0468;
#[inline]
pub unsafe fn PropSheet_Changed(hPropSheetDlg: HWND, hwndPage: HWND) -> LRESULT {
    SendMessageW(hPropSheetDlg, PSM_CHANGED, hwndPage as WPARAM, 0)
}

// CShellLink.h:114 static member dialog proc (object recovered via DWLP_USER)
pub unsafe extern "system" fn SH_ShellLinkDlgProc(
    hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> INT_PTR {
    CShellLink_SH_ShellLinkDlgProc_body(core::ptr::null_mut(), hwndDlg, uMsg, wParam, lParam)
}

// CShellLink.cpp:2707-2743 ExtendedShortcutProc (file-scope dialog proc)
pub unsafe extern "system" fn ExtendedShortcutProc(
    hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            if lParam != 0 {
                let hDlgCtrl = GetDlgItem(hwndDlg, IDC_SHORTEX_RUN_DIFFERENT);
                SendMessageW(hDlgCtrl, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            let hDlgCtrl = GetDlgItem(hwndDlg, IDC_SHORTEX_RUN_DIFFERENT);
            if LOWORD((wParam) as _) as UINT == IDOK {
                if SendMessageW(hDlgCtrl, BM_GETCHECK, 0, 0) == BST_CHECKED {
                    EndDialog(hwndDlg, 1);
                } else {
                    EndDialog(hwndDlg, 0);
                }
            } else if LOWORD((wParam) as _) as UINT == IDCANCEL {
                EndDialog(hwndDlg, -1);
            } else if LOWORD((wParam) as _) as INT == IDC_SHORTEX_RUN_DIFFERENT {
                if SendMessageW(hDlgCtrl, BM_GETCHECK, 0, 0) == BST_CHECKED {
                    SendMessageW(hDlgCtrl, BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
                } else {
                    SendMessageW(hDlgCtrl, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
                }
            }
        }
        _ => {}
    }
    FALSE as INT_PTR
}

// strsafe.h semantics: bounded copy/concat/format, NUL-guaranteed,
// STRSAFE_E_INSUFFICIENT_BUFFER on truncation. The formatter handles only the
// literal + %s/%ls patterns used by CShellLink.cpp (house style precedent:
// send_to_menu.rs cpp_com_string_cb_printf_w).
pub unsafe fn StringCchCatW(dst: LPWSTR, cch: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || cch == 0 { return E_INVALIDARG; }
    let mut out = 0usize;
    while out < cch && *dst.add(out) != 0 { out += 1; }
    if out >= cch { return E_INVALIDARG; } // unterminated destination
    let mut index = 0usize;
    while *src.add(index) != 0 {
        if out + 1 >= cch { *dst.add(cch - 1) = 0; return STRSAFE_E_INSUFFICIENT_BUFFER; }
        *dst.add(out) = *src.add(index); out += 1; index += 1;
    }
    *dst.add(out) = 0;
    S_OK
}
pub unsafe fn cpp_com_string_cch_printf_w(
    dst: LPWSTR, cch: usize, format: LPCWSTR, args: &[LPCWSTR],
) -> HRESULT {
    if dst.is_null() || format.is_null() || cch == 0 { return E_INVALIDARG; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        let is_s = unit == b'%' as WCHAR && *format.add(input + 1) == b's' as WCHAR;
        let is_ls = unit == b'%' as WCHAR
            && *format.add(input + 1) == b'l' as WCHAR
            && *format.add(input + 2) == b's' as WCHAR;
        if is_s || is_ls {
            if arg_index >= args.len() { return E_INVALIDARG; }
            let arg = args[arg_index]; arg_index += 1;
            input += if is_ls { 3 } else { 2 };
            if arg.is_null() { continue; }
            let mut index = 0usize;
            while *arg.add(index) != 0 {
                if out + 1 >= cch { *dst.add(cch - 1) = 0; return STRSAFE_E_INSUFFICIENT_BUFFER; }
                *dst.add(out) = *arg.add(index); out += 1; index += 1;
            }
            continue;
        }
        if out + 1 >= cch { *dst.add(cch - 1) = 0; return STRSAFE_E_INSUFFICIENT_BUFFER; }
        *dst.add(out) = unit; out += 1; input += 1;
    }
    *dst.add(out) = 0;
    S_OK
}
pub unsafe fn StringCbPrintfW1(dst: LPWSTR, bytes: usize, format: LPCWSTR, arg: LPCWSTR) -> HRESULT {
    cpp_com_string_cch_printf_w(dst, bytes / core::mem::size_of::<WCHAR>(), format, &[arg])
}

// ---- FFI imports (WINAPI => extern "system"; cdecl internals => extern "C") ----
extern "system" {
    // kernel32 (winbase.h/fileapi.h/winnls.h/datetimeapi.h)
    fn ExpandEnvironmentStringsW(src: LPCWSTR, dst: LPWSTR, size: DWORD) -> DWORD;
    fn GetFileAttributesW(path: LPCWSTR) -> DWORD;
    fn FindClose(handle: HANDLE) -> BOOL;
    fn DeleteFileW(path: LPCWSTR) -> BOOL;
    #[link_name = "GetFullPathNameW"]
    fn Win32GetFullPathNameW(name: LPCWSTR, length: DWORD, buffer: LPWSTR, file_part: *mut LPWSTR) -> DWORD;
    #[link_name = "GetShortPathNameW"]
    fn Win32GetShortPathNameW(long: LPCWSTR, short: LPWSTR, cch: DWORD) -> DWORD;
    fn GetDriveTypeW(root: LPCWSTR) -> UINT;
    #[link_name = "GetVolumeInformationW"]
    fn Win32GetVolumeInformationW(root: LPCWSTR, name: LPWSTR, name_cch: DWORD, serial: LPDWORD,
                             max_len: LPDWORD, flags: LPDWORD, fs_name: LPWSTR, fs_cch: DWORD) -> BOOL;
    #[link_name = "MultiByteToWideChar"]
    fn Win32MultiByteToWideChar(page: UINT, flags: DWORD, multi: LPCSTR, multi_len: INT,
                           wide: LPWSTR, wide_len: INT) -> INT;
    #[link_name = "WideCharToMultiByte"]
    fn Win32WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, multi: LPSTR,
                           multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;
    fn GetVersion() -> DWORD;
    fn FileTimeToSystemTime(file_time: *const FILETIME, system_time: *mut SYSTEMTIME) -> BOOL;
    #[link_name = "GetDateFormatW"]
    fn Win32GetDateFormatW(locale: LCID, flags: DWORD, date: *const SYSTEMTIME, format: LPCWSTR,
                      buffer: LPWSTR, cch: INT) -> INT;
    // user32 (winuser.h)
    #[link_name = "PostMessageW"] // UNICODE build: PostMessage => PostMessageW
    fn PostMessage(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> BOOL;
    fn GetKeyState(key: INT) -> SHORT;
    fn GetSysColor(index: INT) -> COLORREF;
    #[link_name = "PrivateExtractIconsW"]
    fn Win32PrivateExtractIconsW(path: LPCWSTR, index: INT, cx: INT, cy: INT, icons: *mut HICON,
                            ids: *mut UINT, count: UINT, flags: UINT) -> UINT;
    // gdi32 (wingdi.h)
    fn DeleteObject(object: HGDIOBJ) -> BOOL;
    // comctl32 (commctrl.h/prsht.h)
    // ole32 (objbase.h/combaseapi.h)
    fn ReleaseStgMedium(medium: *mut STGMEDIUM);
    // advapi32 (appmgmt.h:46)
    fn CommandLineFromMsiDescriptor(descriptor: LPWSTR, command_line: LPWSTR, cch: *mut DWORD) -> DWORD;
    // shell32 public exports (shellapi.h/shlobj.h)
    #[link_name = "SHChangeNotify"]
    fn Win32SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn ILLoadFromStream(stream: *mut IStream, pidl: *mut LPITEMIDLIST) -> HRESULT;
    fn ILSaveToStream(stream: *mut IStream, pidl: LPCITEMIDLIST) -> HRESULT;
    fn SHOpenFolderAndSelectItems(folder: PCIDLIST_ABSOLUTE, cidl: UINT,
                                  apidl: PCUITEMID_CHILD_ARRAY, flags: DWORD) -> HRESULT;
    fn FindExecutableW(file: LPCWSTR, directory: LPCWSTR, result: LPWSTR) -> HINSTANCE;
    fn DragQueryFileW(drop: HDROP, index: UINT, path: LPWSTR, cch: UINT) -> UINT;
    fn SHSimpleIDListFromPathW(path: LPCWSTR) -> LPITEMIDLIST;
    fn PathResolveW(path: LPWSTR, dirs: *mut LPCWSTR, flags: DWORD) -> BOOL;
    // shlwapi (shlwapi.h; SHExpand/SHAnsi/datablocks: reactos/shlwapi_undoc.h:140-164)
    #[link_name = "SHExpandEnvironmentStringsW"]
    fn Win32SHExpandEnvironmentStringsW(src: LPCWSTR, dst: LPWSTR, cch: DWORD) -> DWORD;
    fn SHAnsiToUnicode(src: LPCSTR, dst: LPWSTR, cch: INT) -> DWORD;
    fn PathFileExistsW(path: LPCWSTR) -> BOOL;
    fn PathFileExistsAndAttributesW(path: LPCWSTR, attributes: *mut DWORD) -> BOOL;
    fn PathIsDirectoryW(path: LPCWSTR) -> BOOL;
    fn PathAddBackslashW(path: LPWSTR) -> LPWSTR;
    fn PathRemoveBackslashW(path: LPWSTR) -> LPWSTR;
    fn PathFindFileNameW(path: LPCWSTR) -> LPWSTR;
    fn PathFindExtensionW(path: LPCWSTR) -> LPWSTR;
    fn PathGetArgsW(path: LPCWSTR) -> LPWSTR;
    fn PathRemoveArgsW(path: LPWSTR);
    fn PathRemoveFileSpecW(path: LPWSTR) -> BOOL;
    fn PathUnquoteSpacesW(path: LPWSTR) -> BOOL;
    fn PathIsFileSpecW(path: LPCWSTR) -> BOOL;
    fn PathUnExpandEnvStringsW(path: LPCWSTR, buffer: LPWSTR, cch: UINT) -> BOOL;
    fn StrTrimW(text: LPWSTR, trim_chars: LPCWSTR) -> BOOL;
}

// winuser.h: on win32 the *Ptr forms are macro aliases of the LONG forms —
// bind to the real 32-bit exports there (user_notification.rs precedent).
#[cfg(target_pointer_width = "64")]
extern "system" {
    #[link_name = "GetWindowLongPtrW"]
    fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR;
    #[link_name = "SetWindowLongPtrW"]
    fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
}
#[cfg(target_pointer_width = "32")]
extern "system" {
    #[link_name = "GetWindowLongW"]
    fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR;
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
}

extern "C" {
    // msvcrt (cdecl)
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
    // shell32 internals with C linkage:
    // precomp.h:275-279 (EXTERN_C)
    // wine/shell32_main.h:79 (extern "C" block)
    // wine/shell32_main.h:250-251 (extern "C" block; defined in propsheet.cpp)
    // wine/pidl.h (extern "C"; cdecl internal, pidl.rs:406 precedent)
    // wine/shell32_main.h:867 precedent (module instance for resource loads)
    static shell32_hInstance: HINSTANCE;
}

// C++-linkage helper in retained utils.cpp — narrow bridge, see
// CShellLink-bridges.md. The bridge is EXTERN_C HRESULT WINAPI (stdcall), so it
// must live in an extern "system" block: on i386 the reference must decorate as
// `_CShellLink_SHGetNameAndFlagsW_RustBridge@20` to match the C++ definition.
extern "system" {
    #[link_name = "CShellLink_SHGetNameAndFlagsW_RustBridge"]
    fn SHGetNameAndFlagsW(pidl: LPCITEMIDLIST, flags: DWORD, text: LPWSTR, cch: UINT,
                          attributes: *mut DWORD) -> HRESULT;
}


// C-arg-shaped wrappers: the C++ passes int/UINT/size_t length flavours
// interchangeably; keep every call site literal and adapt here.
trait CppComInt { fn cpp_com_int(self) -> INT; }
impl CppComInt for usize { fn cpp_com_int(self) -> INT { self as INT } }
impl CppComInt for u32 { fn cpp_com_int(self) -> INT { self as INT } }
impl CppComInt for i32 { fn cpp_com_int(self) -> INT { self } }
impl CppComInt for u16 { fn cpp_com_int(self) -> INT { self as INT } }
#[inline]
unsafe fn MultiByteToWideChar<A: CppComInt, B: CppComInt>(
    page: UINT, flags: DWORD, multi: LPCSTR, multi_len: A, wide: LPWSTR, wide_len: B) -> INT {
    Win32MultiByteToWideChar(page, flags, multi, multi_len.cpp_com_int(), wide, wide_len.cpp_com_int())
}
#[inline]
unsafe fn WideCharToMultiByte<A: CppComInt, B: CppComInt>(
    page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: A, multi: LPSTR, multi_len: B,
    default_char: LPCSTR, used_default: *mut BOOL) -> INT {
    Win32WideCharToMultiByte(page, flags, wide, wide_len.cpp_com_int(), multi,
                             multi_len.cpp_com_int(), default_char, used_default)
}
#[inline]
unsafe fn WideCharToMultiByte_u32<A: CppComInt, B: CppComInt>(
    page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: A, multi: LPSTR, multi_len: B,
    default_char: LPCSTR, used_default: *mut BOOL) -> u32 {
    WideCharToMultiByte(page, flags, wide, wide_len, multi, multi_len, default_char, used_default) as u32
}
#[inline]
unsafe fn GetFullPathNameW<N: CppComInt>(name: LPCWSTR, length: N, buffer: LPWSTR, file_part: *mut LPWSTR) -> DWORD {
    Win32GetFullPathNameW(name, length.cpp_com_int() as DWORD, buffer, file_part)
}
#[inline]
unsafe fn GetDateFormatW<N: CppComInt>(locale: LCID, flags: DWORD, date: *const SYSTEMTIME,
                                       format: LPCWSTR, buffer: LPWSTR, cch: N) -> INT {
    Win32GetDateFormatW(locale, flags, date, format, buffer, cch.cpp_com_int())
}
#[inline]
unsafe fn GetShortPathNameW<N: CppComInt>(long: LPCWSTR, short: LPWSTR, cch: N) -> DWORD {
    Win32GetShortPathNameW(long, short, cch.cpp_com_int() as DWORD)
}
#[inline]
unsafe fn SHExpandEnvironmentStringsW<N: CppComInt>(src: LPCWSTR, dst: LPWSTR, cch: N) -> DWORD {
    Win32SHExpandEnvironmentStringsW(src, dst, cch.cpp_com_int() as DWORD)
}
#[inline]
unsafe fn PrivateExtractIconsW<A: CppComInt, B: CppComInt, C: CppComInt>(
    path: LPCWSTR, index: A, cx: B, cy: C, icons: *mut HICON, ids: *mut UINT, count: UINT, flags: UINT) -> UINT {
    Win32PrivateExtractIconsW(path, index.cpp_com_int(), cx.cpp_com_int(), cy.cpp_com_int(), icons, ids, count, flags)
}
#[inline]
unsafe fn GetVolumeInformationW<N: CppComInt>(root: LPCWSTR, name: LPWSTR, name_cch: N, serial: LPDWORD,
    max_len: LPDWORD, flags: LPDWORD, fs_name: LPWSTR, fs_cch: DWORD) -> BOOL {
    Win32GetVolumeInformationW(root, name, name_cch.cpp_com_int() as DWORD, serial, max_len, flags, fs_name, fs_cch)
}
#[inline]
unsafe fn SHChangeNotify<A, B>(event: LONG, flags: UINT, item1: *const A, item2: *const B) {
    Win32SHChangeNotify(event, flags, item1.cast(), item2.cast())
}
// kernel32 lstrcpynW with C-int capacity (types.rs:209 raw import shadowed)
#[inline]
unsafe fn lstrcpynW<N: CppComInt>(dst: LPWSTR, src: LPCWSTR, capacity: N) -> LPWSTR {
    crate::types::lstrcpynW(dst, src, capacity.cpp_com_int())
}

// ---- Exported constructors (CShellLink.cpp:3231-3252) ----
// HRESULT WINAPI IShellLink_ConstructFromPath(WCHAR*, REFIID, LPVOID*)
#[no_mangle]
pub unsafe extern "system" fn IShellLink_ConstructFromPath(
    path: *mut WCHAR, riid: REFIID, ppv: *mut LPVOID,
) -> HRESULT {
    let mut ppf: *mut IPersistFile = core::ptr::null_mut();
    let mut _ppf_guard = CppComPtrGuard::new(&mut ppf);
    let hr = CShellLink_CreateInstance(
        core::ptr::null_mut(),
        &IID_IPersistFile,
        (&mut ppf as *mut *mut IPersistFile) as *mut *mut c_void,
    );
    if hr < 0 { return hr; }
    let hr = ((*(*ppf).lpVtbl).Load)(ppf, path, 0);
    if hr < 0 { return hr; }
    ((*(*ppf).lpVtbl).QueryInterface)(ppf, riid, ppv as *mut *mut c_void)
}

// HRESULT WINAPI IShellLink_ConstructFromFile(IShellFolder*, LPCITEMIDLIST, REFIID, LPVOID*)
#[no_mangle]
pub unsafe extern "system" fn IShellLink_ConstructFromFile(
    psf: *mut c_void, pidl: LPCITEMIDLIST, riid: REFIID, ppv: *mut LPVOID,
) -> HRESULT {
    let mut path = [0u16; MAX_PATH];
    if ILGetDisplayNameExW((psf).cast(), pidl, path.as_mut_ptr(), 0) == 0 {
        return E_FAIL;
    }
    IShellLink_ConstructFromPath(path.as_mut_ptr(), riid, ppv)
}
