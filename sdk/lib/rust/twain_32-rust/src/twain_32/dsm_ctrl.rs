// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_19D6725254DA: [u16; 23] = [
    0x0063u16, 0x003A, 0x005C, 0x0077, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073,
    0x005C, 0x0074, 0x0077, 0x0061, 0x0069, 0x006E, 0x005F, 0x0025, 0x0075, 0x005C,
    0x0025, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_1A5C31AA19B8: [u16; 8] = [0x0073u16, 0x0061, 0x006E, 0x0065, 0x002E, 0x0064, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_7DFE8136F3DF: [u16; 11] = [
    0x0067u16, 0x0070, 0x0068, 0x006F, 0x0074, 0x006F, 0x0032, 0x002E, 0x0064, 0x0073,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut DSM_initialized: TW_UINT16 = 0;

static mut DSM_sourceId: TW_UINT32 = 0;

static mut DSM_currentDevice: TW_UINT16 = 0;

static mut DSM_parent: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut event_message: UINT = 0;

static mut nrdevices: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

static mut devices: *mut all_devices = ::core::ptr::null::<all_devices>() as *mut all_devices;

static mut detectionrun: BOOL = (FALSE as i32);

// WINE_DEFAULT_DEBUG_CHANNEL(twain) is a debug-channel macro invocation, not a declaration; it is omitted.

#[repr(C)]
pub struct all_devices {
    pub modname: *mut u16,
    pub identity: TW_IDENTITY,
}

#[repr(C)]
pub struct userselect_data {
    pub origin: pTW_IDENTITY,
    pub result: pTW_IDENTITY,
}

unsafe extern "C" fn twain_add_onedriver(dsname: *const WCHAR) {
    let mut hmod: HMODULE = core::ptr::null_mut();
    let mut dsEntry: DSENTRYPROC = None;
    let mut fakeOrigin: TW_IDENTITY = core::mem::zeroed();
    let mut sourceId: TW_IDENTITY = core::mem::zeroed();
    let mut new_devices: *mut all_devices = core::ptr::null_mut();
    let mut ret: TW_UINT16 = 0;
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    {
        swprintf(
            core::ptr::addr_of_mut!(path) as *mut WCHAR,
            MAX_PATH as usize,
            RUSTCONV_WSTR_19D6725254DA.as_ptr(),
            (core::mem::size_of::<*mut ::core::ffi::c_void>() * 8) as ::core::ffi::c_int,
            dsname,
        );
    }
    hmod = LoadLibraryW(core::ptr::addr_of_mut!(path) as *mut WCHAR as LPCWSTR) as HMODULE;
    if hmod.is_null() {
        ERR!(
            "Failed to load TWAIN Source %s\n",
            debugstr_w(core::ptr::addr_of_mut!(path) as *mut WCHAR)
        );
        return;
    }
    dsEntry = core::mem::transmute::<FARPROC, DSENTRYPROC>(GetProcAddress(
        hmod as HINSTANCE,
        b"DS_Entry\0" as *const u8 as LPCSTR,
    ));
    if dsEntry.is_none() {
        ERR!(
            "Failed to find DS_Entry() in TWAIN DS %s\n",
            debugstr_w(core::ptr::addr_of_mut!(path) as *mut WCHAR)
        );
        return;
    }
    /* Loop to do multiple detects, mostly for sane.ds and gphoto2.ds */
    loop {
        let mut i: ::core::ffi::c_int = 0;

        sourceId.Id = DSM_sourceId;
        sourceId.ProtocolMajor = TWON_PROTOCOLMAJOR as TW_UINT16;
        sourceId.ProtocolMinor = TWON_PROTOCOLMINOR as TW_UINT16;
        ret = dsEntry.unwrap_unchecked()(
            core::ptr::addr_of_mut!(fakeOrigin),
            DG_CONTROL as TW_UINT32,
            DAT_IDENTITY as TW_UINT16,
            MSG_GET as TW_UINT16,
            core::ptr::addr_of_mut!(sourceId) as TW_MEMREF,
        );
        if ret != TWRC_SUCCESS as TW_UINT16 {
            ERR!("Source->(DG_CONTROL,DAT_IDENTITY,MSG_GET) failed!\n");
            break;
        }

        i = 0;
        while i < nrdevices {
            if strcmp(
                core::ptr::addr_of_mut!(sourceId.ProductName) as *mut ::core::ffi::c_char,
                core::ptr::addr_of_mut!((*devices.offset(i as isize)).identity.ProductName)
                    as *mut ::core::ffi::c_char,
            ) == 0
            {
                break;
            }
            i += 1;
        }
        if i < nrdevices {
            break;
        }
        new_devices = realloc(
            devices as *mut ::core::ffi::c_void,
            (core::mem::size_of_val(&*devices.offset(0 as isize)))
                .wrapping_mul((nrdevices + 1) as usize),
        ) as *mut all_devices;
        if new_devices.is_null() {
            break;
        }
        devices = new_devices;
        (*devices.offset(nrdevices as isize)).modname =
            wcsdup(core::ptr::addr_of_mut!(path) as *mut WCHAR) as *mut WCHAR;
        (*devices.offset(nrdevices as isize)).identity = sourceId;
        nrdevices += 1;
        DSM_sourceId = DSM_sourceId.wrapping_add(1);

        if (1) == 0 {
            break;
        }
    }
    FreeLibrary(hmod);
}

unsafe extern "C" fn twain_autodetect() {
    if detectionrun != 0 {
        return;
    }
    detectionrun = (TRUE as i32);

    twain_add_onedriver(RUSTCONV_WSTR_1A5C31AA19B8.as_ptr());
    twain_add_onedriver(RUSTCONV_WSTR_7DFE8136F3DF.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_ControlNull(mut pOrigin: pTW_IDENTITY, mut pDest: pTW_IDENTITY, mut pSource: *mut activeDS, mut MSG: TW_UINT16, mut pData: TW_MEMREF) -> TW_UINT16 {
    let mut message: *mut pending_message = core::ptr::null_mut();
    if MSG != (MSG_CLOSEDSREQ as TW_UINT16) &&
        MSG != (MSG_DEVICEEVENT as TW_UINT16) &&
        MSG != (MSG_XFERREADY as TW_UINT16)
    {
        DSM_twCC = (TWCC_BADPROTOCOL as u16);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    message = HeapAlloc(GetProcessHeap(), 0 as DWORD, core::mem::size_of::<pending_message>() as SIZE_T) as *mut pending_message;
    if message.is_null() {
        DSM_twCC = (TWCC_LOWMEMORY as u16);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    (*message).msg = MSG;
    list_add_tail(core::ptr::addr_of_mut!((*pSource).pending_messages), core::ptr::addr_of_mut!((*message).entry));
    /* Delphi twain only sends us messages from one window, and it
       doesn't even give us the real handle to that window. Other
       applications might decide to forward messages sent to DSM_parent
       or to the one supplied to ENABLEDS. So let's try very hard to
       find a window that will work. */
    if !DSM_parent.is_null() {
        PostMessageW(DSM_parent, event_message, 0 as WPARAM, 0 as LPARAM);
    }
    if !(*pSource).ui_window.is_null() && (*pSource).ui_window != DSM_parent {
        PostMessageW((*pSource).ui_window, event_message, 0 as WPARAM, 0 as LPARAM);
    }
    if !(*pSource).event_window.is_null() && (*pSource).event_window != (*pSource).ui_window &&
        (*pSource).event_window != DSM_parent
    {
        PostMessageW((*pSource).event_window, event_message, 0 as WPARAM, 0 as LPARAM);
    }
    PostMessageW(core::ptr::null_mut(), event_message, 0 as WPARAM, 0 as LPARAM);
    return (TWRC_SUCCESS) as TW_UINT16;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_ProcessEvent(pOrigin: pTW_IDENTITY, pSource: *mut activeDS, pData: TW_MEMREF) -> TW_UINT16 {
    let event: *mut TW_EVENT = pData as *mut TW_EVENT;
    let msg: *mut MSG = (*event).pEvent as *mut MSG;
    let mut result: TW_UINT16 = (TWRC_NOTDSEVENT as u16);

    if (*msg).message == event_message {
        if list_empty(&raw mut (*pSource).pending_messages) == 0 {
            let entry: *mut list = list_head(&raw mut (*pSource).pending_messages);
            let message: *mut pending_message = (entry as *mut u8)
                .offset(-(core::mem::offset_of!(pending_message, entry) as isize))
                as *mut pending_message;
            (*event).TWMessage = (*message).msg;
            list_remove(entry);
        } else {
            (*event).TWMessage = (MSG_NULL as u16);
        }
        result = (TWRC_DSEVENT as u16);
    }

    if !(*msg).hwnd.is_null() {
        let mut dummy: MSG = core::mem::zeroed();
        (*pSource).event_window = (*msg).hwnd;
        if list_empty(&raw mut (*pSource).pending_messages) == 0
            && PeekMessageW(
                &raw mut dummy,
                (*msg).hwnd,
                event_message,
                event_message,
                PM_NOREMOVE,
            ) == 0
        {
            PostMessageW(
                (*msg).hwnd,
                event_message,
                0,
                0,
            );
        }
    }

    return result;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_CloseDS(mut pOrigin: pTW_IDENTITY, mut pData: TW_MEMREF) -> TW_UINT16 {
    let mut twRC: TW_UINT16 = TWRC_SUCCESS as TW_UINT16;
    let mut pIdentity: pTW_IDENTITY = pData as pTW_IDENTITY;
    let mut currentDS: *mut activeDS = ::core::ptr::null_mut::<activeDS>();
    let mut prevDS: *mut activeDS = ::core::ptr::null_mut::<activeDS>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/twain_32/dsm_ctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TWAIN_CloseDS\0" as *const u8 as *const ::core::ffi::c_char,
            212 as ::core::ffi::c_int,
            b"DG_CONTROL/DAT_IDENTITY/MSG_CLOSEDS\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    currentDS = activeSources;
    while !currentDS.is_null() {
        if (*currentDS).identity.Id == (*pIdentity).Id {
            break;
        }
        prevDS = currentDS;
        currentDS = (*currentDS).next as *mut activeDS;
    }
    if currentDS.is_null() {
        DSM_twCC = TWCC_NODS as TW_UINT16;
        return TWRC_FAILURE as TW_UINT16;
    }
    twRC = (*currentDS).dsEntry.unwrap_unchecked()(
        pOrigin,
        DG_CONTROL as TW_UINT32,
        DAT_IDENTITY as TW_UINT16,
        MSG_CLOSEDS as TW_UINT16,
        pData,
    );
    if !prevDS.is_null() {
        (*prevDS).next = (*currentDS).next;
    } else {
        activeSources = (*currentDS).next as *mut activeDS;
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        currentDS as PVOID,
    );
    if twRC as ::core::ffi::c_int == (TWRC_SUCCESS as i32) {
        DSM_twCC = TWCC_SUCCESS as TW_UINT16;
    } else {
        DSM_twCC = TWCC_SEQERROR as TW_UINT16;
    }
    return twRC;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_IdentityGetDefault(pOrigin: pTW_IDENTITY, pData: TW_MEMREF) -> TW_UINT16 {
    let mut pSourceIdentity: pTW_IDENTITY = pData as pTW_IDENTITY;

    DSM_twCC = TWCC_NODS as TW_UINT16;
    twain_autodetect();
    if nrdevices == 0 {
        return TWRC_FAILURE as TW_UINT16;
    }
    *pSourceIdentity = (*devices.offset(0 as isize)).identity;
    DSM_twCC = TWCC_SUCCESS as TW_UINT16;
    return TWRC_SUCCESS as TW_UINT16;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_IdentityGetFirst(pOrigin: pTW_IDENTITY, pData: TW_MEMREF) -> TW_UINT16 {
    let mut pSourceIdentity: pTW_IDENTITY = pData as pTW_IDENTITY;
    twain_autodetect();
    if nrdevices == 0 {
        DSM_twCC = TWCC_NODS as TW_UINT16;
        return TWRC_FAILURE as TW_UINT16;
    }
    DSM_currentDevice = 0 as TW_UINT16;
    let fresh0 = DSM_currentDevice;
    DSM_currentDevice = DSM_currentDevice.wrapping_add(1);
    *pSourceIdentity = (*devices.offset(fresh0 as isize)).identity;
    return TWRC_SUCCESS as TW_UINT16;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_IdentityGetNext(mut pOrigin: pTW_IDENTITY, mut pData: TW_MEMREF) -> TW_UINT16 {
    let mut pSourceIdentity: pTW_IDENTITY = pData as pTW_IDENTITY;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/twain_32/dsm_ctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TWAIN_IdentityGetNext\0" as *const u8 as *const ::core::ffi::c_char,
            276 as ::core::ffi::c_int,
            b"DG_CONTROL/DAT_IDENTITY/MSG_GETNEXT\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    if !(nrdevices != 0) || DSM_currentDevice as ::core::ffi::c_int == nrdevices {
        DSM_twCC = (TWCC_SUCCESS as u16);
        return (TWRC_ENDOFLIST) as TW_UINT16;
    }
    let fresh17 = DSM_currentDevice;
    DSM_currentDevice = DSM_currentDevice.wrapping_add(1);
    *pSourceIdentity = (*devices.offset(fresh17 as isize)).identity;
    return (TWRC_SUCCESS) as TW_UINT16;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_OpenDS(pOrigin: pTW_IDENTITY, pData: TW_MEMREF) -> TW_UINT16 {
    let mut i: TW_UINT16 = 0;
    let mut pIdentity: pTW_IDENTITY = pData as pTW_IDENTITY;
    let mut newSource: *mut activeDS;
    let mut hmod: HMODULE;
    if DSM_initialized == 0 {
        DSM_twCC = (TWCC_SEQERROR as u16);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    twain_autodetect();
    if nrdevices == 0 {
        DSM_twCC = (TWCC_NODS as u16);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    if (*pIdentity).ProductName[0] as ::core::ffi::c_int != '\0' as i32 {
        /* Make sure the source to be opened exists in the device list */
        i = 0;
        while (i as ::core::ffi::c_int) < nrdevices {
            if strcmp(
                &raw mut (*devices.offset(i as isize)).identity.ProductName as *mut ::core::ffi::c_char,
                &raw mut (*pIdentity).ProductName as *mut ::core::ffi::c_char,
            ) == 0
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i as ::core::ffi::c_int == nrdevices {
            i = 0;
        }
    } /* else use the first device */

    /* the source is found in the device list */
    newSource = HeapAlloc(
        GetProcessHeap(),
        0,
        ::core::mem::size_of::<activeDS>() as SIZE_T,
    ) as *mut activeDS;
    if newSource.is_null() {
        DSM_twCC = (TWCC_LOWMEMORY as u16);
        /* Out of memory. */
        return (TWRC_FAILURE) as TW_UINT16;
    }
    hmod = LoadLibraryW((*devices.offset(i as isize)).modname as LPCWSTR) as HMODULE;
    if hmod.is_null() {
        ERR!(
            "Failed to load TWAIN Source %s\n",
            debugstr_w((*devices.offset(i as isize)).modname)
        );
        DSM_twCC = (TWCC_OPERATIONERROR as u16);
        HeapFree(GetProcessHeap(), 0, newSource as PVOID);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    (*newSource).hmod = hmod;
    (*newSource).dsEntry = ::core::mem::transmute::<FARPROC, DSENTRYPROC>(GetProcAddress(
        hmod as HINSTANCE,
        b"DS_Entry\0".as_ptr() as LPCSTR,
    ));
    /* Assign id for the opened data source */
    let fresh_id = DSM_sourceId;
    DSM_sourceId = DSM_sourceId.wrapping_add(1);
    (*pIdentity).Id = fresh_id;
    if TWRC_SUCCESS
        != ((*newSource).dsEntry.unwrap_unchecked()(
            pOrigin,
            DG_CONTROL,
            (DAT_IDENTITY as u16),
            (MSG_OPENDS as u16),
            pIdentity as TW_MEMREF,
        ) as ::core::ffi::c_int) as u32
    {
        DSM_twCC = (TWCC_OPERATIONERROR as u16);
        HeapFree(GetProcessHeap(), 0, newSource as PVOID);
        DSM_sourceId = DSM_sourceId.wrapping_sub(1);
        return (TWRC_FAILURE) as TW_UINT16;
    }
    /* add the data source to an internal active source list */
    (*newSource).next = activeSources as *mut tagActiveDS;
    (*newSource).identity.Id = (*pIdentity).Id;
    strcpy(
        &raw mut (*newSource).identity.ProductName as *mut ::core::ffi::c_char,
        &raw mut (*pIdentity).ProductName as *mut ::core::ffi::c_char,
    );
    list_init(core::ptr::addr_of_mut!((*newSource).pending_messages));
    (*newSource).ui_window = core::ptr::null_mut();
    (*newSource).event_window = core::ptr::null_mut();
    activeSources = newSource;
    DSM_twCC = (TWCC_SUCCESS as u16);
    return (TWRC_SUCCESS) as TW_UINT16;
}

unsafe extern "system" fn userselect_dlgproc(
    mut hwnd: HWND,
    mut msg: UINT,
    mut wparam: WPARAM,
    mut lparam: LPARAM,
) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {
            let mut data: *mut userselect_data = lparam as *mut userselect_data;
            let mut i: ::core::ffi::c_int = 0;
            let mut sourcelist: HWND = ::core::ptr::null_mut::<HWND__>();
            let mut any_devices: BOOL = FALSE as BOOL;
            SetWindowLongPtrW(hwnd, DWLP_USER as ::core::ffi::c_int, data as LONG_PTR);
            sourcelist = GetDlgItem(hwnd, (IDC_LISTSOURCE as i32));
            i = 0 as ::core::ffi::c_int;
            while i < nrdevices {
                let mut id: *mut TW_IDENTITY = &raw mut (*devices.offset(i as isize)).identity;
                let mut index: LRESULT = 0;
                if !((*id).SupportedGroups & (*(*data).origin).SupportedGroups == 0 as TW_UINT32) {
                    index = SendMessageA(
                        sourcelist,
                        LB_ADDSTRING as UINT,
                        0 as WPARAM,
                        &raw mut (*id).ProductName as *mut ::core::ffi::c_char as LPARAM,
                    );
                    SendMessageW(
                        sourcelist,
                        LB_SETITEMDATA as UINT,
                        index as WPARAM,
                        i as LPARAM,
                    );
                    any_devices = TRUE as BOOL;
                }
                i += 1;
            }
            if any_devices != 0 {
                EnableWindow(
                    GetDlgItem(hwnd, (IDOK as i32)),
                    TRUE as BOOL,
                );
                /* FIXME: Select the supplied product name or default source. */
                SendMessageW(
                    sourcelist,
                    LB_SETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
            return TRUE as INT_PTR;
        }
        WM_CLOSE => {
            EndDialog(hwnd, 0 as INT_PTR);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if wparam
                == MAKEWPARAM!(IDCANCEL, BN_CLICKED) as WPARAM
            {
                EndDialog(hwnd, 0 as INT_PTR);
                return TRUE as INT_PTR;
            } else if wparam
                == MAKEWPARAM!(IDOK, BN_CLICKED) as WPARAM
                || wparam
                    == MAKEWPARAM!(IDC_LISTSOURCE, LBN_DBLCLK) as WPARAM
            {
                let mut data_0: *mut userselect_data =
                    GetWindowLongPtrW(hwnd, DWLP_USER as ::core::ffi::c_int)
                        as *mut userselect_data;
                let mut sourcelist_0: HWND = ::core::ptr::null_mut::<HWND__>();
                let mut index_0: LRESULT = 0;
                sourcelist_0 = GetDlgItem(hwnd, (IDC_LISTSOURCE as i32));
                index_0 = SendMessageW(
                    sourcelist_0,
                    LB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
                if index_0 == LB_ERR as LRESULT {
                    return TRUE as INT_PTR;
                }
                index_0 = SendMessageW(
                    sourcelist_0,
                    LB_GETITEMDATA as UINT,
                    index_0 as WPARAM,
                    0 as LPARAM,
                );
                *(*data_0).result = (*devices.offset(index_0 as isize)).identity;
                EndDialog(hwnd, 1 as INT_PTR);
                return TRUE as INT_PTR;
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_UserSelect(mut pOrigin: pTW_IDENTITY, mut pData: TW_MEMREF) -> TW_UINT16 {
    let mut param: userselect_data = userselect_data {
        origin: pOrigin,
        result: pData as pTW_IDENTITY,
    };
    let mut parent: HWND = DSM_parent;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/twain_32/dsm_ctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TWAIN_UserSelect\0" as *const u8 as *const ::core::ffi::c_char,
            439 as ::core::ffi::c_int,
            b"DG_CONTROL/DAT_IDENTITY/MSG_USERSELECT SupportedGroups=0x%lx ProductName=%s\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            (*pOrigin).SupportedGroups,
            wine_dbgstr_a(
                &raw mut (*param.result).ProductName as *mut ::core::ffi::c_char,
            ),
        );
    }
    twain_autodetect();
    if IsWindow(parent) == 0 {
        parent = ::core::ptr::null_mut::<HWND__>();
    }
    if DialogBoxParamW(
        DSM_hinstance,
        MAKEINTRESOURCEW!(DLG_USERSELECT),
        parent,
        Some(userselect_dlgproc),
        &raw mut param as LPARAM,
    ) == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/twain_32/dsm_ctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TWAIN_UserSelect\0" as *const u8 as *const ::core::ffi::c_char,
                449 as ::core::ffi::c_int,
                b"canceled\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        DSM_twCC = (TWCC_SUCCESS as u16);
        return (TWRC_CANCEL) as TW_UINT16;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/twain_32/dsm_ctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TWAIN_UserSelect\0" as *const u8 as *const ::core::ffi::c_char,
            454 as ::core::ffi::c_int,
            b"<-- %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_a(
                &raw mut (*param.result).ProductName as *mut ::core::ffi::c_char,
            ),
        );
    }
    DSM_twCC = (TWCC_SUCCESS as u16);
    return (TWRC_SUCCESS) as TW_UINT16;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_CloseDSM(mut pOrigin: pTW_IDENTITY, mut pData: TW_MEMREF) -> TW_UINT16 {
    let mut currentDS: *mut activeDS = activeSources;
    let mut nextDS: *mut activeDS = ::core::ptr::null_mut::<activeDS>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/twain_32/dsm_ctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TWAIN_CloseDSM\0" as *const u8 as *const ::core::ffi::c_char,
            464 as ::core::ffi::c_int,
            b"DG_CONTROL/DAT_PARENT/MSG_CLOSEDSM\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    if DSM_initialized != 0 {
        DSM_initialized = FALSE as TW_UINT16;
        while !currentDS.is_null() {
            nextDS = (*currentDS).next as *mut activeDS;
            (*currentDS).dsEntry.unwrap_unchecked()(
                pOrigin,
                DG_CONTROL as TW_UINT32,
                DAT_IDENTITY as TW_UINT16,
                MSG_CLOSEDS as TW_UINT16,
                pData,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                currentDS as PVOID,
            );
            currentDS = nextDS;
        }
        activeSources = ::core::ptr::null_mut::<activeDS>();
        DSM_parent = ::core::ptr::null_mut::<HWND__>();
        DSM_twCC = TWCC_SUCCESS as TW_UINT16;
        return TWRC_SUCCESS as TW_UINT16;
    } else {
        DSM_twCC = TWCC_SEQERROR as TW_UINT16;
        return TWRC_FAILURE as TW_UINT16;
    };
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_OpenDSM(pOrigin: pTW_IDENTITY, pData: TW_MEMREF) -> TW_UINT16 {
	let mut twRC: TW_UINT16 = (TWRC_SUCCESS as u16);

	if DSM_initialized == 0 {
		event_message = RegisterWindowMessageA(b"WINE TWAIN_32 EVENT\0".as_ptr() as *const i8);
		DSM_currentDevice = 0;
		DSM_initialized = (TRUE as u16);
		DSM_twCC = (TWCC_SUCCESS as u16);
		twRC = (TWRC_SUCCESS as u16);
	} else {
		/* operation invoked in invalid state */
		DSM_twCC = (TWCC_SEQERROR as u16);
		twRC = (TWRC_FAILURE as u16);
	}
	DSM_parent = pData as HWND;
	return twRC;
}
#[no_mangle]
pub unsafe extern "C" fn TWAIN_GetDSMStatus(pOrigin: pTW_IDENTITY, pData: TW_MEMREF) -> TW_UINT16 {
    let mut pSourceStatus: pTW_STATUS = pData as pTW_STATUS;

    TRACE!("DG_CONTROL/DAT_STATUS/MSG_GET\n");

    (*pSourceStatus).ConditionCode = DSM_twCC;
    DSM_twCC = (TWCC_SUCCESS as u16); /* clear the condition code */
    return (TWRC_SUCCESS) as TW_UINT16;
}
