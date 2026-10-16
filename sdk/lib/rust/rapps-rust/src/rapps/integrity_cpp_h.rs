use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_AA2A982F0618: [WCHAR; 5] = [0x0025, 0x0030, 0x0032, 0x0078, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn VerifyInteg(lpSHA1Hash: LPCWSTR, lpFileName: LPCWSTR) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    /* first off, does it exist at all? */
    let file: HANDLE = CreateFileW(lpFileName, GENERIC_READ, 0, core::ptr::null_mut(), OPEN_EXISTING, FILE_ATTRIBUTE_READONLY, core::ptr::null_mut());

    if file == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    /* let's grab the actual file size to organize the mmap'ing rounds */
    let mut size: LARGE_INTEGER = core::mem::zeroed();
    GetFileSizeEx(file, (core::ptr::addr_of_mut!(size)).cast());

    /* retrieve a handle to map the file contents to memory */
    let map: HANDLE = CreateFileMappingW(file, core::ptr::null_mut(), PAGE_READONLY, 0, 0, core::ptr::null_mut());
    if !map.is_null() {
        /* map that thing in address space */
        let file_map: *const u8 = MapViewOfFile(map, FILE_MAP_READ, 0, 0, 0) as *const u8;
        if !file_map.is_null() {
            let mut ctx: SHA_CTX = core::mem::zeroed();
            /* initialize the SHA-1 context */
            A_SHAInit(core::ptr::addr_of_mut!(ctx));

            /* feed the data to the cookie monster */
            A_SHAUpdate(core::ptr::addr_of_mut!(ctx), file_map, size.__bindgen_anon_1.LowPart);

            /* cool, we don't need this anymore */
            UnmapViewOfFile(file_map as *const core::ffi::c_void);

            /* we're done, compute the final hash */
            let mut sha: [ULONG; 5] = [0; 5];
            A_SHAFinal(core::ptr::addr_of_mut!(ctx), sha.as_mut_ptr());

            let mut buf: [WCHAR; (core::mem::size_of::<[ULONG; 5]>() * 2) + 1] = [0; (core::mem::size_of::<[ULONG; 5]>() * 2) + 1];
            let mut i: UINT = 0;
            while i < core::mem::size_of::<[ULONG; 5]>() as UINT {
                _swprintf(buf.as_mut_ptr().add((2 * i) as usize), CPP_COM_WSTR_AA2A982F0618.as_ptr(), (*(sha.as_ptr() as *const u8).add(i as usize)) as i32);
                i += 1;
            }
            /* does the resulting SHA1 match with the provided one? */
            if _wcsicmp(buf.as_ptr(), lpSHA1Hash) == 0 {
                ret = (TRUE as i32);
            }
        }
        CloseHandle(map);
    }
    CloseHandle(file);
    return ret;
}
