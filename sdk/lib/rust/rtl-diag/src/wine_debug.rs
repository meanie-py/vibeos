use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{LPCSTR, PCCH, RtlAllocateHeap, RtlFreeHeap, va_list};
use crate::shared::NtCurrentTeb;
use core::ffi::c_void;

extern "C" {
    fn vsnprintf(buffer: *mut core::ffi::c_char, count: usize, format: *const core::ffi::c_char, valist: va_list) -> core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct s_alloactions_entry {
    pub thread: HANDLE,
    pub allocations: *mut core::ffi::c_void,
}

pub static mut s_alloactions: [s_alloactions_entry; 32] = [s_alloactions_entry {
    thread: core::ptr::null_mut(),
    allocations: core::ptr::null_mut(),
}; 32];

unsafe fn find_thread_slot() -> i32 {
    let thread = (*NtCurrentTeb()).ClientId.UniqueThread;
    let mut i: i32 = 0;
    while (i as usize) < s_alloactions.len() {
        if s_alloactions[i as usize].thread == thread {
            return i;
        }
        i += 1;
    }
    -1
}

unsafe fn get_thread_slot() -> i32 {
    let slot = find_thread_slot();
    if slot != -1 {
        return slot;
    }

    let thread = (*NtCurrentTeb()).ClientId.UniqueThread;
    let count = (core::mem::size_of_val(&s_alloactions) / core::mem::size_of_val(&s_alloactions[0])) as i32;
    let mut i = 0;
    while i < count {
        if s_alloactions[i as usize].thread.is_null() {
            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(s_alloactions[i as usize].thread),
                thread,
                core::ptr::null_mut(),
            )
            .is_null()
            {
                return i;
            }
        }
        i += 1;
    }

    -1
}

unsafe fn alloc_buffer(size: usize) -> *mut core::ffi::c_char {
    let slot = get_thread_slot();
    if slot == -1 {
        return core::ptr::null_mut();
    }

    let buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        size + core::mem::size_of::<*mut core::ffi::c_void>(),
    ) as *mut *mut core::ffi::c_void;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    *buffer = s_alloactions[slot as usize].allocations;
    s_alloactions[slot as usize].allocations = buffer as *mut core::ffi::c_void;

    buffer.add(1) as *mut core::ffi::c_char
}

unsafe fn free_buffers() {
    let slot = find_thread_slot();
    if slot != -1 {
        return;
    }

    let mut buffer = s_alloactions[slot as usize].allocations;
    while !buffer.is_null() {
        let next = *(buffer as *mut *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, buffer);
        buffer = next;
    }

    s_alloactions[slot as usize].allocations = core::ptr::null_mut();
    s_alloactions[slot as usize].thread = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn wine_dbg_vsprintf(format: *const core::ffi::c_char, valist: va_list) -> *const core::ffi::c_char {

    let mut len: core::ffi::c_int;

    len = vsnprintf(core::ptr::null_mut(), 0, format, valist);
    let buffer = alloc_buffer((len + 1) as usize);
    if buffer.is_null() {
        return b"<allocation failed>\0".as_ptr() as *const core::ffi::c_char;
    }
    len = vsnprintf(buffer, len as usize, format, valist);
    *buffer.offset(len as isize) = 0;
    buffer
}
#[no_mangle]
pub unsafe extern "C" fn wine_dbg_sprintf(format: *const core::ffi::c_char, mut args: ...) -> *const core::ffi::c_char {
    let mut valist: va_list = args.as_va_list();
    let ret = wine_dbg_vsprintf(format, valist);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn wine_dbgstr_wn(str: *const WCHAR, mut n: core::ffi::c_int) -> *const core::ffi::c_char {
    if (str as ULONG_PTR) >> 16 == 0 {
        if str.is_null() {
            return b"(null)\0".as_ptr() as *const core::ffi::c_char;
        }
        return wine_dbg_sprintf(b"#%04x\0".as_ptr() as *const core::ffi::c_char, (str as usize as u32) & 0xffff);
    }
    if n == -1 {
        n = wcslen(str) as core::ffi::c_int;
    }
    if n < 0 {
        n = 0;
    }

    wine_dbg_sprintf(b"%.*S\0".as_ptr() as *const core::ffi::c_char, n, str)
}
#[no_mangle]
pub unsafe extern "C" fn debugstr_us(us: *const _UNICODE_STRING) -> LPCSTR {
    if us.is_null() {
        return b"<null>\0".as_ptr() as LPCSTR;
    }
    debugstr_wn((*us).Buffer, ((*us).Length as usize / core::mem::size_of::<u16>()) as _)
}

unsafe fn default_dbg_vprintf(format: *const core::ffi::c_char, args: va_list) -> core::ffi::c_int {
    vDbgPrintExWithPrefix(
        b"\0".as_ptr() as PCCH,
        -1i32 as ULONG,
        0,
        format as PCCH,
        args,
    ) as core::ffi::c_int
}
#[no_mangle]
pub unsafe extern "C" fn wine_dbg_printf(format: *const core::ffi::c_char, mut args: ...) -> core::ffi::c_int {
    let ret: core::ffi::c_int;
    let mut valist: va_list = args.as_va_list();

    ret = default_dbg_vprintf(format, valist);
    free_buffers();
    ret
}

unsafe fn winefmt_default_dbg_vlog(
    cls: __wine_debug_class,
    channel: *mut __wine_debug_channel,
    file: *const core::ffi::c_char,
    func: *const core::ffi::c_char,
    line: core::ffi::c_int,
    format: *const core::ffi::c_char,
    args: va_list,
) -> core::ffi::c_int {
    let mut ret: core::ffi::c_int = 0;

    ret += wine_dbg_printf(
        b"%04x:\0".as_ptr() as *const core::ffi::c_char,
        HandleToULong((*NtCurrentTeb()).ClientId.UniqueProcess),
    );
    ret += wine_dbg_printf(
        b"%04x:\0".as_ptr() as *const core::ffi::c_char,
        HandleToULong((*NtCurrentTeb()).ClientId.UniqueThread),
    );

    if !format.is_null() {
        ret += default_dbg_vprintf(format, args);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn ros_dbg_log(
    cls: __wine_debug_class,
    channel: *mut __wine_debug_channel,
    file: *const core::ffi::c_char,
    func: *const core::ffi::c_char,
    line: core::ffi::c_int,
    format: *const core::ffi::c_char,
    mut args: ...
) -> core::ffi::c_int {
    let ret: core::ffi::c_int;

    if (__wine_dbg_get_channel_flags(channel) & (1 << cls)) == 0 {
        return -1;
    }

    let mut valist = args.clone();
    ret = winefmt_default_dbg_vlog(cls, channel, file, func, line, format, valist.as_va_list());
    ret
}
