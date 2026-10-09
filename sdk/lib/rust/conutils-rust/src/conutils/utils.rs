use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "system" {
    fn FindResourceExW(hModule: HINSTANCE, lpType: *const u16, lpName: *const u16, wLanguage: u16) -> HRSRC;
    fn LoadResource(hModule: HINSTANCE, hResInfo: HRSRC) -> HGLOBAL;
    fn LockResource(hResData: HGLOBAL) -> *mut u16;
    fn FormatMessageW(dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, lpBuffer: *mut u16, nSize: u32, Arguments: *const va_list) -> u32;
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const i8) -> *mut c_void;
    fn GetFileType(hFile: *mut c_void) -> u32;
    fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut u32) -> i32;
}

// InterlockedCompareExchangePointer is a compiler intrinsic on x86 (lowered to an
// inline cmpxchg, no symbol), so the original extern decl resolved there but had no
// provider on arm64 → undefined at link. A pointer-width atomic CAS is the portable
// equivalent and keeps the call site below unchanged.
#[inline]
unsafe fn InterlockedCompareExchangePointer(
    Destination: *mut *mut c_void,
    Exchange: *mut c_void,
    Comperand: *mut c_void,
) -> *mut c_void {
    use core::sync::atomic::{AtomicPtr, Ordering};
    let atomic = &*(Destination as *const AtomicPtr<c_void>);
    match atomic.compare_exchange(Comperand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(prev) | Err(prev) => prev,
    }
}

const RT_STRING: u32 = 6;

const FILE_TYPE_CHAR: u32 = 0x0002;
const FILE_TYPE_REMOTE: u32 = 0x8000;

const LANG_NEUTRAL: u16 = 0x00;
const SUBLANG_NEUTRAL: u16 = 0x00;
const MAKELANGID_LN_SN: u16 = ((SUBLANG_NEUTRAL as u16) << 10) | (LANG_NEUTRAL as u16);

// Thread-safe function pointer for SetThreadUILanguage (from kernel32)
static mut pfnSetThreadUILanguage: *mut c_void = core::ptr::null_mut();

unsafe fn LOWORD(dw: u32) -> u16 {
    dw as u16
}

unsafe fn MAKEINTRESOURCEW(id: u32) -> *const u16 {
    id as *const u16
}

unsafe fn min_u32(a: u32, b: u32) -> u32 {
    if a < b { a } else { b }
}

#[no_mangle]
pub unsafe extern "system" fn K32LoadStringExW(
    hInstance: HINSTANCE,
    uID: u32,
    LanguageId: u16,
    lpBuffer: *mut u16,
    nBufferMax: i32,
) -> i32 {
    let mut uID = uID;
    let hrsrc: HRSRC;
    let hmem: HGLOBAL;
    let mut p: *mut u16;
    let i: u32;

    if lpBuffer.is_null() {
        return 0;
    }

    hrsrc = FindResourceExW(
        hInstance,
        RT_STRING as *const u16,
        MAKEINTRESOURCEW(((LOWORD(uID) >> 4) + 1) as u32),
        LanguageId,
    );
    if hrsrc.is_null() {
        return 0;
    }

    hmem = LoadResource(hInstance, hrsrc);
    if hmem.is_null() {
        return 0;
    }

    p = LockResource(hmem);

    uID &= 0x000F;
    for _ in 0..uID {
        p = p.add((*p as usize) + 1);
    }

    if nBufferMax == 0 {
        *(lpBuffer as *mut *mut u16) = p.add(1);
        return *p as i32;
    }

    i = min_u32((nBufferMax as u32).wrapping_sub(1), *p as u32);
    if i > 0 {
        core::ptr::copy_nonoverlapping(p.add(1) as *const u16, lpBuffer, i as usize);
        *lpBuffer.add(i as usize) = 0u16;
    } else {
        if nBufferMax > 1 {
            core::ptr::write_volatile(lpBuffer, 0u16);
            return 0;
        }
    }

    i as i32
}

#[no_mangle]
pub unsafe extern "system" fn K32LoadStringW(
    hInstance: HINSTANCE,
    uID: u32,
    lpBuffer: *mut u16,
    nBufferMax: i32,
) -> i32 {
    K32LoadStringExW(hInstance, uID, MAKELANGID_LN_SN, lpBuffer, nBufferMax)
}

#[no_mangle]
pub unsafe extern "system" fn FormatMessageSafeW(
    dwFlags: u32,
    lpSource: *const c_void,
    dwMessageId: u32,
    dwLanguageId: u32,
    lpBuffer: *mut u16,
    nSize: u32,
    Arguments: *const va_list,
) -> u32 {
    let dwLength: u32;
    let dwFlags_mut = dwFlags;

    // SEH_TRY — direct call (DUMMY_PSEH: SEH handlers are dead code)
    // inner SEH_TRY
    dwLength = FormatMessageW(
        dwFlags_mut,
        lpSource,
        dwMessageId,
        dwLanguageId,
        lpBuffer,
        nSize,
        Arguments,
    );
    // inner SEH_EXCEPT handler is dead in DUMMY_PSEH — no need for catch block

    dwLength
}

#[no_mangle]
pub unsafe extern "C" fn ConSetThreadUILanguage(lang_id: LANGID) -> LANGID {
    let initial = core::ptr::read_volatile(&raw const pfnSetThreadUILanguage);
    if initial.is_null() {
        let kernel32_name: [u16; 13] = [
            b'k' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'e' as u16, b'l' as u16,
            b'3' as u16, b'2' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16,
            0u16,
        ];
        let h_module: *mut c_void = GetModuleHandleW(kernel32_name.as_ptr());
        let proc_name = b"SetThreadUILanguage\0".as_ptr() as *const i8;
        let p_func = GetProcAddress(h_module, proc_name);
        if p_func.is_null() {
            return 0;
        }
        InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!(pfnSetThreadUILanguage),
            p_func,
            core::ptr::null_mut(),
        );
    }
    let fn_ptr = core::ptr::read_volatile(&raw const pfnSetThreadUILanguage);
    if !fn_ptr.is_null() {
        let func: unsafe extern "system" fn(LANGID) -> LANGID = core::mem::transmute(fn_ptr);
        func(lang_id)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn IsTTYHandle(hHandle: *mut c_void) -> i32 {
    ((GetFileType(hHandle) & !FILE_TYPE_REMOTE) == FILE_TYPE_CHAR) as i32
}

#[no_mangle]
pub unsafe extern "C" fn IsConsoleHandle(hHandle: HANDLE) -> i32 {
    let mut dwMode: u32 = 0;

    if (GetFileType(hHandle) & !FILE_TYPE_REMOTE) != FILE_TYPE_CHAR {
        return 0;
    }

    GetConsoleMode(hHandle, &mut dwMode)
}
