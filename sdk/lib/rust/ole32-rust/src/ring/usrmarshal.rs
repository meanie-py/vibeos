use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn sprintf(s: *mut i8, format: *const i8, ...) -> i32;
    fn wine_dbg_sprintf(format: *const i8, ...) -> *const i8;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn GetBitmapBits(hbit: HBITMAP, cb: i32, lpvBits: *mut core::ffi::c_void) -> i32;
    fn GetObjectW(h: *mut core::ffi::c_void, c: i32, pv: *mut core::ffi::c_void) -> i32;
    fn CreateBitmapIndirect(pbm: *const BITMAP) -> HBITMAP;
    fn RaiseException(dwExceptionCode: DWORD, dwExceptionFlags: DWORD, nNumberOfArguments: DWORD, lpArguments: *const ULONG_PTR);
    fn RpcRaiseException(exception: RPC_STATUS);
    fn CoTaskMemAlloc(cb: usize) -> LPVOID;
}
extern "C" {
    fn GetClipboardFormatNameW(format: CLIPFORMAT, lpszFormatName: *mut WCHAR, cchMaxCount: i32) -> i32;
    fn RegisterClipboardFormatW(lpszFormat: LPCWSTR) -> UINT;
    fn HandleToULong(h: *mut core::ffi::c_void) -> ULONG;
    fn DeleteObject(ho: HGLOBAL) -> i32;
    fn GetMetaFileBitsEx(hmf: HMETAFILE, size: UINT, buffer: *mut u8) -> UINT;
    fn DeleteMetaFile(hmf: HMETAFILE) -> i32;
    fn ALIGN_POINTER(ptr: *mut u8, align: usize) -> *mut u8;
    fn GetEnhMetaFileBits(hemf: HENHMETAFILE, buf_size: u32, buffer: *mut u8) -> u32;
    fn SetEnhMetaFileBits(nSize: UINT, pb: *const BYTE) -> HENHMETAFILE;
    fn DeleteEnhMetaFile(hemf: HENHMETAFILE) -> i32;
    fn CoGetMarshalSizeMax( pulSize: *mut DWORD, riid: REFIID, pUnk: *mut IUnknown, dwDestContext: DWORD, pvDestContext: *mut core::ffi::c_void, mshlflags: DWORD, ) -> HRESULT;
    fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: i32, ppstm: *mut *mut IStream) -> HRESULT;
    fn CoMarshalInterface(pStm: *mut IStream, riid: REFIID, pUnk: *mut IUnknown, dwDestContext: DWORD, pvDestContext: LPVOID, mshlflags: DWORD) -> HRESULT;
    fn CoUnmarshalInterface(pStm: *mut IStream, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn ReleaseStgMedium(pmedium: *const STGMEDIUM);
}

// ---- file-scope preamble (translated) ----
pub const USER_MARSHAL_PTR_PREFIX: DWORD =
    ('U' as DWORD) | (('s' as DWORD) << 8) |
    (('e' as DWORD) << 16) | (('r' as DWORD) << 24);

#[repr(C)]
pub struct SNB_wire {
    pub charcnt: ULONG,
    pub strcnt: ULONG,
    pub datalen: ULONG,
    pub data: [WCHAR; 1],
}

unsafe fn debugstr_user_flags(pFlags: *mut ULONG) -> *const i8 {

    let mut buf: [i8; 12] = [0; 12];
    let loword: *const i8;

    match LOWORD(*pFlags) as u32 {
        0 => {
            loword = b"MSHCTX_LOCAL\0".as_ptr() as *const i8;
        }
        1 => {
            loword = b"MSHCTX_NOSHAREDMEM\0".as_ptr() as *const i8;
        }
        2 => {
            loword = b"MSHCTX_DIFFERENTMACHINE\0".as_ptr() as *const i8;
        }
        3 => {
            loword = b"MSHCTX_INPROC\0".as_ptr() as *const i8;
        }
        _ => {
            sprintf(buf.as_mut_ptr(), b"%d\0".as_ptr() as *const i8, LOWORD(*pFlags));
            loword = buf.as_ptr();
        }
    }

    if HIWORD(*pFlags) == 0x0010 {
        wine_dbg_sprintf(b"MAKELONG(%s, NDR_LOCAL_DATA_REPRESENTATION)\0".as_ptr() as *const i8, loword)
    } else {
        wine_dbg_sprintf(b"MAKELONG(%s, 0x%04x)\0".as_ptr() as *const i8, loword, HIWORD(*pFlags))
    }
}
#[no_mangle]
pub unsafe extern "C" fn CLIPFORMAT_UserSize(
    pFlags: *mut ULONG,
    size: ULONG,
    pCF: *mut CLIPFORMAT,
) -> ULONG {
    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        size,
        pCF,
    );

    let mut size = size;
    ALIGN_LENGTH(&mut size as *mut ULONG, 3);

    size += 8;

    if (*pCF >= 0xc000) && (LOWORD(*pFlags) == MSHCTX_DIFFERENTMACHINE) {
        let mut format: [WCHAR; 255] = core::mem::zeroed();
        size += 3 * core::mem::size_of::<UINT>() as ULONG;
        let ret = GetClipboardFormatNameW(
            *pCF,
            format.as_mut_ptr(),
            ARRAY_SIZE(format.as_ptr()) - 1,
        );
        if ret == 0 {
            RaiseException(DV_E_CLIPFORMAT, 0, 0, core::ptr::null_mut());
        }
        size += (ret + 1) as ULONG * core::mem::size_of::<WCHAR>() as ULONG;
    }
    size
}
#[no_mangle]
pub unsafe extern "C" fn CLIPFORMAT_UserMarshal(pFlags: *mut ULONG, pBuffer: *mut u8, pCF: *mut CLIPFORMAT) -> *mut u8 {

    const WDT_REMOTE_CALL: DWORD = 0x0001;
    const WDT_INPROC_CALL: DWORD = 0x0002;
    const DV_E_CLIPFORMAT: DWORD = 0x8004006A;

    TRACE!(b"(%s, %p, &0x%04x\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, *pCF);

    let mut pBuffer = pBuffer;
    ALIGN_POINTER(pBuffer, 3);

    if (*pCF >= 0xc000) && (LOWORD(*pFlags) == MSHCTX_DIFFERENTMACHINE) {
        let mut format: [WCHAR; 255] = core::mem::zeroed();
        let mut len: UINT;

        *(pBuffer as *mut DWORD) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(4);
        *(pBuffer as *mut DWORD) = *pCF as DWORD;
        pBuffer = pBuffer.add(4);

        len = GetClipboardFormatNameW(*pCF, format.as_mut_ptr(), (format.len() - 1) as i32) as UINT;
        if len == 0 {
            RaiseException(DV_E_CLIPFORMAT, 0, 0, core::ptr::null());
        }
        len += 1;
        *(pBuffer as *mut UINT) = len;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        *(pBuffer as *mut UINT) = 0;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        *(pBuffer as *mut UINT) = len;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        TRACE!(b"marshaling format name %s\n\0".as_ptr() as *const i8, debugstr_w(format.as_ptr()));
        core::ptr::copy_nonoverlapping(format.as_ptr() as *const u8, pBuffer, (len as usize) * core::mem::size_of::<WCHAR>());
        pBuffer = pBuffer.add((len as usize) * core::mem::size_of::<WCHAR>());
    } else {
        *(pBuffer as *mut DWORD) = WDT_INPROC_CALL;
        pBuffer = pBuffer.add(4);
        *(pBuffer as *mut DWORD) = *pCF as DWORD;
        pBuffer = pBuffer.add(4);
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn CLIPFORMAT_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, pCF: *mut CLIPFORMAT) -> *mut u8 {


    const WDT_INPROC_CALL: LONG = 1;
    const WDT_REMOTE_CALL: LONG = 2;
    const DV_E_CLIPFORMAT: DWORD = 0x8004006A;

    TRACE!(b"(%s, %p, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, pCF);

    pBuffer = ((pBuffer as usize + 3) & !3) as *mut u8;

    let fContext = *(pBuffer as *const DWORD) as LONG;
    pBuffer = pBuffer.add(4);

    if fContext == WDT_INPROC_CALL {
        *pCF = *(pBuffer as *const CLIPFORMAT);
        pBuffer = pBuffer.add(4);
    } else if fContext == WDT_REMOTE_CALL {
        let mut cf: CLIPFORMAT;
        let len: UINT;

        if *(pBuffer as *const DWORD) == 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(4);

        len = *(pBuffer as *const UINT);
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *(pBuffer as *const UINT) != 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *(pBuffer as *const UINT) != len {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *((pBuffer as *const WCHAR).add(len as usize - 1)) != 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        TRACE!(b"unmarshaling clip format %s\n\0".as_ptr() as *const i8, debugstr_w(pBuffer as LPCWSTR));
        cf = RegisterClipboardFormatW(pBuffer as LPCWSTR);
        pBuffer = pBuffer.add((len as usize) * core::mem::size_of::<WCHAR>());
        if cf == 0 {
            RaiseException(DV_E_CLIPFORMAT, 0, 0, core::ptr::null());
        }
        *pCF = cf;
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn CLIPFORMAT_UserFree(_pFlags: *mut ULONG, _pCF: *mut CLIPFORMAT) {
}

unsafe fn handle_UserSize(pFlags: *mut ULONG, StartingSize: ULONG, handle: *mut HANDLE) -> ULONG {
    if LOWORD(*pFlags) == MSHCTX_DIFFERENTMACHINE {
        ERR!(b"can't remote a local handle\n\0".as_ptr() as *const i8);
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null_mut());
        return StartingSize;
    }

    let mut size = StartingSize;
    ALIGN_LENGTH(&mut size as *mut ULONG, 3);
    size + core::mem::size_of::<RemotableHandle>() as ULONG
}

unsafe fn handle_UserMarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, handle: *mut HANDLE) -> *mut u8 {

    if LOWORD(*pFlags) == MSHCTX_DIFFERENTMACHINE {
        ERR!("can't remote a local handle\n");
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
        return pBuffer;
    }

    ALIGN_POINTER!(pBuffer, 3);
    let remhandle = pBuffer as *mut RemotableHandle;
    (*remhandle).fContext = WDT_INPROC_CALL;
    (*remhandle).u.hInproc = *handle as LONG_PTR;
    pBuffer.add(core::mem::size_of::<RemotableHandle>())
}

unsafe fn handle_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, handle: *mut HANDLE) -> *mut u8 {
    let mut remhandle: *mut RemotableHandle;

    ALIGN_POINTER!(pBuffer, 3);
    remhandle = pBuffer as *mut RemotableHandle;
    if (*remhandle).fContext != WDT_INPROC_CALL {
        RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null_mut());
    }
    *handle = (*remhandle).u.hInproc as isize as HANDLE;
    return pBuffer.add(core::mem::size_of::<RemotableHandle>());
}

unsafe fn handle_UserFree(pFlags: *mut ULONG, handle: *mut HANDLE) {
    /* nothing to do */
}

#[no_mangle]
pub unsafe extern "system" fn HGLOBAL_UserSize(pFlags: *mut ULONG, StartingSize: ULONG, phGlobal: *mut HGLOBAL) -> ULONG {
    let mut size = StartingSize;

    TRACE!(b"(%s, %d, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), StartingSize, phGlobal);

    ALIGN_LENGTH(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;

    if LOWORD(*pFlags) == MSHCTX_INPROC {
        size += core::mem::size_of::<HGLOBAL>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;
        if !(*phGlobal).is_null() {
            size += 3 * core::mem::size_of::<ULONG>() as ULONG;
            let ret = GlobalSize(*phGlobal);
            size += ret as ULONG;
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn HGLOBAL_UserMarshal(pFlags: *mut ULONG, pBuffer: *mut u8, phGlobal: *mut HGLOBAL) -> *mut u8 {

    const MSHCTX_INPROC: u32 = 3;
    const WDT_INPROC_CALL: u32 = 1;
    const WDT_INPROC64_CALL: u32 = 4;
    const WDT_REMOTE_CALL: u32 = 2;

    TRACE!(b"(%s, %p, &%p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer as *mut core::ffi::c_void, *phGlobal);

    let mut pBuffer = pBuffer;
    pBuffer = ((pBuffer as usize + 3) & !3) as *mut u8;

    if (*pFlags & 0xFFFF) == MSHCTX_INPROC {
        if core::mem::size_of::<HGLOBAL>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HGLOBAL) = *phGlobal;
        pBuffer = pBuffer.add(core::mem::size_of::<HGLOBAL>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = HandleToULong(*phGlobal);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        if !(*phGlobal).is_null() {
            let size = GlobalSize(*phGlobal);
            *(pBuffer as *mut ULONG) = size as ULONG;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = HandleToULong(*phGlobal);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = size as ULONG;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            let memory = GlobalLock(*phGlobal) as *const u8;
            memcpy(pBuffer as *mut core::ffi::c_void, memory as *const core::ffi::c_void, size);
            pBuffer = pBuffer.add(size);
            GlobalUnlock(*phGlobal);
        }
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HGLOBAL_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phGlobal: *mut HGLOBAL) -> *mut u8 {

    const WDT_INPROC_CALL: ULONG = 1;
    const WDT_INPROC64_CALL: ULONG = 4;
    const WDT_REMOTE_CALL: ULONG = 2;
    const RPC_S_INVALID_TAG: DWORD = 1751;
    const RPC_X_BAD_STUB_DATA: DWORD = 1783;

    TRACE!(b"(%s, %p, &%p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, *phGlobal);

    ALIGN_POINTER!(pBuffer, 3);

    let fContext = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL) && (core::mem::size_of::<HGLOBAL>() < 8)) ||
       ((fContext == WDT_INPROC64_CALL) && (core::mem::size_of::<HGLOBAL>() == 8)) {
        *phGlobal = *(pBuffer as *const HGLOBAL);
        pBuffer = pBuffer.add(core::mem::size_of::<HGLOBAL>());
    } else if fContext == WDT_REMOTE_CALL {
        let handle = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            let size = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            if *(pBuffer as *const ULONG) != handle {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            if *(pBuffer as *const ULONG) != size {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            *phGlobal = GlobalAlloc(GMEM_MOVEABLE, size as usize);
            let memory = GlobalLock(*phGlobal);
            memcpy(memory, pBuffer as *const core::ffi::c_void, size as usize);
            pBuffer = pBuffer.add(size as usize);
            GlobalUnlock(*phGlobal);
        } else {
            *phGlobal = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HGLOBAL_UserFree(pFlags: *mut ULONG, phGlobal: *mut HGLOBAL) {

    fn LOWORD(dw: DWORD_PTR) -> u16 {
        (dw & 0xFFFF) as u16
    }

    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phGlobal
    );

    if LOWORD(*pFlags as DWORD_PTR) != MSHCTX_INPROC as u16 && !(*phGlobal).is_null() {
        GlobalFree(*phGlobal);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HBITMAP_UserSize(flags: *mut ULONG, size: ULONG, bmp: *mut HBITMAP) -> ULONG {
    TRACE!(b"(%s, %d, %p)\n\0".as_ptr() as *const i8, debugstr_user_flags(flags), size, *bmp);

    let mut size = size;
    ALIGN_LENGTH(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*flags) == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*bmp).is_null() {
            size += core::mem::size_of::<ULONG>() as ULONG;
            size += FIELD_OFFSET(userBITMAP, cbSize) as ULONG;
            size += GetBitmapBits(*bmp, 0, core::ptr::null_mut()) as ULONG;
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn HBITMAP_UserMarshal(flags: *mut ULONG, buffer: *mut u8, bmp: *mut HBITMAP) -> *mut u8 {

    const MSHCTX_INPROC: u32 = 3;
    const WDT_INPROC_CALL: ULONG = 1;
    const WDT_REMOTE_CALL: ULONG = 2;

    TRACE!(b"(%s, %p, %p)\n\0".as_ptr() as *const i8, debugstr_user_flags(flags), buffer as *const core::ffi::c_void, *bmp as *const core::ffi::c_void);

    let mut buffer = buffer;
    ALIGN_POINTER(buffer, 3);

    if ((*flags) & 0xFFFF) == MSHCTX_INPROC {
        *(buffer as *mut ULONG) = WDT_INPROC_CALL;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
        *(buffer as *mut ULONG) = (*bmp) as ULONG_PTR as ULONG;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
    } else {
        *(buffer as *mut ULONG) = WDT_REMOTE_CALL;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
        *(buffer as *mut ULONG) = (*bmp) as ULONG_PTR as ULONG;
        buffer = buffer.add(core::mem::size_of::<ULONG>());

        if !(*bmp).is_null() {
            const HEADER_SIZE: ULONG = core::mem::offset_of!(_userBITMAP, cbSize) as ULONG;
            let mut bitmap: BITMAP = core::mem::zeroed();
            let bitmap_size: ULONG;

            bitmap_size = GetBitmapBits(*bmp, 0, core::ptr::null_mut()) as ULONG;
            *(buffer as *mut ULONG) = bitmap_size;
            buffer = buffer.add(core::mem::size_of::<ULONG>());

            GetObjectW((*bmp) as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bitmap) as *mut core::ffi::c_void);
            memcpy(buffer as *mut core::ffi::c_void, core::ptr::addr_of!(bitmap) as *const core::ffi::c_void, HEADER_SIZE as usize);
            buffer = buffer.add(HEADER_SIZE as usize);

            GetBitmapBits(*bmp, bitmap_size as i32, buffer as *mut core::ffi::c_void);
            buffer = buffer.add(bitmap_size as usize);
        }
    }
    buffer
}
#[no_mangle]
pub unsafe extern "C" fn HBITMAP_UserUnmarshal(flags: *mut ULONG, mut buffer: *mut u8, bmp: *mut HBITMAP) -> *mut u8 {

    const WDT_INPROC_CALL: ULONG = 1;
    const WDT_REMOTE_CALL: ULONG = 2;

    buffer = ((buffer as usize + 3) & !3) as *mut u8;

    let context = *(buffer as *const ULONG);
    buffer = buffer.add(core::mem::size_of::<ULONG>());

    if context == WDT_INPROC_CALL {
        *bmp = *(buffer as *const HBITMAP);
        buffer = buffer.add(core::mem::size_of::<HBITMAP>());
    } else if context == WDT_REMOTE_CALL {
        let handle = *(buffer as *const ULONG);
        buffer = buffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            let header_size = core::mem::offset_of!(userBITMAP, cbSize) as ULONG;
            let mut bitmap: BITMAP = core::mem::zeroed();
            let bitmap_size: ULONG;

            bitmap_size = *(buffer as *const ULONG);
            buffer = buffer.add(core::mem::size_of::<ULONG>());
            let bits = HeapAlloc(GetProcessHeap(), 0, bitmap_size as SIZE_T);

            memcpy(&mut bitmap as *mut BITMAP as *mut core::ffi::c_void, buffer as *const core::ffi::c_void, header_size as usize);
            buffer = buffer.add(header_size as usize);

            memcpy(bits, buffer as *const core::ffi::c_void, bitmap_size as usize);
            buffer = buffer.add(bitmap_size as usize);

            bitmap.bmBits = bits;
            *bmp = CreateBitmapIndirect(&bitmap);

            HeapFree(GetProcessHeap(), 0, bits);
        } else {
            *bmp = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    buffer
}
#[no_mangle]
pub unsafe extern "C" fn HBITMAP_UserFree(flags: *mut ULONG, bmp: *mut HBITMAP) {

    TRACE!(
        b"(%s, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        *bmp,
    );

    if LOWORD(*flags as DWORD_PTR) != MSHCTX_INPROC {
        DeleteObject(*bmp as HGLOBAL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HPALETTE_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    phPal: *mut HPALETTE,
) -> ULONG {
    FIXME!(b":stub\n\0".as_ptr() as PCSTR);
    StartingSize
}
#[no_mangle]
pub unsafe extern "C" fn HPALETTE_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    phPal: *mut HPALETTE,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HPALETTE_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    phPal: *mut HPALETTE,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HPALETTE_UserFree(pFlags: *mut ULONG, phPal: *mut HPALETTE) {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILE_UserSize(pFlags: *mut ULONG, StartingSize: ULONG, phmf: *mut HMETAFILE) -> ULONG {
    let mut size = StartingSize;

    TRACE!(
        b"(%s, %d, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        *phmf
    );

    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*pFlags) == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG_PTR>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phmf).is_null() {
            let mfsize: UINT;

            size += 2 * core::mem::size_of::<ULONG>() as ULONG;
            mfsize = GetMetaFileBitsEx(*phmf, 0, core::ptr::null_mut());
            size += mfsize;
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILE_UserMarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phmf: *mut HMETAFILE) -> *mut u8 {

    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phmf
    );

    pBuffer = ALIGNED_POINTER(pBuffer, 3) as *mut u8;

    if ((*pFlags) & 0xFFFF) == MSHCTX_INPROC {
        if core::mem::size_of::<HMETAFILE>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HMETAFILE) = *phmf;
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILE>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = (*phmf as ULONG_PTR) as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phmf).is_null() {
            let mfsize = GetMetaFileBitsEx(*phmf, 0, core::ptr::null_mut());

            *(pBuffer as *mut ULONG) = mfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = mfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            GetMetaFileBitsEx(*phmf, mfsize, pBuffer);
            pBuffer = pBuffer.add(mfsize as usize);
        }
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILE_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phmf: *mut HMETAFILE) -> *mut u8 {
    let fContext: ULONG;

    TRACE!(b"(%s, %p, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, phmf);

    ALIGN_POINTER(pBuffer, 3);

    fContext = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL) && (core::mem::size_of::<HMETAFILE>() < 8)) ||
       ((fContext == WDT_INPROC64_CALL) && (core::mem::size_of::<HMETAFILE>() == 8))
    {
        *phmf = *(pBuffer as *const HMETAFILE);
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILE>());
    }
    else if fContext == WDT_REMOTE_CALL
    {
        let handle: ULONG;

        handle = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0
        {
            let size: ULONG;
            size = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            if size != *(pBuffer as *const ULONG)
            {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null_mut());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *phmf = SetMetaFileBitsEx(size, pBuffer as *const core::ffi::c_void);
            pBuffer = pBuffer.add(size as usize);
        }
        else
        {
            *phmf = core::ptr::null_mut();
        }
    }
    else
    {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null_mut());
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILE_UserFree(pFlags: *mut ULONG, phmf: *mut HMETAFILE) {

    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phmf,
    );

    if LOWORD(*pFlags) != MSHCTX_INPROC {
        DeleteMetaFile(*phmf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HENHMETAFILE_UserSize(pFlags: *mut ULONG, mut size: ULONG, phEmf: *mut HENHMETAFILE) -> ULONG {
    TRACE!(b"(%s, %d, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), size, *phEmf);

    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*pFlags) == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG_PTR>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phEmf).is_null() {
            let emfsize: UINT;

            size += 2 * core::mem::size_of::<ULONG>() as ULONG;
            emfsize = GetEnhMetaFileBits(*phEmf, 0, core::ptr::null_mut());
            size += emfsize;
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn HENHMETAFILE_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phEmf: *mut HENHMETAFILE,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer as LPVOID,
        *phEmf as LPVOID,
    );

    pBuffer = ALIGN_POINTER(pBuffer, 3);

    if LOWORD(*pFlags) == 3 {
        if core::mem::size_of::<HENHMETAFILE>() == 8 {
            *(pBuffer as *mut ULONG) = 0x80000001;
        } else {
            *(pBuffer as *mut ULONG) = 0x80000000;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HENHMETAFILE) = *phEmf;
        pBuffer = pBuffer.add(core::mem::size_of::<HENHMETAFILE>());
    } else {
        *(pBuffer as *mut ULONG) = 0x80000002;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = *phEmf as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phEmf).is_null() {
            let emfsize = GetEnhMetaFileBits(*phEmf, 0, core::ptr::null_mut());

            *(pBuffer as *mut ULONG) = emfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = emfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            GetEnhMetaFileBits(*phEmf, emfsize, pBuffer);
            pBuffer = pBuffer.add(emfsize as usize);
        }
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HENHMETAFILE_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phEmf: *mut HENHMETAFILE) -> *mut u8 {

    TRACE!(b"(%s, %p, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, phEmf);

    ALIGN_POINTER!(pBuffer, 3);

    let fContext = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL) && (core::mem::size_of::<HENHMETAFILE>() < 8)) ||
       ((fContext == WDT_INPROC64_CALL) && (core::mem::size_of::<HENHMETAFILE>() == 8))
    {
        *phEmf = *(pBuffer as *const HENHMETAFILE);
        pBuffer = pBuffer.add(core::mem::size_of::<HENHMETAFILE>());
    }
    else if fContext == WDT_REMOTE_CALL
    {
        let handle = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0
        {
            let size = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            if size != *(pBuffer as *const ULONG)
            {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *phEmf = SetEnhMetaFileBits(size, pBuffer);
            pBuffer = pBuffer.add(size as usize);
        }
        else
        {
            *phEmf = core::ptr::null_mut();
        }
    }
    else
    {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HENHMETAFILE_UserFree(pFlags: *mut ULONG, phEmf: *mut HENHMETAFILE) {

    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phEmf,
    );

    if LOWORD(*pFlags as DWORD_PTR) != 1u16 {
        DeleteEnhMetaFile(*phEmf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILEPICT_UserSize(pFlags: *mut ULONG, mut size: ULONG, phMfp: *mut HMETAFILEPICT) -> ULONG {

    #[inline]
    unsafe fn LOWORD(l: DWORD_PTR) -> u16 {
        (l & 0xFFFF) as u16
    }

    TRACE!(b"(%s, %d, &%p)\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), size, *phMfp);

    size = ((size + 3) & !3);

    size += core::mem::size_of::<ULONG>() as ULONG;

    if LOWORD(*pFlags as DWORD_PTR) == 1 {
        size += core::mem::size_of::<HMETAFILEPICT>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phMfp).is_null() {
            let mfpict = GlobalLock(*phMfp as HGLOBAL) as *mut METAFILEPICT;

            size += 3 * core::mem::size_of::<ULONG>() as ULONG;
            size += core::mem::size_of::<ULONG>() as ULONG;

            size = HMETAFILE_UserSize(pFlags, size, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp as HGLOBAL);
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILEPICT_UserMarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phMfp: *mut HMETAFILEPICT) -> *mut u8 {

    TRACE!(b"(%s, %p, &%p)\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, *phMfp);

    pBuffer = ALIGNED_POINTER(pBuffer, 3) as *mut u8;

    if ((*pFlags) & 0xFFFF) == MSHCTX_INPROC {
        if core::mem::size_of::<HMETAFILEPICT>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HMETAFILEPICT) = *phMfp;
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILEPICT>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = (*phMfp) as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phMfp).is_null() {
            let mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
            let remmfpict = pBuffer as *mut remoteMETAFILEPICT;

            (*remmfpict).mm = (*mfpict).mm;
            (*remmfpict).xExt = (*mfpict).xExt;
            (*remmfpict).yExt = (*mfpict).yExt;
            pBuffer = pBuffer.add(3 * core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = USER_MARSHAL_PTR_PREFIX;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            pBuffer = HMETAFILE_UserMarshal(pFlags, pBuffer, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp);
        }
    }
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILEPICT_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, phMfp: *mut HMETAFILEPICT) -> *mut u8 {

    const WDT_INPROC_CALL: ULONG = 1;
    const WDT_INPROC64_CALL: ULONG = 4;
    const E_OUTOFMEMORY: HRESULT = 0x8007000E;

    TRACE!(b"(%s, %p, %p)\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, phMfp);

    ALIGN_POINTER(pBuffer, 3);

    let fContext = *(pBuffer as *mut ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if fContext == WDT_INPROC_CALL || fContext == WDT_INPROC64_CALL {
        *phMfp = *(pBuffer as *mut HMETAFILEPICT);
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILEPICT>());
    } else {
        let handle = *(pBuffer as *mut ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *phMfp = core::ptr::null_mut();

        if handle != 0 {
            let remmfpict = pBuffer as *const remoteMETAFILEPICT;

            *phMfp = GlobalAlloc(GMEM_MOVEABLE, core::mem::size_of::<METAFILEPICT>());
            if (*phMfp).is_null() {
                RpcRaiseException(E_OUTOFMEMORY as RPC_STATUS);
            }

            let mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
            (*mfpict).mm = (*remmfpict).mm;
            (*mfpict).xExt = (*remmfpict).xExt;
            (*mfpict).yExt = (*remmfpict).yExt;
            pBuffer = pBuffer.add(3 * core::mem::size_of::<ULONG>());
            let user_marshal_prefix = *(pBuffer as *mut ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            if user_marshal_prefix != USER_MARSHAL_PTR_PREFIX {
                RpcRaiseException(RPC_X_INVALID_TAG);
            }

            pBuffer = HMETAFILE_UserUnmarshal(pFlags, pBuffer, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp);
        }
    }
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn HMETAFILEPICT_UserFree(pFlags: *mut ULONG, phMfp: *mut HMETAFILEPICT) {

    const MSHCTX_INPROC: u16 = 3;

    #[repr(C)]
    struct METAFILEPICT {
        mm: i32,
        xExt: i32,
        yExt: i32,
        hMF: HMETAFILE,
    }

    TRACE!(b"(%s, &%p)\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), *phMfp);

    if (LOWORD(*pFlags as DWORD_PTR) != MSHCTX_INPROC) && !(*phMfp).is_null() {
        let mfpict: *mut METAFILEPICT;

        mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
        HMETAFILE_UserFree(pFlags, core::ptr::addr_of_mut!((*mfpict).hMF));
        GlobalUnlock(*phMfp);

        GlobalFree(*phMfp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn WdtpInterfacePointer_UserSize(
    pFlags: *mut ULONG,
    RealFlags: ULONG,
    StartingSize: ULONG,
    punk: *mut IUnknown,
    riid: REFIID,
) -> ULONG {

    const MSHLFLAGS_NORMAL: DWORD = 0;

    let mut marshal_size: DWORD = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%s, 0%x, %d, %p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        RealFlags,
        StartingSize,
        punk,
        debugstr_guid(riid),
    );

    hr = CoGetMarshalSizeMax(
        &mut marshal_size,
        riid,
        punk,
        LOWORD(RealFlags) as DWORD,
        core::ptr::null_mut(),
        MSHLFLAGS_NORMAL,
    );
    if FAILED(hr) != 0 {
        return StartingSize;
    }

    let mut StartingSize = StartingSize;
    StartingSize = ((StartingSize + 3) & !3);
    StartingSize += 2 * core::mem::size_of::<DWORD>() as ULONG;
    return StartingSize + marshal_size;
}
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserMarshal(
    pFlags: *mut ULONG,
    RealFlags: ULONG,
    pBuffer: *mut u8,
    punk: *mut IUnknown,
    riid: REFIID,
) -> *mut u8 {

    let h = GlobalAlloc(GMEM_MOVEABLE, 0);
    let mut stm: *mut IStream = core::ptr::null_mut();
    let size: DWORD;
    let ptr: LPVOID;

    TRACE!(
        b"(%s, 0x%x, %p, &%p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        RealFlags,
        pBuffer,
        punk,
        debugstr_guid(riid),
    );

    if h.is_null() {
        return core::ptr::null_mut();
    }

    if CreateStreamOnHGlobal(h, 1, &mut stm) != 0 {
        GlobalFree(h);
        return core::ptr::null_mut();
    }

    if CoMarshalInterface(stm, riid, punk, LOWORD(RealFlags as DWORD_PTR) as DWORD, core::ptr::null_mut(), 0) != 0 {
        IStream_Release(stm as *mut IUnknown);
        return pBuffer;
    }

    let mut pBuffer = pBuffer;
    pBuffer = ((pBuffer as usize + 3) & !3) as *mut u8;

    size = GlobalSize(h) as DWORD;

    *(pBuffer as *mut DWORD) = size;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    *(pBuffer as *mut DWORD) = size;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    ptr = GlobalLock(h);
    memcpy(pBuffer as *mut core::ffi::c_void, ptr, size as usize);
    GlobalUnlock(h);

    IStream_Release(stm as *mut IUnknown);
    pBuffer.add(size as usize)
}
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    ppunk: *mut *mut IUnknown,
    riid: REFIID,
) -> *mut u8 {

    TRACE!(
        b"(%s, %p, %p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        ppunk,
        debugstr_guid(riid),
    );

    let mut pBuffer = pBuffer;
    pBuffer = ((pBuffer as usize + 3) & !3) as *mut u8;

    let size = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    if size != *(pBuffer as *const DWORD) {
        RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
    }

    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    let h = GlobalAlloc(GMEM_MOVEABLE, size as usize);
    if h.is_null() {
        RaiseException(RPC_X_NO_MEMORY, 0, 0, core::ptr::null());
    }

    let mut stm: *mut IStream = core::ptr::null_mut();
    if CreateStreamOnHGlobal(h, 1, &mut stm) != S_OK {
        GlobalFree(h);
        RaiseException(RPC_X_NO_MEMORY, 0, 0, core::ptr::null());
    }

    let ptr = GlobalLock(h);
    memcpy(ptr, pBuffer as *const core::ffi::c_void, size as usize);
    GlobalUnlock(h);

    let orig = *ppunk;
    let hr = CoUnmarshalInterface(stm, riid, ppunk as *mut *mut core::ffi::c_void);
    IStream_Release(stm);

    if hr != S_OK {
        RaiseException(hr as DWORD, 0, 0, core::ptr::null());
    }

    if !orig.is_null() {
        IUnknown_Release(orig);
    }

    pBuffer.add(size as usize)
}
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserFree(punk: *mut IUnknown) {

    TRACE!(b"(%p)\n\0".as_ptr(), punk);
    if !punk.is_null() {
        IUnknown_Release(punk);
    }
}
#[no_mangle]
pub unsafe extern "C" fn STGMEDIUM_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    pStgMedium: *mut STGMEDIUM,
) -> ULONG {
    let mut size = StartingSize;

    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        pStgMedium,
    );

    ALIGN_LENGTH(size, 3);

    size += 2 * core::mem::size_of::<DWORD>() as ULONG;
    if (*pStgMedium).tymed != TYMED_NULL {
        size += core::mem::size_of::<DWORD>() as ULONG;
    }

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hGlobal.is_null() {
                size = HGLOBAL_UserSize(pFlags, size, core::ptr::addr_of_mut!((*pStgMedium).u.hGlobal));
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.lpszFileName.is_null() {
                TRACE!(
                    b"file name is %s\n\0".as_ptr() as *const i8,
                    debugstr_w((*pStgMedium).u.lpszFileName),
                );
                size += 3 * core::mem::size_of::<DWORD>() as ULONG
                    + (lstrlenW((*pStgMedium).u.lpszFileName) + 1) as ULONG
                        * core::mem::size_of::<WCHAR>() as ULONG;
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.pstm.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStream_QueryInterface(
                    (*pStgMedium).u.pstm,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut LPVOID,
                );
                size = WdtpInterfacePointer_UserSize(pFlags, LOWORD(*pFlags), size, unk, &IID_IStream);
                IUnknown_Release(unk);
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.pstg.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStorage_QueryInterface(
                    (*pStgMedium).u.pstg,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut LPVOID,
                );
                size = WdtpInterfacePointer_UserSize(pFlags, LOWORD(*pFlags), size, unk, &IID_IStorage);
                IUnknown_Release(unk);
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hBitmap.is_null() {
                size = HBITMAP_UserSize(pFlags, size, core::ptr::addr_of_mut!((*pStgMedium).u.hBitmap));
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hMetaFilePict.is_null() {
                size = HMETAFILEPICT_UserSize(pFlags, size, core::ptr::addr_of_mut!((*pStgMedium).u.hMetaFilePict));
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hEnhMetaFile.is_null() {
                size = HENHMETAFILE_UserSize(pFlags, size, core::ptr::addr_of_mut!((*pStgMedium).u.hEnhMetaFile));
            }
        }
        _ => {
            RaiseException(DV_E_TYMED, 0, 0, core::ptr::null_mut());
        }
    }

    if !(*pStgMedium).pUnkForRelease.is_null() {
        size = WdtpInterfacePointer_UserSize(
            pFlags,
            LOWORD(*pFlags),
            size,
            (*pStgMedium).pUnkForRelease,
            &IID_IUnknown,
        );
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn STGMEDIUM_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut STGMEDIUM,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pStgMedium,
    );

    let mut pBuffer = pBuffer;
    ALIGN_POINTER(pBuffer, 3);

    *(pBuffer as *mut DWORD) = (*pStgMedium).tymed;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    if (*pStgMedium).tymed != TYMED_NULL {
        *(pBuffer as *mut DWORD) = (*pStgMedium).u.pstg as DWORD_PTR as DWORD;
        pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    }

    *(pBuffer as *mut DWORD) = (*pStgMedium).pUnkForRelease as DWORD_PTR as DWORD;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hGlobal.is_null() {
                pBuffer = HGLOBAL_UserMarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hGlobal);
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.lpszFileName.is_null() {
                let len = lstrlenW((*pStgMedium).u.lpszFileName);
                *(pBuffer as *mut DWORD) = (len + 1) as DWORD;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                *(pBuffer as *mut DWORD) = 0;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                *(pBuffer as *mut DWORD) = (len + 1) as DWORD;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

                TRACE!(
                    b"file name is %s\n\0".as_ptr() as *const i8,
                    debugstr_w((*pStgMedium).u.lpszFileName),
                );
                memcpy(
                    pBuffer as *mut _,
                    (*pStgMedium).u.lpszFileName as *const _,
                    ((len + 1) * core::mem::size_of::<WCHAR>()) as usize,
                );
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.pstm.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStream_QueryInterface(
                    (*pStgMedium).u.pstm,
                    &IID_IUnknown,
                    &mut unk as *mut *mut IUnknown as *mut *mut _,
                );
                pBuffer = WdtpInterfacePointer_UserMarshal(
                    pFlags,
                    LOWORD(*pFlags),
                    pBuffer,
                    unk,
                    &IID_IStream,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.pstg.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStorage_QueryInterface(
                    (*pStgMedium).u.pstg,
                    &IID_IUnknown,
                    &mut unk as *mut *mut IUnknown as *mut *mut _,
                );
                pBuffer = WdtpInterfacePointer_UserMarshal(
                    pFlags,
                    LOWORD(*pFlags),
                    pBuffer,
                    unk,
                    &IID_IStorage,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hBitmap.is_null() {
                pBuffer = HBITMAP_UserMarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hBitmap);
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hMetaFilePict.is_null() {
                pBuffer = HMETAFILEPICT_UserMarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hMetaFilePict);
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).u.hEnhMetaFile.is_null() {
                pBuffer = HENHMETAFILE_UserMarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hEnhMetaFile);
            }
        }
        _ => {
            RaiseException(DV_E_TYMED, 0, 0, core::ptr::null_mut());
        }
    }

    if !(*pStgMedium).pUnkForRelease.is_null() {
        pBuffer = WdtpInterfacePointer_UserMarshal(
            pFlags,
            LOWORD(*pFlags),
            pBuffer,
            (*pStgMedium).pUnkForRelease,
            &IID_IUnknown,
        );
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn STGMEDIUM_UserUnmarshal(pFlags: *mut ULONG, mut pBuffer: *mut u8, pStgMedium: *mut STGMEDIUM) -> *mut u8 {
    let mut content: DWORD = 0;
    let releaseunk: DWORD;

    pBuffer = core::ptr::read_unaligned((pBuffer as usize + 3 & !3) as *const u8) as *mut u8;
    pBuffer = ((pBuffer as usize + 3) & !3) as *mut u8;

    TRACE!(b"(%s, %p, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, pStgMedium);

    (*pStgMedium).tymed = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    if (*pStgMedium).tymed != TYMED_NULL {
        content = *(pBuffer as *const DWORD);
        pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    }
    releaseunk = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HGLOBAL_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hGlobal);
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if content != 0 {
                let conformance: DWORD;
                let variance: DWORD;
                conformance = *(pBuffer as *const DWORD);
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                if *(pBuffer as *const DWORD) != 0 {
                    ERR!(b"invalid offset %d\n\0".as_ptr() as *const i8, *(pBuffer as *const DWORD));
                    RpcRaiseException(RPC_S_INVALID_BOUND);
                    return core::ptr::null_mut();
                }
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                variance = *(pBuffer as *const DWORD);
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                if conformance != variance {
                    ERR!(b"conformance (%d) and variance (%d) should be equal\n\0".as_ptr() as *const i8, conformance, variance);
                    RpcRaiseException(RPC_S_INVALID_BOUND);
                    return core::ptr::null_mut();
                }
                if conformance > 0x7fffffff {
                    ERR!(b"conformance 0x%x too large\n\0".as_ptr() as *const i8, conformance);
                    RpcRaiseException(RPC_S_INVALID_BOUND);
                    return core::ptr::null_mut();
                }
                (*pStgMedium).u.lpszFileName = CoTaskMemAlloc((conformance as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
                if (*pStgMedium).u.lpszFileName.is_null() {
                    RpcRaiseException(ERROR_OUTOFMEMORY);
                }
                TRACE!(b"unmarshalled file name is %s\n\0".as_ptr() as *const i8, debugstr_wn(pBuffer as *const WCHAR, variance as i32));
                memcpy((*pStgMedium).u.lpszFileName as *mut core::ffi::c_void, pBuffer as *const core::ffi::c_void, (variance as usize) * core::mem::size_of::<WCHAR>());
                pBuffer = pBuffer.add((variance as usize) * core::mem::size_of::<WCHAR>());
            } else {
                (*pStgMedium).u.lpszFileName = core::ptr::null_mut();
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = WdtpInterfacePointer_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.pstm as *mut *mut IStream as *mut *mut IUnknown, &IID_IStream);
            } else {
                if !(*pStgMedium).u.pstm.is_null() {
                    IStream_Release((*pStgMedium).u.pstm);
                }
                (*pStgMedium).u.pstm = core::ptr::null_mut();
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = WdtpInterfacePointer_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.pstg as *mut *mut IStorage as *mut *mut IUnknown, &IID_IStorage);
            } else {
                if !(*pStgMedium).u.pstg.is_null() {
                    IStorage_Release((*pStgMedium).u.pstg);
                }
                (*pStgMedium).u.pstg = core::ptr::null_mut();
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HBITMAP_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hBitmap);
            } else {
                (*pStgMedium).u.hBitmap = core::ptr::null_mut();
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HMETAFILEPICT_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hMetaFilePict);
            } else {
                (*pStgMedium).u.hMetaFilePict = core::ptr::null_mut();
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HENHMETAFILE_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).u.hEnhMetaFile);
            } else {
                (*pStgMedium).u.hEnhMetaFile = core::ptr::null_mut();
            }
        }
        _ => {
            RaiseException(DV_E_TYMED, 0, 0, core::ptr::null());
        }
    }

    if releaseunk != 0 {
        pBuffer = WdtpInterfacePointer_UserUnmarshal(pFlags, pBuffer, &mut (*pStgMedium).pUnkForRelease, &IID_IUnknown);
    }

    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn STGMEDIUM_UserFree(flags: *mut ULONG, med: *mut STGMEDIUM) {

    fn LOWORD(l: DWORD) -> WORD {
        (l & 0xffff) as WORD
    }

    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_user_flags(flags), med);

    match (*med).tymed {
        TYMED_NULL | TYMED_FILE | TYMED_ISTREAM | TYMED_ISTORAGE => {
            ReleaseStgMedium(med);
        }
        TYMED_HGLOBAL | TYMED_GDI | TYMED_MFPICT | TYMED_ENHMF => {
            if LOWORD(*flags) == MSHCTX_INPROC {
                (*med).tymed = TYMED_NULL;
            }
            ReleaseStgMedium(med);
        }
        _ => {
            RaiseException(DV_E_TYMED, 0, 0, core::ptr::null());
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ASYNC_STGMEDIUM_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> ULONG {
    STGMEDIUM_UserSize(pFlags, StartingSize, pStgMedium)
}
#[no_mangle]
pub unsafe extern "C" fn ASYNC_STGMEDIUM_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> *mut u8 {
    STGMEDIUM_UserMarshal(pFlags, pBuffer, pStgMedium)
}
#[no_mangle]
pub unsafe extern "C" fn ASYNC_STGMEDIUM_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> *mut u8 {
    TRACE!(b"\n\0".as_ptr() as *const i8);
    STGMEDIUM_UserUnmarshal(pFlags, pBuffer, pStgMedium)
}
#[no_mangle]
pub unsafe extern "C" fn ASYNC_STGMEDIUM_UserFree(pFlags: *mut ULONG, pStgMedium: *mut ASYNC_STGMEDIUM) {
    STGMEDIUM_UserFree(pFlags, pStgMedium);
}
#[no_mangle]
pub unsafe extern "C" fn FLAG_STGMEDIUM_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    pStgMedium: *mut FLAG_STGMEDIUM,
) -> ULONG {
    FIXME!(b":stub\n\0".as_ptr() as PCHAR);
    StartingSize
}
#[no_mangle]
pub unsafe extern "C" fn FLAG_STGMEDIUM_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut FLAG_STGMEDIUM,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as PCHAR);
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn FLAG_STGMEDIUM_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut FLAG_STGMEDIUM,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}
#[no_mangle]
pub unsafe extern "C" fn FLAG_STGMEDIUM_UserFree(
    pFlags: *mut ULONG,
    pStgMedium: *mut FLAG_STGMEDIUM,
) {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "C" fn SNB_UserSize(pFlags: *mut ULONG, StartingSize: ULONG, pSnb: *mut SNB) -> ULONG {

    let mut size = StartingSize;

    TRACE!(b"(%s, %d, %p\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), StartingSize, pSnb);

    size = ALIGNED_LENGTH(size, 3);

    size += 3 * core::mem::size_of::<ULONG>() as ULONG;

    if !(*pSnb).is_null() {
        let mut ptrW = *pSnb;

        while !(*ptrW).is_null() {
            size += ((lstrlenW(*ptrW) + 1) as ULONG) * core::mem::size_of::<WCHAR>() as ULONG;
            ptrW = ptrW.offset(1);
        }
    }

    size
}
#[no_mangle]
pub unsafe extern "C" fn SNB_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pSnb: *mut *mut *mut u16,
) -> *mut u8 {

    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pSnb,
    );

    let mut pBuffer = pBuffer;
    ALIGN_POINTER(pBuffer, 3);

    let wire = pBuffer as *mut SNB_wire;
    (*wire).charcnt = 0;
    (*wire).strcnt = 0;
    let mut size = 3 * core::mem::size_of::<ULONG>();

    if !(*pSnb).is_null() {
        let mut ptrW = *pSnb;
        let mut dataW = (*wire).data.as_mut_ptr();

        while !(*ptrW).is_null() {
            let len = lstrlenW(*ptrW) as ULONG + 1;

            (*wire).strcnt += 1;
            (*wire).charcnt += len;
            core::ptr::copy_nonoverlapping(
                *ptrW,
                dataW,
                len as usize,
            );
            dataW = dataW.add(len as usize);

            size += (len as usize) * core::mem::size_of::<u16>();
            ptrW = ptrW.add(1);
        }
    }

    (*wire).datalen = (*wire).charcnt;
    pBuffer.add(size)
}
#[no_mangle]
pub unsafe extern "C" fn SNB_UserUnmarshal(pFlags: *mut ULONG, pBuffer: *mut u8, pSnb: *mut SNB) -> *mut u8 {

    let umcb = pFlags as *mut USER_MARSHAL_CB;
    let wire = pBuffer as *mut SNB_wire;

    TRACE!(b"(%s, %p, %p)\n\0".as_ptr() as *const i8, debugstr_user_flags(pFlags), pBuffer, pSnb);

    if !(*pSnb).is_null() {
        ((*(*umcb).pStubMsg).pfnFree.unwrap())(*pSnb as *mut core::ffi::c_void);
    }

    if (*wire).datalen == 0 {
        *pSnb = core::ptr::null_mut();
    } else {
        let mut src = (*wire).data.as_mut_ptr();
        let mut dest: *mut WCHAR;
        let mut ptrW: *mut *mut WCHAR;

        ptrW = ((*(*umcb).pStubMsg).pfnAllocate.unwrap())(
            ((*wire).strcnt + 1) * core::mem::size_of::<*mut WCHAR>() + (*wire).datalen * core::mem::size_of::<WCHAR>()
        ) as *mut *mut WCHAR;
        *pSnb = ptrW as SNB;
        dest = (ptrW.add(((*wire).strcnt + 1) as usize)) as *mut WCHAR;

        for _ in 0..(*wire).strcnt {
            let len = lstrlenW(src) as ULONG;
            memcpy(dest as *mut core::ffi::c_void, src as *const core::ffi::c_void, ((len + 1) * core::mem::size_of::<WCHAR>() as ULONG) as usize);
            *ptrW = dest;
            src = src.add((len + 1) as usize);
            dest = dest.add((len + 1) as usize);
            ptrW = ptrW.add(1);
        }
        *ptrW = core::ptr::null_mut();
    }

    pBuffer.add(3 * core::mem::size_of::<ULONG>() + ((*wire).datalen * core::mem::size_of::<WCHAR>() as ULONG) as usize)
}
#[no_mangle]
pub unsafe extern "C" fn SNB_UserFree(pFlags: *mut ULONG, pSnb: *mut *mut *mut u16) {
    let umcb = pFlags as *mut USER_MARSHAL_CB;
    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, pSnb);
    if !(*pSnb).is_null() {
        ((*(*umcb).pStubMsg).pfnFree.unwrap())(*pSnb as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn IClassFactory_CreateInstance_Proxy(
    This: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr(),
        pUnkOuter,
        debugstr_guid(riid),
        ppvObject,
    );
    *ppvObject = core::ptr::null_mut();
    if !pUnkOuter.is_null() {
        ERR!(b"aggregation is not allowed on remote objects\n\0".as_ptr());
        return CLASS_E_NOAGGREGATION;
    }
    IClassFactory_RemoteCreateInstance_Proxy(This, riid, ppvObject as *mut *mut IUnknown)
}
#[no_mangle]
pub unsafe extern "C" fn IClassFactory_CreateInstance_Stub(
    This: *mut IClassFactory,
    riid: *const GUID,
    ppvObject: *mut *mut IUnknown,
) -> HRESULT {

    TRACE!(b"(%s, %p)\n\0".as_ptr() as *const i8, debugstr_guid(riid), ppvObject);
    IClassFactory_CreateInstance(This, core::ptr::null_mut(), riid, ppvObject as *mut *mut core::ffi::c_void)
}
#[no_mangle]
pub unsafe extern "C" fn IClassFactory_LockServer_Proxy(
    This: *mut IClassFactory,
    fLock: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as PCSTR);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IClassFactory_LockServer_Stub(
    This: *mut IClassFactory,
    fLock: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IEnumUnknown_Next_Proxy(
    This: *mut IEnumUnknown,
    celt: ULONG,
    rgelt: *mut *mut IUnknown,
    pceltFetched: *mut ULONG
) -> HRESULT {

    let mut fetched: ULONG = 0;
    let pceltFetched = if pceltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pceltFetched
    };
    IEnumUnknown_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumUnknown_Next_Stub(
    This: *mut IEnumUnknown,
    celt: ULONG,
    rgelt: *mut *mut IUnknown,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumUnknown_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn IBindCtx_SetBindOptions_Proxy(
    This: *mut IBindCtx,
    pbindopts: *mut BIND_OPTS,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IBindCtx_SetBindOptions_Stub(
    This: *mut IBindCtx,
    pbindopts: *mut BIND_OPTS2,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IBindCtx_GetBindOptions_Proxy(
    This: *mut IBindCtx,
    pbindopts: *mut BIND_OPTS,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr());
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IBindCtx_GetBindOptions_Stub(
    This: *mut IBindCtx,
    pbindopts: *mut BIND_OPTS2,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    return E_NOTIMPL;
}
#[no_mangle]
pub unsafe extern "C" fn IEnumMoniker_Next_Proxy(
    This: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let mut fetched: ULONG = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr(), This, celt, rgelt, pceltFetched);
    let pceltFetched = if pceltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pceltFetched
    };
    IEnumMoniker_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumMoniker_Next_Stub(
    This: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);
    *pceltFetched = 0;
    hr = IEnumMoniker_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn IRunnableObject_IsRunning_Proxy(
    This: *mut IRunnableObject,
) -> BOOL {
    let mut rv: BOOL = 0;
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    core::ptr::write_bytes(&mut rv as *mut BOOL, 0, 1);
    rv
}
#[no_mangle]
pub unsafe extern "C" fn IRunnableObject_IsRunning_Stub(
    This: *mut IRunnableObject
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IMoniker_BindToObject_Proxy(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riidResult: REFIID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IMoniker_BindToObject_Stub(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riidResult: *const GUID,
    ppvResult: *mut *mut IUnknown,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IMoniker_BindToStorage_Proxy(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%p %p %s %p)\n\0".as_ptr() as *const i8,
        This,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvObj,
    );
    IMoniker_RemoteBindToStorage_Proxy(This, pbc, pmkToLeft, riid, ppvObj as *mut *mut IUnknown)
}
#[no_mangle]
pub unsafe extern "C" fn IMoniker_BindToStorage_Stub(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvObj: *mut *mut IUnknown,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%p %p %s %p)\n\0".as_ptr(),
        This,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvObj,
    );
    IMoniker_BindToStorage(This, pbc, pmkToLeft, riid, ppvObj as *mut *mut core::ffi::c_void)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumString_Next_Proxy(
    This: *mut IEnumString,
    celt: ULONG,
    rgelt: *mut LPOLESTR,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);
    let pceltFetched = if pceltFetched.is_null() { &mut fetched as *mut ULONG } else { pceltFetched };
    IEnumString_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumString_Next_Stub(
    This: *mut IEnumString,
    celt: ULONG,
    rgelt: *mut LPOLESTR,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut hr: HRESULT;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);
    *pceltFetched = 0;
    hr = IEnumString_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn ISequentialStream_Read_Proxy(
    This: *mut ISequentialStream,
    pv: *mut core::ffi::c_void,
    cb: ULONG,
    pcbRead: *mut ULONG
) -> HRESULT {
    let mut read: ULONG = 0;
    let hr: HRESULT;

    TRACE!(b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8, This, pv, cb, pcbRead);

    hr = ISequentialStream_RemoteRead_Proxy(This, pv, cb, &mut read);
    if !pcbRead.is_null() {
        *pcbRead = read;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn ISequentialStream_Read_Stub(
    This: *mut IStream,
    pv: *mut u8,
    cb: DWORD,
    pcbRead: *mut DWORD,
) -> HRESULT {

    TRACE!(b"(%p)->(%p, %d, %p)\n\0".as_ptr(), This, pv, cb, pcbRead);
    ISequentialStream_Read(This, pv as *mut core::ffi::c_void, cb, pcbRead)
}
#[no_mangle]
pub unsafe extern "C" fn ISequentialStream_Write_Proxy(
    This: *mut ISequentialStream,
    pv: *const core::ffi::c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {

    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr(),
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = ISequentialStream_RemoteWrite_Proxy(This, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn ISequentialStream_Write_Stub(
    This: *mut ISequentialStream,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );
    ISequentialStream_Write(This, pv as *const core::ffi::c_void, cb, pcbWritten)
}
#[no_mangle]
pub unsafe extern "C" fn IStream_Seek_Proxy(
    This: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER,
) -> HRESULT {

    let mut newpos: ULARGE_INTEGER = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlongdlibMove.QuadPart,
        dwOrigin,
        plibNewPosition,
    );

    hr = IStream_RemoteSeek_Proxy(This, dlibMove, dwOrigin, &mut newpos);
    if !plibNewPosition.is_null() {
        *plibNewPosition = newpos;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn IStream_Seek_Stub(
    This: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %d, %p)\n\0".as_ptr(),
        This,
        wine_dbgstr_longlongdlibMove.QuadPart,
        dwOrigin,
        plibNewPosition
    );
    IStream_Seek(This, dlibMove, dwOrigin, plibNewPosition)
}
#[no_mangle]
pub unsafe extern "C" fn IStream_CopyTo_Proxy(
    This: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {

    let mut read: ULARGE_INTEGER = core::mem::zeroed();
    let mut written: ULARGE_INTEGER = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %s, %p, %p)\n\0".as_ptr(),
        This,
        pstm,
        wine_dbgstr_longlong(*(*core::ptr::addr_of!(cb)).QuadPart),
        pcbRead,
        pcbWritten,
    );

    hr = IStream_RemoteCopyTo_Proxy(
        This,
        pstm,
        cb,
        core::ptr::addr_of_mut!(read),
        core::ptr::addr_of_mut!(written),
    );
    if !pcbRead.is_null() {
        *pcbRead = read;
    }
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn IStream_CopyTo_Stub(
    This: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%p, %s, %p, %p)\n\0".as_ptr(),
        This,
        pstm,
        wine_dbgstr_longlong((*(&cb as *const ULARGE_INTEGER as *const i64))),
        pcbRead,
        pcbWritten,
    );

    IStream_CopyTo(This, pstm, cb, pcbRead, pcbWritten)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumSTATSTG_Next_Proxy(
    This: *mut IEnumSTATSTG,
    celt: ULONG,
    rgelt: *mut STATSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr() as PCHAR, This, celt, rgelt, pceltFetched);
    let pceltFetched = if pceltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pceltFetched
    };
    IEnumSTATSTG_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumSTATSTG_Next_Stub(
    This: *mut IEnumSTATSTG,
    celt: ULONG,
    rgelt: *mut STATSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumSTATSTG_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn IStorage_OpenStream_Proxy(
    This: *mut IStorage,
    pwcsName: LPCOLESTR,
    reserved1: *mut core::ffi::c_void,
    grfMode: DWORD,
    reserved2: DWORD,
    ppstm: *mut *mut IStream,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%s, %p, %08x, %d %p)\n\0".as_ptr(),
        This,
        debugstr_w(pwcsName),
        reserved1,
        grfMode,
        reserved2,
        ppstm,
    );
    if !reserved1.is_null() {
        WARN!(b"reserved1 %p\n\0".as_ptr(), reserved1);
    }

    IStorage_RemoteOpenStream_Proxy(
        This,
        pwcsName,
        0,
        core::ptr::null_mut(),
        grfMode,
        reserved2,
        ppstm,
    )
}
#[no_mangle]
pub unsafe extern "C" fn IStorage_OpenStream_Stub(
    This: *mut IStorage,
    pwcsName: LPCWSTR,
    cbReserved1: ULONG,
    reserved1: *mut u8,
    grfMode: DWORD,
    reserved2: DWORD,
    ppstm: *mut *mut IStream
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %d, %p, %08x, %d %p)\n\0".as_ptr(),
        This,
        debugstr_w(pwcsName),
        cbReserved1,
        reserved1,
        grfMode,
        reserved2,
        ppstm
    );
    if cbReserved1 != 0 || !reserved1.is_null() {
        WARN!(b"cbReserved1 %d reserved1 %p\n\0".as_ptr(), cbReserved1, reserved1);
    }

    IStorage_OpenStream(This, pwcsName, core::ptr::null_mut(), grfMode, reserved2, ppstm)
}
#[no_mangle]
pub unsafe extern "C" fn IStorage_EnumElements_Proxy(
    This: *mut IStorage,
    reserved1: DWORD,
    reserved2: *mut core::ffi::c_void,
    reserved3: DWORD,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%d, %p, %d, %p)\n\0".as_ptr(),
        This,
        reserved1,
        reserved2,
        reserved3,
        ppenum,
    );
    if !reserved2.is_null() {
        WARN!(b"reserved2 %p\n\0".as_ptr(), reserved2);
    }

    IStorage_RemoteEnumElements_Proxy(
        This,
        reserved1,
        0,
        core::ptr::null_mut(),
        reserved3,
        ppenum,
    )
}
#[no_mangle]
pub unsafe extern "C" fn IStorage_EnumElements_Stub(
    This: *mut IStorage,
    reserved1: DWORD,
    cbReserved2: ULONG,
    reserved2: *mut u8,
    reserved3: DWORD,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%d, %d, %p, %d, %p)\n\0".as_ptr(),
        This,
        reserved1,
        cbReserved2,
        reserved2,
        reserved3,
        ppenum,
    );
    if cbReserved2 != 0 || !reserved2.is_null() {
        WARN!(
            b"cbReserved2 %d reserved2 %p\n\0".as_ptr(),
            cbReserved2,
            reserved2,
        );
    }

    IStorage_EnumElements(This, reserved1, core::ptr::null_mut(), reserved3, ppenum)
}
#[no_mangle]
pub unsafe extern "C" fn ILockBytes_ReadAt_Proxy(
    This: *mut core::ffi::c_void,
    ulOffset: ULARGE_INTEGER,
    pv: *mut core::ffi::c_void,
    cb: DWORD,
    pcbRead: *mut DWORD,
) -> HRESULT {

    let mut read: DWORD = 0;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr(),
        This,
        wine_dbgstr_longlong((*core::ptr::addr_of!(ulOffset.QuadPart)).clone()),
        pv,
        cb,
        pcbRead,
    );

    let hr = ILockBytes_RemoteReadAt_Proxy(This, ulOffset, pv, cb, &mut read);
    if !pcbRead.is_null() {
        *pcbRead = read;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn ILockBytes_ReadAt_Stub(
    This: *mut core::ffi::c_void,
    ulOffset: ULARGE_INTEGER,
    pv: *mut u8,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlong(*(*core::ptr::addr_of!(ulOffset)).QuadPart),
        pv,
        cb,
        pcbRead,
    );
    ILockBytes_ReadAt(This, ulOffset, pv, cb, pcbRead)
}
#[no_mangle]
pub unsafe extern "C" fn ILockBytes_WriteAt_Proxy(
    This: *mut core::ffi::c_void,
    ulOffset: ULARGE_INTEGER,
    pv: *const core::ffi::c_void,
    cb: DWORD,
    pcbWritten: *mut DWORD,
) -> HRESULT {

    let mut written: DWORD = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlong(ulOffset.QuadPart),
        pv,
        cb,
        pcbWritten,
    );

    hr = ILockBytes_RemoteWriteAt_Proxy(This, ulOffset, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn ILockBytes_WriteAt_Stub(
    This: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlongulOffset.QuadPart,
        pv,
        cb,
        pcbWritten,
    );
    ILockBytes_WriteAt(This, ulOffset, pv, cb, pcbWritten)
}
#[no_mangle]
pub unsafe extern "C" fn IFillLockBytes_FillAppend_Proxy(
    This: *mut IFillLockBytes,
    pv: *const core::ffi::c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr(),
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = IFillLockBytes_RemoteFillAppend_Proxy(This, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn IFillLockBytes_FillAppend_Stub(
    This: *mut IFillLockBytes,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8, This, pv, cb, pcbWritten);
    IFillLockBytes_FillAppend(This, pv, cb, pcbWritten)
}
#[no_mangle]
pub unsafe extern "C" fn IFillLockBytes_FillAt_Proxy(
    This: *mut IFillLockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const core::ffi::c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {

    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlong(ulOffset.QuadPart.quad_part as i64),
        pv,
        cb,
        pcbWritten,
    );

    hr = IFillLockBytes_RemoteFillAt_Proxy(This, ulOffset, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn IFillLockBytes_FillAt_Stub(
    This: *mut IFillLockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        wine_dbgstr_longlong(ulOffset.QuadPart),
        pv,
        cb,
        pcbWritten,
    );
    IFillLockBytes_FillAt(This, ulOffset, pv, cb, pcbWritten)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumFORMATETC_Next_Proxy(
    This: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let mut fetched: ULONG = 0;
    let pceltFetched = if pceltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pceltFetched
    };
    IEnumFORMATETC_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumFORMATETC_Next_Stub(
    This: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let mut hr: HRESULT;
    *pceltFetched = 0;
    hr = IEnumFORMATETC_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn IEnumSTATDATA_Next_Proxy(
    This: *mut IEnumSTATDATA,
    celt: ULONG,
    rgelt: *mut STATDATA,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr(), This, celt, rgelt, pceltFetched);
    let pceltFetched = if pceltFetched.is_null() {
        &mut fetched as *mut ULONG
    } else {
        pceltFetched
    };
    IEnumSTATDATA_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumSTATDATA_Next_Stub(
    This: *mut IEnumSTATDATA,
    celt: ULONG,
    rgelt: *mut STATDATA,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8, This, celt, rgelt, pceltFetched);
    *pceltFetched = 0;
    hr = IEnumSTATDATA_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnDataChange_Proxy(
    This: *mut IAdviseSink,
    pFormatetc: *mut FORMATETC,
    pStgmed: *mut STGMEDIUM,
) {

    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr(),
        This,
        pFormatetc,
        pStgmed,
    );
    IAdviseSink_RemoteOnDataChange_Proxy(This, pFormatetc, pStgmed);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnDataChange_Stub(
    This: *mut core::ffi::c_void,
    pFormatetc: *mut core::ffi::c_void,
    pStgmed: *mut core::ffi::c_void,
) -> HRESULT {

    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr(), This, pFormatetc, pStgmed);
    IAdviseSink_OnDataChange(This, pFormatetc, pStgmed);
    0
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnViewChange_Proxy(
    This: *mut IAdviseSink,
    dwAspect: DWORD,
    lindex: LONG,
) {

    TRACE!(b"(%p)->(%d, %d)\n\0".as_ptr() as *const i8, This, dwAspect, lindex);
    IAdviseSink_RemoteOnViewChange_Proxy(This, dwAspect, lindex);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnViewChange_Stub(
    This: *mut IAdviseSink,
    dwAspect: DWORD,
    lindex: LONG,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%d, %d)\n\0".as_ptr() as *const i8,
        This,
        dwAspect,
        lindex,
    );
    IAdviseSink_OnViewChange(This, dwAspect, lindex);
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnRename_Proxy(
    This: *mut IAdviseSink,
    pmk: *mut IMoniker,
) {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink_RemoteOnRename_Proxy(This, pmk);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnRename_Stub(
    This: *mut IAdviseSink,
    pmk: *mut IMoniker,
) -> HRESULT {

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink_OnRename(This, pmk);
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnSave_Proxy(
    This: *mut IAdviseSink
) {

    TRACE!(b"(%p)\n\0".as_ptr(), This);
    IAdviseSink_RemoteOnSave_Proxy(This);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnSave_Stub(
    This: *mut IAdviseSink
) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_OnSave(This);
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnClose_Proxy(
    This: *mut IAdviseSink,
) {

    TRACE!(b"(%p)\n\0".as_ptr(), This);
    IAdviseSink_RemoteOnClose_Proxy(This);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink_OnClose_Stub(
    This: *mut IAdviseSink
) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_OnClose(This);
    0x00000000 as HRESULT
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink2_OnLinkSrcChange_Proxy(
    This: *mut IAdviseSink2,
    pmk: *mut IMoniker,
) {

    TRACE!(b"(%p)->(%p)\n\0".as_ptr(), This, pmk);
    IAdviseSink2_RemoteOnLinkSrcChange_Proxy(This, pmk);
}
#[no_mangle]
pub unsafe extern "C" fn IAdviseSink2_OnLinkSrcChange_Stub(
    This: *mut IAdviseSink2,
    pmk: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink2_OnLinkSrcChange(This, pmk);
    S_OK
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_GetData_Proxy(
    This: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {

    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8, This, pformatetcIn, pmedium);
    IDataObject_RemoteGetData_Proxy(This, pformatetcIn, pmedium)
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_GetData_Stub(
    This: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pRemoteMedium: *mut STGMEDIUM,
) -> HRESULT {

    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8, This, pformatetcIn, pRemoteMedium);
    IDataObject_GetData(This, pformatetcIn, pRemoteMedium)
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_GetDataHere_Proxy(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {

    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr(),
        iface,
        fmt,
        med,
    );

    if ((*med).tymed & (TYMED_HGLOBAL | TYMED_FILE | TYMED_ISTREAM | TYMED_ISTORAGE)) == 0 {
        return DV_E_TYMED;
    }
    if (*med).tymed != (*fmt).tymed {
        return DV_E_TYMED;
    }

    let release = (*med).pUnkForRelease;
    (*med).pUnkForRelease = core::ptr::null_mut();

    let mut stg: *mut IStorage = core::ptr::null_mut();
    if (*med).tymed == TYMED_ISTREAM || (*med).tymed == TYMED_ISTORAGE {
        stg = (*med).u.pstg;
        if !stg.is_null() {
            IStorage_AddRef(stg);
        }
    }

    let hr = IDataObject_RemoteGetDataHere_Proxy(iface, fmt, med);

    (*med).pUnkForRelease = release;
    if !stg.is_null() {
        if !(*med).u.pstg.is_null() {
            IStorage_Release((*med).u.pstg);
        }
        (*med).u.pstg = stg;
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_GetDataHere_Stub(
    This: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pRemoteMedium: *mut STGMEDIUM
) -> HRESULT {
    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8, This, pformatetc, pRemoteMedium);
    IDataObject_GetDataHere(This, pformatetc, pRemoteMedium)
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_SetData_Proxy(
    This: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IDataObject_SetData_Stub(
    This: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut FLAG_STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as PCHAR);
    return E_NOTIMPL;
}
#[no_mangle]
pub unsafe extern "C" fn IOleInPlaceActiveObject_TranslateAccelerator_Proxy(
    This: *mut IOleInPlaceActiveObject,
    lpmsg: LPMSG,
) -> HRESULT {

    TRACE!(b"(%p %p)\n\0".as_ptr(), This, lpmsg);
    IOleInPlaceActiveObject_RemoteTranslateAccelerator_Proxy(This)
}
#[no_mangle]
pub unsafe extern "C" fn IOleInPlaceActiveObject_TranslateAccelerator_Stub(
    This: *mut IOleInPlaceActiveObject,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    return S_FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn IOleInPlaceActiveObject_ResizeBorder_Proxy(
    This: *mut IOleInPlaceActiveObject,
    prcBorder: LPCRECT,
    pUIWindow: *mut IOleInPlaceUIWindow,
    fFrameWindow: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IOleInPlaceActiveObject_ResizeBorder_Stub(
    This: *mut IOleInPlaceActiveObject,
    prcBorder: LPCRECT,
    riid: REFIID,
    pUIWindow: *mut IOleInPlaceUIWindow,
    fFrameWindow: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IOleCache2_UpdateCache_Proxy(
    This: *mut IOleCache2,
    pDataObject: LPDATAOBJECT,
    grfUpdf: DWORD,
    pReserved: LPVOID,
) -> HRESULT {

    TRACE!(
        b"(%p, %p, 0x%08x, %p)\n\0".as_ptr() as *const i8,
        This,
        pDataObject,
        grfUpdf,
        pReserved,
    );
    IOleCache2_RemoteUpdateCache_Proxy(This, pDataObject, grfUpdf, pReserved as LONG_PTR)
}
#[no_mangle]
pub unsafe extern "C" fn IOleCache2_UpdateCache_Stub(
    This: *mut IOleCache2,
    pDataObject: LPDATAOBJECT,
    grfUpdf: DWORD,
    pReserved: LONG_PTR,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, 0x%08x, %li)\n\0".as_ptr() as PCHAR,
        This,
        pDataObject,
        grfUpdf,
        pReserved,
    );
    IOleCache2_UpdateCache(This, pDataObject, grfUpdf, pReserved as LPVOID)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumOLEVERB_Next_Proxy(
    This: *mut IEnumOLEVERB,
    celt: ULONG,
    rgelt: LPOLEVERB,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr(), This, celt, rgelt, pceltFetched);
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumOLEVERB_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}
#[no_mangle]
pub unsafe extern "C" fn IEnumOLEVERB_Next_Stub(
    This: *mut IEnumOLEVERB,
    celt: ULONG,
    rgelt: LPOLEVERB,
    pceltFetched: *mut ULONG,
) -> HRESULT {

    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumOLEVERB_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_Draw_Proxy(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut core::ffi::c_void,
    ptd: *mut DVTARGETDEVICE,
    hdcTargetDev: HDC,
    hdcDraw: HDC,
    lprcBounds: LPCRECTL,
    lprcWBounds: LPCRECTL,
    pfnContinue: Option<unsafe extern "system" fn(ULONG_PTR) -> BOOL>,
    dwContinue: ULONG_PTR,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_Draw_Stub(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: ULONG_PTR,
    ptd: *mut DVTARGETDEVICE,
    hdcTargetDev: ULONG_PTR,
    hdcDraw: ULONG_PTR,
    lprcBounds: LPCRECTL,
    lprcWBounds: LPCRECTL,
    pContinue: *mut IContinue,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as PCHAR);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_GetColorSet_Proxy(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut c_void,
    ptd: *mut DVTARGETDEVICE,
    hicTargetDev: HDC,
    ppColorSet: *mut *mut LOGPALETTE,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as PCHAR);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_GetColorSet_Stub(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: ULONG_PTR,
    ptd: *mut DVTARGETDEVICE,
    hicTargetDev: ULONG_PTR,
    ppColorSet: *mut *mut LOGPALETTE,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as LPCSTR);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_Freeze_Proxy(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut core::ffi::c_void,
    pdwFreeze: *mut DWORD,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_Freeze_Stub(
    This: *mut IViewObject,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: ULONG_PTR,
    pdwFreeze: *mut DWORD,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_GetAdvise_Proxy(
    This: *mut IViewObject,
    pAspects: *mut DWORD,
    pAdvf: *mut DWORD,
    ppAdvSink: *mut *mut IAdviseSink,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn IViewObject_GetAdvise_Stub(
    This: *mut IViewObject,
    pAspects: *mut DWORD,
    pAdvf: *mut DWORD,
    ppAdvSink: *mut *mut IAdviseSink,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
