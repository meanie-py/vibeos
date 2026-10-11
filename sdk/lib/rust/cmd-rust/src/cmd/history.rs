use crate::cmd::console::ConInString;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut size: INT = 0;

static mut max_size: INT = 100 as INT;

static mut Top: LPHIST_ENTRY = ::core::ptr::null::<tagHISTORY>() as *mut tagHISTORY;

static mut Bottom: LPHIST_ENTRY = ::core::ptr::null::<tagHISTORY>() as *mut tagHISTORY;

static mut curr_ptr: LPHIST_ENTRY = ::core::ptr::null::<tagHISTORY>() as *mut tagHISTORY;

#[repr(C)]
pub struct tagHISTORY {
    pub prev: *mut tagHISTORY,
    pub next: *mut tagHISTORY,
    pub string: LPTSTR,
}

pub type HIST_ENTRY = tagHISTORY;
pub type LPHIST_ENTRY = *mut tagHISTORY;
#[no_mangle]
pub unsafe extern "C" fn CommandHistory(mut param: LPTSTR) -> INT {
    let mut tmp: LPTSTR;
    let mut tmp_int: INT;
    let mut h_tmp: LPHIST_ENTRY;
    let mut szBuffer: [TCHAR; 2048] = [0; 2048];

    tmp = _tcschr(param, ('/' as u16));

    if !tmp.is_null() {
        param = tmp;
        match _totupper((*param.offset(1 as isize) as INT) as u16) {
            70 => {
                CleanHistory();
                InitHistory();
            }
            82 => loop {
                ConInString(
                    core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
                    (core::mem::size_of::<[TCHAR; 2048]>()
                        / core::mem::size_of::<TCHAR>()) as DWORD,
                );
                if *(core::ptr::addr_of_mut!(szBuffer) as LPTSTR) != 0 {
                    History(
                        0 as INT,
                        core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
                    );
                } else {
                    break;
                }
            },
            65 => {
                History(
                    0 as INT,
                    param.offset(2 as isize),
                );
            }
            83 => {
                tmp_int = _ttoi(param.offset(2 as isize)) as INT;
                if tmp_int != 0 {
                    set_size(tmp_int);
                }
            }
            _ => return 1 as INT,
        }
    } else {
        h_tmp = (*Top).prev;
        while h_tmp != Bottom {
            let fmt: [u16; 4] = ['%' as u16, 's' as u16, '\n' as u16, 0];
            ConOutPrintf!(fmt.as_ptr() as LPCTSTR, (*h_tmp).string);
            h_tmp = (*h_tmp).prev;
        }
    }
    return 0 as INT;
}
#[no_mangle]
pub unsafe extern "C" fn set_size(mut new_size: INT) {
    if !(!Top.is_null() && !Bottom.is_null()) {
        RtlAssert(
            b"Top && Bottom\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"base/shell/cmd/history.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            123 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    while new_size < size {
        del((*Top).prev as LPHIST_ENTRY);
    }
    max_size = new_size;
}
#[no_mangle]
pub unsafe extern "C" fn InitHistory() {
    size = 0 as INT;

    Top = cmd_alloc!(core::mem::size_of::<HIST_ENTRY>() as SIZE_T) as LPHIST_ENTRY;
    if Top.is_null() {
        CmdTrace(
            2 as INT, // __WINE_DBCL_WARN
            b"base/shell/cmd/history.c\0"
                as *const u8 as LPCSTR,
            139 as INT,
            b"InitHistory\0" as *const u8 as LPCSTR,
            b"Cannot allocate memory for Top!\n\0" as *const u8 as LPCSTR,
        );
        return;
    }
    Bottom = cmd_alloc!(core::mem::size_of::<HIST_ENTRY>() as SIZE_T) as LPHIST_ENTRY;
    if Bottom.is_null() {
        CmdTrace(
            2 as INT, // __WINE_DBCL_WARN
            b"base/shell/cmd/history.c\0"
                as *const u8 as LPCSTR,
            145 as INT,
            b"InitHistory\0" as *const u8 as LPCSTR,
            b"Cannot allocate memory for Bottom!\n\0" as *const u8 as LPCSTR,
        );
        cmd_free!(Top);
        Top = core::ptr::null_mut();
        return;
    }

    (*Top).prev = Bottom;
    (*Top).next = core::ptr::null_mut();
    (*Top).string = core::ptr::null_mut();

    (*Bottom).prev = core::ptr::null_mut();
    (*Bottom).next = Top;
    (*Bottom).string = core::ptr::null_mut();

    curr_ptr = Bottom;
}
#[no_mangle]
pub unsafe extern "C" fn CleanHistory() {
    while (*Bottom).next != Top {
        del((*Bottom).next);
    }

    cmd_free!(Top as *mut core::ffi::c_void);
    cmd_free!(Bottom as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn History_del_current_entry(mut str: LPTSTR) {
    let mut tmp: LPHIST_ENTRY = core::ptr::null_mut::<tagHISTORY>();

    if !(!Top.is_null() && !Bottom.is_null()) {
        RtlAssert(
            b"Top && Bottom\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"base/shell/cmd/history.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            179 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    if size == 0 {
        return;
    }

    if curr_ptr == Bottom {
        curr_ptr = (*Bottom).next as LPHIST_ENTRY;
    }

    if curr_ptr == Top {
        curr_ptr = (*Top).prev as LPHIST_ENTRY;
    }

    tmp = curr_ptr;
    curr_ptr = (*curr_ptr).prev as LPHIST_ENTRY;
    del(tmp);
    History(-1, str);
}

unsafe extern "C" fn del(item: LPHIST_ENTRY) {
    if !(!Top.is_null() && !Bottom.is_null()) {
        RtlAssert(
            b"Top && Bottom\0".as_ptr() as PVOID,
            b"base/shell/cmd/history.c\0".as_ptr() as PVOID,
            201 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    if item.is_null() || item == Top || item == Bottom {
        CmdTrace(
            __WINE_DBCL_TRACE as INT,
            b"base/shell/cmd/history.c\0".as_ptr() as LPCSTR,
            207 as INT,
            b"del\0".as_ptr() as LPCSTR,
            b"del in base/shell/cmd/history.c: returning\nitem is 0x%08x (Bottom is0x%08x)\n\0".as_ptr() as LPCSTR,
            item,
            Bottom,
        );
        return;
    }
    if !(*item).string.is_null() {
        cmd_free!((*item).string as *mut core::ffi::c_void);
    }
    (*(*item).next).prev = (*item).prev;
    (*(*item).prev).next = (*item).next;
    cmd_free!(item as *mut core::ffi::c_void);
    size -= 1;
}

unsafe extern "C" fn add_at_bottom(mut string: LPTSTR) {
    let mut tmp: LPHIST_ENTRY = ::core::ptr::null_mut::<tagHISTORY>();
    /*delete first entry if maximum number of entries is reached*/
    while size >= max_size {
        del((*Top).prev);
    }
    while _istspace!(*string) != 0 {
        string = string.offset(1);
    }
    if *string == ('\0' as u16) {
        return;
    }
    /*if new entry is the same than the last do not add it*/
    if size != 0 {
        if _tcscmp(string, (*(*Bottom).next).string) == 0 {
            return;
        }
    }
    /*create new empty Bottom*/
    tmp = cmd_alloc!(core::mem::size_of::<HIST_ENTRY>() as usize) as LPHIST_ENTRY;
    if tmp.is_null() {
        return;
    }
    /*fill old bottom with string, it will become new Bottom->next*/
    (*Bottom).string = cmd_alloc!(
        (_tcslen(string) + 1) * core::mem::size_of::<TCHAR>(),
    ) as LPTSTR;
    if (*Bottom).string.is_null() {
        cmd_free!(tmp);
        return;
    }
    _tcscpy((*Bottom).string, string);

    (*tmp).next = Bottom;
    (*tmp).prev = ::core::ptr::null_mut::<tagHISTORY>();
    (*tmp).string = ::core::ptr::null_mut::<TCHAR>();

    (*Bottom).prev = tmp;

    /*save the new Bottom value*/
    Bottom = tmp;

    /*set new size*/
    size += 1;
}
#[no_mangle]
pub unsafe extern "C" fn History_move_to_bottom() {
    if !(!Top.is_null() && !Bottom.is_null()) {
        RtlAssert(
            b"Top && Bottom\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"base/shell/cmd/history.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            282 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    curr_ptr = Bottom;
}
#[no_mangle]
pub unsafe extern "C" fn PeekHistory(mut dir: INT) -> LPCTSTR {
    let mut entry: LPHIST_ENTRY = curr_ptr;
    if !(!Top.is_null() && !Bottom.is_null()) {
        RtlAssert(
            b"Top && Bottom\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"base/shell/cmd/history.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            291 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if dir == 0 {
        return ::core::ptr::null::<WCHAR>();
    }
    if dir < 0 {
        if (*entry).next == Top || entry == Top {
            entry = Bottom;
        }
        entry = (*entry).next as LPHIST_ENTRY;
    } else {
        if (*entry).prev == Bottom || entry == Bottom {
            entry = Top;
        }
        entry = (*entry).prev as LPHIST_ENTRY;
    }
    return (*entry).string as LPCTSTR;
}
#[no_mangle]
pub unsafe extern "C" fn History(mut dir: INT, mut commandline: LPTSTR) {
    if dir == 0 {
        add_at_bottom(commandline);
        curr_ptr = Bottom;
        return;
    }
    if size == 0 {
        *commandline.offset(0 as isize) = ('\0' as u16);
        return;
    }
    if dir < 0 {
        if (*curr_ptr).next == Top || curr_ptr == Top {
            curr_ptr = Top;
            *commandline.offset(0 as isize) = ('\0' as u16);
            return;
        }
        curr_ptr = (*curr_ptr).next;
        if !(*curr_ptr).string.is_null() {
            _tcscpy(commandline, (*curr_ptr).string);
        }
    }
    if dir > 0 {
        if (*curr_ptr).prev == Bottom || curr_ptr == Bottom {
            curr_ptr = Bottom;
            *commandline.offset(0 as isize) = ('\0' as u16);
            return;
        }
        curr_ptr = (*curr_ptr).prev;
        if !(*curr_ptr).string.is_null() {
            _tcscpy(commandline, (*curr_ptr).string);
        }
    }
}
