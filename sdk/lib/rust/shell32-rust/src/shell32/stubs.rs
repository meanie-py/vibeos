//! FLIPPED: this TU is wired into the crate and shipping; the C++ original is
//! commented out of dll/win32/shell32/CMakeLists.txt.
use core::ffi::c_void;
use crate::prelude::*;
use crate::shell32::sdk_consts::*;

extern "system" {
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
}

// Upstream-sync 2026-09: the four functions below (GetAppPathFromLink,
// GetCurrentProcessExplicitAppUserModelID, SetCurrentProcessExplicitAppUserModelID,
// RunInstallUninstallStubs) are genuinely lost exports -- real Win32 stub bodies in
// upstream's stubs.cpp with no Rust twin, because this TU's flip predates these upstream
// additions. Ported 1:1; every one of them is ALREADY an "Unimplemented"/FIXME stub in
// the C, so this is not a lesser port, just closing the export gap the flip left open.
#[no_mangle]
pub unsafe extern "system" fn GetAppPathFromLink(
    _pItem: *mut IShellItem,
    _pwszPathBuffer: LPWSTR,
    _dwBufferLen: DWORD,
) -> HRESULT {
    FIXME!("GetAppPathFromLink() stub\n");
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentProcessExplicitAppUserModelID(
    AppID: *mut PWSTR,
) -> HRESULT {
    SHStrDupW(
        [b'R' as u16, b'O' as u16, b'S' as u16, b' ' as u16,
         b'F' as u16, b'i' as u16, b'x' as u16, b'M' as u16, b'e' as u16, 0u16]
            .as_ptr(),
        AppID as *mut LPWSTR,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SetCurrentProcessExplicitAppUserModelID(
    _AppID: PCWSTR,
) -> HRESULT {
    FIXME!("SetCurrentProcessExplicitAppUserModelID stub\n");
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RunInstallUninstallStubs() -> DWORD {
    FIXME!("RunInstallUninstallStubs() stub!\n");
    0
}

// Upstream-sync 2026-09 (round 2): SHQueryUserNotificationState -- fifth lost export the
// scanner's tail-comment fix surfaced, alongside the four above. Trivial 1:1 port
// (shellapi.h QUERY_USER_NOTIFICATION_STATE is a plain enum, not in the bindgen island).
type QUERY_USER_NOTIFICATION_STATE = core::ffi::c_int;
const QUNS_ACCEPTS_NOTIFICATIONS: QUERY_USER_NOTIFICATION_STATE = 5;

#[no_mangle]
pub unsafe extern "system" fn SHQueryUserNotificationState(
    pquns: *mut QUERY_USER_NOTIFICATION_STATE,
) -> HRESULT {
    FIXME!("SHQueryUserNotificationState() stub\n");
    if pquns.is_null() {
        return E_INVALIDARG;
    }
    *pquns = QUNS_ACCEPTS_NOTIFICATIONS;
    S_OK
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn CheckDiskSpace() {
    FIXME!("CheckDiskSpace() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn SHReValidateDarwinCache() {
    FIXME!("SHReValidateDarwinCache() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn GetFileDescriptor(
    pFileGroupDesc: *mut FILEGROUPDESCRIPTOR,
    bUnicode: BOOLEAN,
    iIndex: c_int,
    lpName: PWSTR,
) -> *mut FILEDESCRIPTOR {
    FIXME!("GetFileDescriptor() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn MakeShellURLFromPathW(
    lpPath: PCWSTR,
    lpUrl: PWSTR,
    cchMax: c_int,
) -> BOOL {
    FIXME!("MakeShellURLFromPathW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn MakeShellURLFromPathA(
    lpPath: PCHAR,
    lpUrl: PCHAR,
    cchMax: c_int,
) -> BOOL {
    let mut szPath = [0u16; MAX_PATH as usize];
    let mut szURL = [0u16; MAX_PATH as usize];
    let ret;
    SHAnsiToUnicode(lpPath, szPath.as_mut_ptr(), (szPath.len() - 1) as c_int); // _countof gives element count
    ret = MakeShellURLFromPathW(szPath.as_ptr(), szURL.as_mut_ptr(), (szURL.len() - 1) as c_int);
    SHUnicodeToAnsi(szURL.as_ptr(), lpUrl, cchMax);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn SHParseDarwinIDFromCacheW(
    lpUnknown1: PCWSTR,
    lpUnknown2: PWSTR,
) -> HRESULT {
    FIXME!("SHParseDarwinIDFromCacheW() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHCopyMonikerToTemp(
    pMoniker: *mut IMoniker,
    lpInput: PCWSTR,
    lpOutput: PWSTR,
    cchMax: c_int,
) -> HRESULT {
    TRACE!("SHCopyMonikerToTemp() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn CheckWinIniForAssocs() -> HLOCAL {
    FIXME!("CheckWinIniForAssocs() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn SHGetSetFolderCustomSettingsW(
    pfcs: *mut SHFOLDERCUSTOMSETTINGSW,
    pszPath: PCWSTR,
    dwReadWrite: DWORD,
) -> HRESULT {
    FIXME!("SHGetSetFolderCustomSettingsW() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHGetSetFolderCustomSettingsA(
    pfcs: *mut SHFOLDERCUSTOMSETTINGSA,
    pszPath: PCHAR,
    dwReadWrite: DWORD,
) -> HRESULT {
    FIXME!("SHGetSetFolderCustomSettingsA() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn CDefFolderMenu_MergeMenu(
    hInstance: HINSTANCE,
    uMainMerge: UINT,
    uPopupMerge: UINT,
    lpQcmInfo: *mut QCMINFO,
) {
    FIXME!("CDefFolderMenu_MergeMenu() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn CDefFolderMenu_Create(
    pidlFolder: LPCITEMIDLIST,
    hwnd: HWND,
    uidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    psf: *mut IShellFolder,
    lpfn: LPFNDFMCALLBACK,
    hProgID: HKEY,
    hBaseProgID: HKEY,
    ppcm: *mut *mut IContextMenu,
) -> HRESULT {
    FIXME!("CDefFolderMenu_Create() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHWaitOp_Operate(
    lpUnknown1: LPVOID,
    dwUnknown2: DWORD,
) {
    FIXME!("SHWaitOp_Operate() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn RealDriveTypeFlags(
    iDrive: c_int,
    bUnknown: BOOLEAN,
) -> c_int {
    FIXME!("RealDriveTypeFlags() stub\n");
    1
}
#[no_mangle]
pub unsafe extern "system" fn ShellHookProc(
    iCode: c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> c_long {
    TRACE!("ShellHookProc() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheShortenPathW(
    lpPath: PWSTR,
    bShorten: BOOLEAN,
) -> BOOL {
    FIXME!("SheShortenPathW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheShortenPathA(
    lpPath: PCHAR,
    bShorten: BOOLEAN,
) -> BOOL {
    let mut szPath = [0u16; MAX_PATH as usize];
    MultiByteToWideChar(CP_ACP, 0, lpPath, -1, szPath.as_mut_ptr(), (szPath.len() - 1) as c_int);
    let ret = SheShortenPathW(szPath.as_mut_ptr(), bShorten);
    WideCharToMultiByte(CP_ACP, 0, szPath.as_ptr(), -1, lpPath, (MAX_PATH) as i32, core::ptr::null_mut(), core::ptr::null_mut());
    ret
}
#[no_mangle]
pub unsafe extern "system" fn SheSetCurDrive(iIndex: c_int) -> c_int {
    FIXME!("SheSetCurDrive() stub\n");
    1
}
#[no_mangle]
pub unsafe extern "system" fn SheGetPathOffsetW(lpPath: PWSTR) -> c_int {
    FIXME!("SheGetPathOffsetW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheGetDirExW(
    lpDrive: PWSTR,
    lpCurDirLen: *mut DWORD,
    lpCurDir: PWSTR,
) -> BOOL {
    FIXME!("SheGetDirExW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheGetCurDrive() -> c_int {
    FIXME!("SheGetCurDrive() stub\n");
    1
}
#[no_mangle]
pub unsafe extern "system" fn SheFullPathW(
    lpFullName: PWSTR,
    dwPathSize: DWORD,
    lpBuffer: PWSTR,
) -> c_int {
    FIXME!("SheFullPathW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheFullPathA(
    lpFullName: PCHAR,
    dwPathSize: DWORD,
    lpBuffer: PCHAR,
) -> c_int {
    FIXME!("SheFullPathA() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheConvertPathW(
    lpCmd: PWSTR,
    lpFileName: PWSTR,
    uCmdLen: UINT,
) -> BOOL {
    FIXME!("SheConvertPathW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheChangeDirExW(lpDir: PWSTR) -> c_int {
    FIXME!("SheChangeDirExW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SheChangeDirExA(lpDir: PCHAR) -> c_int {
    FIXME!("SheChangeDirExA() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHInvokePrinterCommandW(
    hwnd: HWND,
    uAction: UINT,
    lpBuf1: PCWSTR,
    lpBuf2: PCWSTR,
    fModal: BOOLEAN,
) -> BOOL {
    FIXME!("SHInvokePrinterCommandW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHInvokePrinterCommandA(
    hwnd: HWND,
    uAction: UINT,
    lpBuf1: PCHAR,
    lpBuf2: PCHAR,
    fModal: BOOLEAN,
) -> BOOL {
    FIXME!("SHInvokePrinterCommandA() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHCreateProcessAsUserW(
    pscpi: *mut SHCREATEPROCESSINFOW,
) -> BOOL {
    FIXME!("SHCreateProcessAsUserW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn PrintersGetCommand_RunDLL(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PWSTR,
    nCmdShow: c_int,
) {
    FIXME!("PrintersGetCommand_RunDLL() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn PrintersGetCommand_RunDLLA(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PCHAR,
    nCmdShow: c_int,
) {
    FIXME!("PrintersGetCommand_RunDLLA() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn PrintersGetCommand_RunDLLW(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PWSTR,
    nCmdShow: c_int,
) {
    FIXME!("PrintersGetCommand_RunDLLW() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn SHGetShellFolderViewCB(hwnd: HWND) -> *mut IShellFolderViewCB {
    FIXME!("SHGetShellFolderViewCB() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn SHLookupIconIndexA(
    lpName: PCHAR,
    iIndex: c_int,
    uFlags: UINT,
) -> c_int {
    FIXME!("SHLookupIconIndexA() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHLookupIconIndexW(
    lpName: PCWSTR,
    iIndex: c_int,
    uFlags: UINT,
) -> c_int {
    FIXME!("SHLookupIconIndexW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn PifMgr_OpenProperties(
    lpAppPath: PCWSTR,
    lpPifPath: PCWSTR,
    hInfIndex: UINT,
    options: UINT,
) -> HANDLE {
    FIXME!("PifMgr_OpenProperties() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn PifMgr_GetProperties(
    hHandle: HANDLE,
    lpName: PCHAR,
    lpUnknown: PVOID,
    iUnknown: c_int,
    uUnknown: UINT,
) -> c_int {
    FIXME!("PifMgr_GetProperties() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn PifMgr_SetProperties(
    hHandle: HANDLE,
    lpName: PCHAR,
    lpUnknown: *const c_void,
    iUnknown: c_int,
    uUnknown: UINT,
) -> c_int {
    FIXME!("PifMgr_SetProperties() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHStartNetConnectionDialogW(
    hwnd: HWND,
    pszRemoteName: PCWSTR,
    dwType: DWORD,
) -> HRESULT {
    FIXME!("SHStartNetConnectionDialogW() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn PifMgr_CloseProperties(
    hHandle: HANDLE,
    uUnknown: UINT,
) -> HANDLE {
    FIXME!("PifMgr_CloseProperties() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn DAD_DragEnterEx2(
    hwndTarget: HWND,
    ptStart: POINT,
    pdtObject: *mut IDataObject,
) -> BOOL {
    FIXME!("DAD_DragEnterEx2() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHGetNetResource(
    lpUnknown1: PVOID,
    iIndex: UINT,
    lpUnknown2: PVOID,
    cchMax: UINT,
) -> UINT {
    FIXME!("SHGetNetResource() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn DragQueryInfo(
    hDrop: HDROP,
    pDragInfo: *mut DRAGINFO,
) -> BOOL {
    FIXME!("DragQueryInfo() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn DDECreatePostNotify(lpUnknown: PVOID) -> PVOID {
    FIXME!("DDECreatePostNotify() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn AppCompat_RunDLLW(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PWSTR,
    nCmdShow: c_int,
) {
    FIXME!("AppCompat_RunDLLW() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn Control_RunDLLAsUserW(
    hwnd: HWND,
    hInstance: HINSTANCE,
    pszCmdLine: PWSTR,
    nCmdShow: c_int,
) {
    FIXME!("Control_RunDLLAsUserW() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn DragQueryFileAorW(
    hDrop: HDROP,
    iIndex: UINT,
    lpFile: PWSTR,
    ucb: UINT,
    bUnicode: BOOLEAN,
    bShorten: BOOLEAN,
) -> UINT {
    FIXME!("DragQueryFileAorW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHNetConnectionDialog(
    hwndOwner: HWND,
    lpstrRemoteName: PCWSTR,
    dwType: DWORD,
) -> DWORD {
    FIXME!("SHNetConnectionDialog() stub\n");
    ERROR_INVALID_PARAMETER as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn DAD_SetDragImageFromListView(
    hwnd: HWND,
    pt: POINT,
) -> BOOL {
    FIXME!("DAD_SetDragImageFromListView() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHHandleDiskFull(
    hwndOwner: HWND,
    uDrive: UINT,
) {
    FIXME!("SHHandleDiskFull() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn ILGetPseudoNameW(
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
    szStr: PWSTR,
    iUnknown: c_int,
) -> BOOL {
    TRACE!("ILGetPseudoNameW() stub\n");
    *szStr = 0;
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHGlobalDefect(dwUnknown: DWORD) {
    TRACE!("SHGlobalDefect() stub\n");
}
#[no_mangle]
pub unsafe extern "system" fn Printers_GetPidl(
    pidl: LPCITEMIDLIST,
    lpName: PCWSTR,
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
) -> LPITEMIDLIST {
    FIXME!("Printers_GetPidl() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn Printers_AddPrinterPropPages(
    lpUnknown1: PVOID,
    lpUnknown2: PVOID,
) -> c_long {
    FIXME!("Printers_AddPrinterPropPages() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn ExtractIconResInfoW(
    hHandle: HANDLE,
    lpFileName: PCWSTR,
    wIndex: WORD,
    lpSize: *mut WORD,
    lpIcon: *mut HANDLE,
) -> WORD {
    FIXME!("ExtractIconResInfoW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn ExtractVersionResource16W(
    lpName: PWSTR,
    lpHandle: *mut HANDLE,
) -> DWORD {
    FIXME!("ExtractVersionResource16W() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn FindExeDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> *mut BOOLEAN {
    FIXME!("FindExeDlgProc() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn InternalExtractIconListW(
    hHandle: HANDLE,
    lpFileName: PWSTR,
    lpCount: *mut c_int,
) -> HANDLE {
    FIXME!("InternalExtractIconListW() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn InternalExtractIconListA(
    hHandle: HANDLE,
    lpFileName: PCHAR,
    lpCount: *mut c_int,
) -> HANDLE {
    FIXME!("InternalExtractIconListA() stub\n");
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn FirstUserLogon(
    lpUnknown1: PWSTR,
    lpUnknown2: PWSTR,
) -> HRESULT {
    FIXME!("FirstUserLogon() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHSetFolderPathW(
    csidl: c_int,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszPath: PCWSTR,
) -> HRESULT {
    FIXME!("SHSetFolderPathW() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHGetUserPicturePathW(
    lpPath: PCWSTR,
    csidl: c_int,
    lpUnknown: PVOID,
) -> HRESULT {
    FIXME!("SHGetUserPicturePathW() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn SHSetUserPicturePathW(
    lpPath: PCWSTR,
    csidl: c_int,
    lpUnknown: PVOID,
) -> HRESULT {
    FIXME!("SHGetUserPicturePathA() stub\n");
    E_FAIL
}
#[no_mangle]
pub unsafe extern "system" fn PathIsSlowW(
    pszFile: PCWSTR,
    dwAttr: DWORD,
) -> BOOL {
    FIXME!("PathIsSlowW() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn SHGetProcessDword(
    dwUnknown1: DWORD,
    dwUnknown2: DWORD,
) -> DWORD {
    TRACE!("SHGetProcessDword() stub\n");
    0
}
#[no_mangle]
pub unsafe extern "system" fn CheckStagingArea() -> DWORD {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         shell32.dll
//  * FILE:            dll/win32/shell32/stubs.c
//  * PURPOSE:         shell32.dll stubs
//  * PROGRAMMER:      Dmitry Chapyshev (dmitry@reactos.org)
//  * NOTES:           If you implement a function, remove it from this file
//  * UPDATE HISTORY:
//  *      03/02/2009  Created
//  */
//
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// CheckDiskSpace(VOID)
// {
//     FIXME("CheckDiskSpace() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// SHReValidateDarwinCache(VOID)
// {
//     FIXME("SHReValidateDarwinCache() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" FILEDESCRIPTOR*
// WINAPI
// GetFileDescriptor(FILEGROUPDESCRIPTOR *pFileGroupDesc, BOOL bUnicode, INT iIndex, LPWSTR lpName)
// {
//     FIXME("GetFileDescriptor() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// MakeShellURLFromPathW(LPCWSTR lpPath, LPWSTR lpUrl, INT cchMax)
// {
//     FIXME("MakeShellURLFromPathW() stub\n");
//     lpUrl = NULL;
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// MakeShellURLFromPathA(LPCSTR lpPath, LPSTR lpUrl, INT cchMax)
// {
//     WCHAR szPath[MAX_PATH], szURL[MAX_PATH];
//     BOOL ret;
//     SHAnsiToUnicode(lpPath, szPath, _countof(szPath));
//     ret = MakeShellURLFromPathW(szPath, szURL, _countof(szURL));
//     SHUnicodeToAnsi(szURL, lpUrl, cchMax);
//     return ret;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHParseDarwinIDFromCacheW(LPCWSTR lpUnknown1, LPWSTR lpUnknown2)
// {
//     FIXME("SHParseDarwinIDFromCacheW() stub\n");
//     lpUnknown2 = NULL;
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHCopyMonikerToTemp(IMoniker *pMoniker, LPCWSTR lpInput, LPWSTR lpOutput, INT cchMax)
// {
//     /* Unimplemented in XP SP3 */
//     TRACE("SHCopyMonikerToTemp() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HLOCAL
// WINAPI
// CheckWinIniForAssocs(VOID)
// {
//     FIXME("CheckWinIniForAssocs() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHGetSetFolderCustomSettingsW(LPSHFOLDERCUSTOMSETTINGSW pfcs,
//                               LPCWSTR pszPath,
//                               DWORD dwReadWrite)
// {
//     FIXME("SHGetSetFolderCustomSettingsW() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHGetSetFolderCustomSettingsA(LPSHFOLDERCUSTOMSETTINGSA pfcs,
//                               LPCSTR pszPath,
//                               DWORD dwReadWrite)
// {
//     FIXME("SHGetSetFolderCustomSettingsA() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// CDefFolderMenu_MergeMenu(HINSTANCE hInstance,
//                          UINT uMainMerge,
//                          UINT uPopupMerge,
//                          LPQCMINFO lpQcmInfo)
// {
//     FIXME("CDefFolderMenu_MergeMenu() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// CDefFolderMenu_Create(LPITEMIDLIST pidlFolder,
//                       HWND hwnd,
//                       UINT uidl,
//                       PCUITEMID_CHILD_ARRAY *apidl,
//                       IShellFolder *psf,
//                       LPFNDFMCALLBACK lpfn,
//                       HKEY hProgID,
//                       HKEY hBaseProgID,
//                       IContextMenu **ppcm)
// {
//     FIXME("CDefFolderMenu_Create() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// SHWaitOp_Operate(LPVOID lpUnknown1, DWORD dwUnknown2)
// {
//     FIXME("SHWaitOp_Operate() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// RealDriveTypeFlags(INT iDrive, BOOL bUnknown)
// {
//     FIXME("RealDriveTypeFlags() stub\n");
//     return 1;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" LONG
// WINAPI
// ShellHookProc(INT iCode, WPARAM wParam, LPARAM lParam)
// {
//     /* Unimplemented in WinXP SP3 */
//     TRACE("ShellHookProc() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SheShortenPathW(LPWSTR lpPath, BOOL bShorten)
// {
//     FIXME("SheShortenPathW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SheShortenPathA(LPSTR lpPath, BOOL bShorten)
// {
//     BOOL ret;
//     WCHAR szPath[MAX_PATH];
//     MultiByteToWideChar(CP_ACP, 0, lpPath, -1, szPath, _countof(szPath));
//     ret = SheShortenPathW(szPath, bShorten);
//     WideCharToMultiByte(CP_ACP, 0, szPath, -1, lpPath, MAX_PATH, NULL, NULL);
//     return ret;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheSetCurDrive(INT iIndex)
// {
//     FIXME("SheSetCurDrive() stub\n");
//     return 1;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheGetPathOffsetW(LPWSTR lpPath)
// {
//     FIXME("SheGetPathOffsetW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SheGetDirExW(LPWSTR lpDrive,
//              LPDWORD lpCurDirLen,
//              LPWSTR lpCurDir)
// {
//     FIXME("SheGetDirExW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheGetCurDrive(VOID)
// {
//     FIXME("SheGetCurDrive() stub\n");
//     return 1;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheFullPathW(LPWSTR lpFullName, DWORD dwPathSize, LPWSTR lpBuffer)
// {
//     FIXME("SheFullPathW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheFullPathA(LPSTR lpFullName, DWORD dwPathSize, LPSTR lpBuffer)
// {
//     FIXME("SheFullPathA() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SheConvertPathW(LPWSTR lpCmd, LPWSTR lpFileName, UINT uCmdLen)
// {
//     FIXME("SheConvertPathW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheChangeDirExW(LPWSTR lpDir)
// {
//     FIXME("SheChangeDirExW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SheChangeDirExA(LPSTR lpDir)
// {
//     FIXME("SheChangeDirExA() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SHInvokePrinterCommandW(HWND hwnd,
//                         UINT uAction,
//                         LPCWSTR lpBuf1,
//                         LPCWSTR lpBuf2,
//                         BOOL fModal)
// {
//     FIXME("SHInvokePrinterCommandW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SHInvokePrinterCommandA(HWND hwnd,
//                         UINT uAction,
//                         LPCSTR lpBuf1,
//                         LPCSTR lpBuf2,
//                         BOOL fModal)
// {
//     FIXME("SHInvokePrinterCommandA() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// SHCreateProcessAsUserW(PSHCREATEPROCESSINFOW pscpi)
// {
//     FIXME("SHCreateProcessAsUserW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// PrintersGetCommand_RunDLL(HWND hwnd, HINSTANCE hInstance, LPWSTR pszCmdLine, int nCmdShow)
// {
//     FIXME("PrintersGetCommand_RunDLL() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// PrintersGetCommand_RunDLLA(HWND hwnd, HINSTANCE hInstance, LPSTR pszCmdLine, int nCmdShow)
// {
//     FIXME("PrintersGetCommand_RunDLLA() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// PrintersGetCommand_RunDLLW(HWND hwnd, HINSTANCE hInstance, LPWSTR pszCmdLine, int nCmdShow)
// {
//     FIXME("PrintersGetCommand_RunDLLW() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" IShellFolderViewCB*
// WINAPI
// SHGetShellFolderViewCB(HWND hwnd)
// {
//     FIXME("SHGetShellFolderViewCB() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SHLookupIconIndexA(LPCSTR lpName, INT iIndex, UINT uFlags)
// {
//     FIXME("SHLookupIconIndexA() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// SHLookupIconIndexW(LPCWSTR lpName, INT iIndex, UINT uFlags)
// {
//     FIXME("SHLookupIconIndexW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HANDLE
// WINAPI
// PifMgr_OpenProperties(LPCWSTR lpAppPath, LPCWSTR lpPifPath, UINT hInfIndex, UINT options)
// {
//     FIXME("PifMgr_OpenProperties() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// PifMgr_GetProperties(HANDLE hHandle, LPCSTR lpName, LPVOID lpUnknown, INT iUnknown, UINT uUnknown)
// {
//     FIXME("PifMgr_GetProperties() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" INT
// WINAPI
// PifMgr_SetProperties(HANDLE hHandle, LPCSTR lpName, LPCVOID lpUnknown, INT iUnknown, UINT uUnknown)
// {
//     FIXME("PifMgr_SetProperties() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHStartNetConnectionDialogW(
//      HWND hwnd,
//      LPCWSTR pszRemoteName,
//      DWORD dwType)
// {
//     FIXME("SHStartNetConnectionDialogW() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HANDLE
// WINAPI
// PifMgr_CloseProperties(HANDLE hHandle, UINT uUnknown)
// {
//     FIXME("PifMgr_CloseProperties() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// DAD_DragEnterEx2(HWND hwndTarget,
//                  POINT ptStart,
//                  IDataObject *pdtObject)
// {
//     FIXME("DAD_DragEnterEx2() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" UINT
// WINAPI
// SHGetNetResource(LPVOID lpUnknown1, UINT iIndex, LPVOID lpUnknown2, UINT cchMax)
// {
//     FIXME("SHGetNetResource() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// DragQueryInfo(HDROP hDrop, DRAGINFO *pDragInfo)
// {
//     FIXME("DragQueryInfo() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" LPVOID
// WINAPI
// DDECreatePostNotify(LPVOID lpUnknown)
// {
//     FIXME("DDECreatePostNotify() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// AppCompat_RunDLLW(HWND hwnd, HINSTANCE hInstance, LPWSTR pszCmdLine, int nCmdShow)
// {
//     FIXME("AppCompat_RunDLLW() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// Control_RunDLLAsUserW(HWND hwnd, HINSTANCE hInstance, LPWSTR pszCmdLine, int nCmdShow)
// {
//     FIXME("Control_RunDLLAsUserW() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" UINT
// WINAPI
// DragQueryFileAorW(HDROP hDrop, UINT iIndex, LPWSTR lpFile, UINT ucb, BOOL bUnicode, BOOL bShorten)
// {
//     FIXME("DragQueryFileAorW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" DWORD
// WINAPI
// SHNetConnectionDialog(HWND hwndOwner,
//                       LPCWSTR lpstrRemoteName,
//                       DWORD dwType)
// {
//     FIXME("SHNetConnectionDialog() stub\n");
//     return ERROR_INVALID_PARAMETER;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// DAD_SetDragImageFromListView(HWND hwnd, POINT pt)
// {
//     FIXME("DAD_SetDragImageFromListView() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" void
// WINAPI
// SHHandleDiskFull(HWND hwndOwner, UINT uDrive)
// {
//     FIXME("SHHandleDiskFull() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// ILGetPseudoNameW(LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2, LPWSTR szStr, INT iUnknown)
// {
//     /* Unimplemented in WinXP SP3 */
//     TRACE("ILGetPseudoNameW() stub\n");
//     *szStr = 0;
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" VOID
// WINAPI
// SHGlobalDefect(DWORD dwUnknown)
// {
//     /* Unimplemented in WinXP SP3 */
//     TRACE("SHGlobalDefect() stub\n");
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" LPITEMIDLIST
// WINAPI
// Printers_GetPidl(LPCITEMIDLIST pidl, LPCWSTR lpName, DWORD dwUnknown1, DWORD dwUnknown2)
// {
//     FIXME("Printers_GetPidl() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" LONG
// WINAPI
// Printers_AddPrinterPropPages(LPVOID lpUnknown1, LPVOID lpUnknown2)
// {
//     FIXME("Printers_AddPrinterPropPages() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" WORD
// WINAPI
// ExtractIconResInfoW(
//      HANDLE hHandle,
//      LPCWSTR lpFileName,
//      WORD wIndex,
//      LPWORD lpSize,
//      LPHANDLE lpIcon)
// {
//     FIXME("ExtractIconResInfoW() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" DWORD
// WINAPI
// ExtractVersionResource16W(LPWSTR lpName, LPHANDLE lpHandle)
// {
//     FIXME("ExtractVersionResource16W() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL*
// WINAPI
// FindExeDlgProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     FIXME("FindExeDlgProc() stub\n");
//     return 0;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HANDLE
// WINAPI
// InternalExtractIconListW(HANDLE hHandle,
//                          LPWSTR lpFileName,
//                          LPINT lpCount)
// {
//     FIXME("InternalExtractIconListW() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HANDLE
// WINAPI
// InternalExtractIconListA(HANDLE hHandle,
//                          LPSTR lpFileName,
//                          LPINT lpCount)
// {
//     FIXME("InternalExtractIconListA() stub\n");
//     return NULL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// FirstUserLogon(LPWSTR lpUnknown1, LPWSTR lpUnknown2)
// {
//     FIXME("FirstUserLogon() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHSetFolderPathW(
//      INT csidl,
//      HANDLE hToken,
//      DWORD dwFlags,
//      LPCWSTR pszPath)
// {
//     FIXME("SHSetFolderPathW() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHGetUserPicturePathW(LPCWSTR lpPath, int csidl, LPVOID lpUnknown)
// {
//     FIXME("SHGetUserPicturePathW() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" HRESULT
// WINAPI
// SHSetUserPicturePathW(LPCWSTR lpPath, int csidl, LPVOID lpUnknown)
// {
//     FIXME("SHGetUserPicturePathA() stub\n");
//     return E_FAIL;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" BOOL
// WINAPI
// PathIsSlowW(
//      LPCWSTR pszFile,
//      DWORD dwAttr)
// {
//     FIXME("PathIsSlowW() stub\n");
//     return FALSE;
// }
//
// /*
//  * Unimplemented
//  */
// extern "C" DWORD
// WINAPI
// SHGetProcessDword(DWORD dwUnknown1, DWORD dwUnknown2)
// {
//     /* Unimplemented in WinXP SP3 */
//     TRACE("SHGetProcessDword() stub\n");
//     return 0;
// }
//
// extern "C"
// DWORD WINAPI CheckStagingArea(VOID)
// {
//     /* Called by native explorer */
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
