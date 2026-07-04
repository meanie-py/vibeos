//! startctxmnu.cpp — `CStartMenuBtnCtxMenu`, the Start-button context menu.
//! A pure-COM tier-C class: it implements exactly ONE interface, `IContextMenu`
//! (so `object == interface`, single `lpVtbl`-first — the simplest COM shape,
//! like trayclock but with no window). It both IMPLEMENTS `IContextMenu`
//! (QueryContextMenu/InvokeCommand/GetCommandString are its own vtable slots)
//! and CONSUMES it (m_Inner, the shell folder's own context menu, is called
//! through the same vtable) plus consumes `IShellFolder` (m_Folder, psfDesktop)
//! and `ITrayWindow` (m_TrayWnd).
//!
//! The class reserves the whole `ID_SHELL_CMD_*` id range for its own items and
//! offsets the inner (shell-folder) menu's ids by `INNERIDOFFSET` so the two
//! id spaces never collide — see the comment on `INNERIDOFFSET` below and the
//! long note in the C++ original.
//!
//! CComPtr members (m_TrayWnd/m_Inner/m_Folder) become raw pointers with the
//! AddRef-on-assign / Release-on-final-Release discipline done by hand; the
//! dtor also `ILFree`s m_FolderPidl.

use core::ffi::c_void;
use explorer_sys::com::{
    IContextMenu, IContextMenuVtbl, IShellFolder, CMINVOKECOMMANDINFO, CMINVOKECOMMANDINFOEX,
    CMF_VERBSONLY, CMIC_MASK_UNICODE, E_FAIL, E_NOTIMPL, E_OUTOFMEMORY, E_POINTER, ID_SHELL_CMD_FIRST,
    ID_SHELL_CMD_LAST, ID_SHELL_CMD_PROPERTIES, ID_SHELL_CMD_OPEN_ALL_USERS,
    ID_SHELL_CMD_EXPLORE_ALL_USERS, IID_IContextMenu, IID_IShellFolder, LPCITEMIDLIST, LPITEMIDLIST,
    REFIID, S_FALSE, S_OK,
};
use explorer_sys::externs::{
    IsUserAnAdmin, SHAnsiToUnicode, SHCloneSpecialIDList, SHGetDesktopFolder, SHGetFolderPathW,
    SHGetPathFromIDListW, SHRestricted, SHUnicodeToAnsi, ILClone, ILFindLastID, ILFree,
    ILRemoveLastID, CSIDL_COMMON_STARTMENU, CSIDL_STARTMENU, IDS_EXPLORE_ALL_USERS,
    IDS_OPEN_ALL_USERS, IDS_PROPERTIES, REST_NOCOMMONGROUPS, SHGFP_TYPE_CURRENT,
};
use explorer_sys::com::{IID_IUnknown, ITrayWindow};
use explorer_sys::globals::{hExplorerInstance, hProcessHeap};
use explorer_sys::server::{interlocked_decrement, interlocked_increment};
use winsrv_sys::winsrv::{
    AppendMenuW, HeapAlloc, HeapFree, LoadStringW, BOOL, DWORD, HMENU, HRESULT, HWND, IID, LONG,
    LPCSTR, LPSTR, MF_SEPARATOR, MF_STRING, SIZE_T, SW_NORMAL, UINT, UINT_PTR,
};

const MAX_PATH: usize = 260;

/// AddStartContextMenuItems uses ID_SHELL_CMD ids directly and relies on
/// idCmdFirst being 0; m_Inner->QueryContextMenu is not aware of this, so we
/// reserve the entire ID_SHELL_CMD range for ourselves and push the inner
/// menu's ids past it by INNERIDOFFSET. (See the original comment.)
const INNERIDOFFSET: UINT = ID_SHELL_CMD_LAST + 1;

#[inline]
fn is_shell_cmd_id(id: UINT_PTR) -> BOOL {
    (id < INNERIDOFFSET as UINT_PTR) as BOOL
}

// C_ASSERTs from the C++ (compile-time invariants the id game relies on).
const _: () = assert!(ID_SHELL_CMD_FIRST != 0);
const _: () = assert!(ID_SHELL_CMD_LAST < 0xffff / 2);

// ---- the object (single-interface COM: object == IContextMenu) ----

#[repr(C)]
pub struct CStartMenuBtnCtxMenu {
    pub lpVtbl: *const IContextMenuVtbl,
    pub ref_: LONG,
    pub m_TrayWnd: *mut ITrayWindow, // CComPtr
    pub m_Inner: *mut IContextMenu,  // CComPtr
    pub m_Folder: *mut IShellFolder, // CComPtr
    pub m_Owner: HWND,
    pub m_FolderPidl: LPITEMIDLIST,
}

#[inline]
unsafe fn guid_eq(a: REFIID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

// ---- IUnknown (BEGIN_COM_MAP: COM_INTERFACE_ENTRY_IID(IID_IContextMenu)) ----

unsafe extern "system" fn ctxmenu_QueryInterface(
    iface: *mut IContextMenu,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IContextMenu) {
        *ppv = this as *mut c_void;
        ctxmenu_AddRef(iface);
        return S_OK;
    }
    *ppv = core::ptr::null_mut();
    explorer_sys::com::E_NOINTERFACE
}

unsafe extern "system" fn ctxmenu_AddRef(iface: *mut IContextMenu) -> u32 {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    interlocked_increment(&mut (*this).ref_) as u32
}

unsafe extern "system" fn ctxmenu_Release(iface: *mut IContextMenu) -> u32 {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    let n = interlocked_decrement(&mut (*this).ref_);
    if n == 0 {
        // ~CStartMenuBtnCtxMenu(): ILFree(m_FolderPidl) + CComPtr member release.
        if !(*this).m_FolderPidl.is_null() {
            ILFree((*this).m_FolderPidl);
        }
        if !(*this).m_Folder.is_null() {
            let f = (*this).m_Folder;
            ((*(*f).lpVtbl).Release)(f);
        }
        if !(*this).m_Inner.is_null() {
            let i = (*this).m_Inner;
            ((*(*i).lpVtbl).Release)(i);
        }
        if !(*this).m_TrayWnd.is_null() {
            let t = (*this).m_TrayWnd;
            ((*(*t).lpVtbl).Release)(t);
        }
        HeapFree(hProcessHeap, 0, this as *mut c_void);
        return 0;
    }
    n as u32
}

// ---- private helpers (CStartMenuBtnCtxMenu::*) ----

unsafe fn CreateContextMenuFromShellFolderPidl(
    this: *mut CStartMenuBtnCtxMenu,
    hPopup: HMENU,
    idCmdFirst: UINT,
    idCmdLast: UINT,
) -> HRESULT {
    // m_Folder->GetUIObjectOf(m_Owner, 1, (LPCITEMIDLIST*)&m_FolderPidl,
    //                         IID_NULL_PPV_ARG(IContextMenu, &m_Inner))
    // IID_NULL_PPV_ARG expands to: &IID_IContextMenu, NULL (prgfInOut), &m_Inner.
    let folder = (*this).m_Folder;
    let mut hRet = ((*(*folder).lpVtbl).GetUIObjectOf)(
        folder,
        (*this).m_Owner,
        1,
        core::ptr::addr_of_mut!((*this).m_FolderPidl) as *const LPCITEMIDLIST,
        &IID_IContextMenu,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*this).m_Inner) as *mut *mut c_void,
    );
    if hRet >= 0 {
        if !hPopup.is_null() {
            let inner = (*this).m_Inner;
            hRet = ((*(*inner).lpVtbl).QueryContextMenu)(
                inner,
                hPopup,
                0,
                idCmdFirst,
                idCmdLast,
                CMF_VERBSONLY,
            );
            if hRet >= 0 {
                return hRet;
            }
        }
    }
    E_FAIL
}

unsafe fn AddStartContextMenuItems(this: *mut CStartMenuBtnCtxMenu, hPopup: HMENU) {
    let mut szBuf: [u16; MAX_PATH] = [0; MAX_PATH];

    // Add the "Properties" menu item.
    if LoadStringW(
        hExplorerInstance,
        IDS_PROPERTIES,
        szBuf.as_mut_ptr(),
        szBuf.len() as i32,
    ) != 0
    {
        AppendMenuW(
            hPopup,
            MF_STRING,
            ID_SHELL_CMD_PROPERTIES as UINT_PTR,
            szBuf.as_ptr(),
        );
    }

    if SHRestricted(REST_NOCOMMONGROUPS) == 0 {
        // Check if we should add menu items for the common start menu.
        let hRet = SHGetFolderPathW(
            (*this).m_Owner,
            CSIDL_COMMON_STARTMENU,
            core::ptr::null_mut(),
            SHGFP_TYPE_CURRENT,
            szBuf.as_mut_ptr(),
        );
        if hRet >= 0 && hRet != S_FALSE {
            // The directory exists, but only show the items if the user can
            // actually make changes to the common start menu — most likely
            // only if the user has administrative rights.
            if IsUserAnAdmin() != 0 {
                AppendMenuW(hPopup, MF_SEPARATOR, 0, core::ptr::null());

                // "Open All Users"
                if LoadStringW(
                    hExplorerInstance,
                    IDS_OPEN_ALL_USERS,
                    szBuf.as_mut_ptr(),
                    szBuf.len() as i32,
                ) != 0
                {
                    AppendMenuW(
                        hPopup,
                        MF_STRING,
                        ID_SHELL_CMD_OPEN_ALL_USERS as UINT_PTR,
                        szBuf.as_ptr(),
                    );
                }

                // "Explore All Users"
                if LoadStringW(
                    hExplorerInstance,
                    IDS_EXPLORE_ALL_USERS,
                    szBuf.as_mut_ptr(),
                    szBuf.len() as i32,
                ) != 0
                {
                    AppendMenuW(
                        hPopup,
                        MF_STRING,
                        ID_SHELL_CMD_EXPLORE_ALL_USERS as UINT_PTR,
                        szBuf.as_ptr(),
                    );
                }
            }
        }
    }
}

// ---- Initialize (public) ----

unsafe fn CStartMenuBtnCtxMenu_Initialize(
    this: *mut CStartMenuBtnCtxMenu,
    pTrayWnd: *mut ITrayWindow,
    hWndOwner: HWND,
) -> HRESULT {
    // m_TrayWnd = pTrayWnd (CComPtr assign: AddRef new).
    if !pTrayWnd.is_null() {
        ((*(*pTrayWnd).lpVtbl).AddRef)(pTrayWnd);
    }
    (*this).m_TrayWnd = pTrayWnd;
    (*this).m_Owner = hWndOwner;
    S_OK
}

// ---- IContextMenu methods ----

unsafe extern "system" fn ctxmenu_QueryContextMenu(
    iface: *mut IContextMenu,
    hPopup: HMENU,
    _indexMenu: UINT,
    idCmdFirst: UINT,
    idCmdLast: UINT,
    _uFlags: UINT,
) -> HRESULT {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    let mut hRet: HRESULT = S_OK;
    let idInnerFirst: UINT = idCmdFirst.wrapping_add(INNERIDOFFSET);

    // CComPtr<IShellFolder> psfDesktop; (local, starts NULL)
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();

    // m_Inner = NULL; (release old, set null)
    if !(*this).m_Inner.is_null() {
        let i = (*this).m_Inner;
        ((*(*i).lpVtbl).Release)(i);
        (*this).m_Inner = core::ptr::null_mut();
    }

    let pidlStart = SHCloneSpecialIDList((*this).m_Owner, CSIDL_STARTMENU, 1);
    if !pidlStart.is_null() {
        (*this).m_FolderPidl = ILClone(ILFindLastID(pidlStart));
        ILRemoveLastID(pidlStart);

        if !(*this).m_FolderPidl.is_null() {
            hRet = SHGetDesktopFolder(&mut psfDesktop);
            if hRet >= 0 {
                // psfDesktop->BindToObject(pidlStart, NULL,
                //     IID_PPV_ARG(IShellFolder, &m_Folder))
                hRet = ((*(*psfDesktop).lpVtbl).BindToObject)(
                    psfDesktop,
                    pidlStart,
                    core::ptr::null_mut(),
                    &IID_IShellFolder,
                    core::ptr::addr_of_mut!((*this).m_Folder) as *mut *mut c_void,
                );
                if hRet >= 0 {
                    hRet =
                        CreateContextMenuFromShellFolderPidl(this, hPopup, idInnerFirst, idCmdLast);
                }
            }
        }

        ILFree(pidlStart);
    }

    if idCmdLast.wrapping_sub(idCmdFirst) >= ID_SHELL_CMD_LAST.wrapping_sub(ID_SHELL_CMD_FIRST) {
        AddStartContextMenuItems(this, hPopup);
        hRet = if hRet >= 0 {
            hRet.wrapping_add(idInnerFirst as HRESULT)
        } else {
            idInnerFirst as HRESULT
        };
    }

    // ~CComPtr psfDesktop
    if !psfDesktop.is_null() {
        ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
    }

    hRet
}

unsafe extern "system" fn ctxmenu_InvokeCommand(
    iface: *mut IContextMenu,
    lpici: *const CMINVOKECOMMANDINFO,
) -> HRESULT {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    let uiCmdId: UINT = (*lpici).lpVerb as usize as UINT; // PtrToUlong(lpici->lpVerb)

    if is_shell_cmd_id(uiCmdId as UINT_PTR) == 0 {
        let mut cmici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
        cmici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as DWORD;

        // Setup and invoke the shell command.
        cmici.hwnd = (*this).m_Owner;
        cmici.nShow = SW_NORMAL as i32;
        cmici.fMask = CMIC_MASK_UNICODE;

        let mut szVerbW: [u16; MAX_PATH] = [0; MAX_PATH];
        // IS_INTRESOURCE(lpici->lpVerb): ((ULONG_PTR)lpVerb >> 16) == 0
        if ((*lpici).lpVerb as usize) >> 16 == 0 {
            // MAKEINTRESOURCEA/W(uiCmdId - INNERIDOFFSET) — LOWORD as pointer.
            let low = (uiCmdId.wrapping_sub(INNERIDOFFSET) & 0xFFFF) as usize;
            cmici.lpVerb = low as LPCSTR;
            cmici.lpVerbW = low as *const u16;
        } else {
            cmici.lpVerb = (*lpici).lpVerb;
            SHAnsiToUnicode((*lpici).lpVerb, szVerbW.as_mut_ptr(), szVerbW.len() as i32);
            cmici.lpVerbW = szVerbW.as_ptr();
        }

        let mut szDirA: [u8; MAX_PATH] = [0; MAX_PATH];
        let mut szDirW: [u16; MAX_PATH] = [0; MAX_PATH];
        if SHGetPathFromIDListW((*this).m_FolderPidl, szDirW.as_mut_ptr()) != 0 {
            SHUnicodeToAnsi(szDirW.as_ptr(), szDirA.as_mut_ptr() as LPSTR, szDirA.len() as i32);
            cmici.lpDirectory = szDirA.as_ptr() as LPCSTR;
            cmici.lpDirectoryW = szDirW.as_ptr();
        }

        let inner = (*this).m_Inner;
        return ((*(*inner).lpVtbl).InvokeCommand)(
            inner,
            &cmici as *const CMINVOKECOMMANDINFOEX as *const CMINVOKECOMMANDINFO,
        );
    }

    // m_TrayWnd->ExecContextMenuCmd(uiCmdId)
    let tw = (*this).m_TrayWnd;
    ((*(*tw).lpVtbl).ExecContextMenuCmd)(tw, uiCmdId);
    S_OK
}

unsafe extern "system" fn ctxmenu_GetCommandString(
    iface: *mut IContextMenu,
    idCmd: UINT_PTR,
    uType: UINT,
    pwReserved: *mut UINT,
    pszName: LPSTR,
    cchMax: UINT,
) -> HRESULT {
    let this = iface as *mut CStartMenuBtnCtxMenu;
    if is_shell_cmd_id(idCmd) == 0 && !(*this).m_Inner.is_null() {
        let inner = (*this).m_Inner;
        return ((*(*inner).lpVtbl).GetCommandString)(
            inner, idCmd, uType, pwReserved, pszName, cchMax,
        );
    }
    E_NOTIMPL
}

// ---- static vtable ----

static vt_CStartMenuBtnCtxMenu: IContextMenuVtbl = IContextMenuVtbl {
    QueryInterface: ctxmenu_QueryInterface,
    AddRef: ctxmenu_AddRef,
    Release: ctxmenu_Release,
    QueryContextMenu: ctxmenu_QueryContextMenu,
    InvokeCommand: ctxmenu_InvokeCommand,
    GetCommandString: ctxmenu_GetCommandString,
};

// ---- construction + factory ----

unsafe fn CStartMenuBtnCtxMenu_ctor(this: *mut CStartMenuBtnCtxMenu) {
    (*this).lpVtbl = &vt_CStartMenuBtnCtxMenu;
    (*this).ref_ = 0;
    (*this).m_TrayWnd = core::ptr::null_mut();
    (*this).m_Inner = core::ptr::null_mut();
    (*this).m_Folder = core::ptr::null_mut();
    (*this).m_Owner = core::ptr::null_mut();
    (*this).m_FolderPidl = core::ptr::null_mut();
}

/// `CStartMenuBtnCtxMenu_CreateInstance` —
/// `ShellObjectCreatorInit<CStartMenuBtnCtxMenu>(m_TrayWnd, m_Owner,
///  IID_PPV_ARG(IContextMenu, ppCtxMenu))`:
/// HeapAlloc + ctor(ref=0) → AddRef → Initialize(pTrayWnd, hWndOwner) →
/// QI(IID_IContextMenu, ppv) → Release.
#[no_mangle]
pub unsafe extern "C" fn CStartMenuBtnCtxMenu_CreateInstance(
    m_TrayWnd: *mut ITrayWindow,
    m_Owner: HWND,
    ppCtxMenu: *mut *mut IContextMenu,
) -> HRESULT {
    let pobj = HeapAlloc(
        hProcessHeap,
        0,
        core::mem::size_of::<CStartMenuBtnCtxMenu>() as SIZE_T,
    ) as *mut CStartMenuBtnCtxMenu;
    if pobj.is_null() {
        return E_OUTOFMEMORY;
    }
    CStartMenuBtnCtxMenu_ctor(pobj);

    let iface = pobj as *mut IContextMenu;
    ctxmenu_AddRef(iface);

    let mut hr = CStartMenuBtnCtxMenu_Initialize(pobj, m_TrayWnd, m_Owner);
    if hr >= 0 {
        hr = ctxmenu_QueryInterface(
            iface,
            &IID_IContextMenu,
            ppCtxMenu as *mut *mut c_void,
        );
    }

    ctxmenu_Release(iface);
    hr
}
