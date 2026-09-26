use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CRYPT_DEShash(dst: *mut core::ffi::c_uchar, key: *const core::ffi::c_uchar, src: *const core::ffi::c_uchar) -> *mut core::ffi::c_uchar;
    fn CRYPT_DESunhash(dst: *mut core::ffi::c_uchar, key: *const core::ffi::c_uchar, src: *const core::ffi::c_uchar) -> *mut core::ffi::c_uchar;
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
}
extern "system" {
    fn QueryPerformanceCounter(lpPerformanceCount: *mut LARGE_INTEGER) -> BOOLEAN;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
}

// ---- file-scope preamble (translated) ----
pub static CRYPT_LMhash_Magic: [u8; 8] =
    [b'K', b'G', b'S', b'!', b'@', b'#', b'$', b'%'];

pub static DefaultSessionKey: [u8; 16] =
    [b'D', b'e', b'f', b'S', b'e', b's', b's', b'i', b'o', b'n', b'K', b'e', b'y', b'!', b'@', b'#'];

pub static mut KsecDeviceHandle: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn SystemFunction001(
    data: *const BYTE,
    key: *const BYTE,
    output: LPBYTE,
) -> NTSTATUS {

    if data.is_null() || output.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    CRYPT_DEShash(output, key, data);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction002(
    data: *const BYTE,
    key: *const BYTE,
    output: LPBYTE,
) -> NTSTATUS {
    if data.is_null() || output.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    CRYPT_DESunhash(output, key, data);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction003(key: *const BYTE, output: LPBYTE) -> NTSTATUS {
    if output.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    CRYPT_DEShash(output, key, CRYPT_LMhash_Magic.as_ptr());
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction004(
    in_: *const ustring,
    key: *const ustring,
    out: *mut ustring,
) -> NTSTATUS {
    #[repr(C)]
    union Data {
        uc: [core::ffi::c_uchar; 8],
        ui: [core::ffi::c_uint; 2],
    }
    let mut data = Data { uc: [0; 8] };
    let mut deskey: [core::ffi::c_uchar; 7] = [0; 7];
    let crypt_len: core::ffi::c_uint;
    let mut ofs: core::ffi::c_uint;

    if (*key).Length <= 0 {
        return STATUS_INVALID_PARAMETER_2;
    }

    crypt_len = ((*in_).Length + 7) & !7;
    if (*out).MaximumLength < (crypt_len + 8) {
        (*out).Length = crypt_len + 8;
        return STATUS_BUFFER_TOO_SMALL;
    }

    data.ui[0] = (*in_).Length;
    data.ui[1] = 1;

    if ((*key).Length as usize) < core::mem::size_of_val(&deskey) {
        memset(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&deskey),
        );
        memcpy(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            (*key).Buffer as *const core::ffi::c_void,
            (*key).Length as usize,
        );
    } else {
        memcpy(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            (*key).Buffer as *const core::ffi::c_void,
            core::mem::size_of_val(&deskey),
        );
    }

    CRYPT_DEShash((*out).Buffer, deskey.as_mut_ptr(), data.uc.as_mut_ptr());

    ofs = 0;
    while ofs < (crypt_len - 8) {
        CRYPT_DEShash(
            (*out).Buffer.add(8 + ofs as usize),
            deskey.as_mut_ptr(),
            (*in_).Buffer.add(ofs as usize),
        );
        ofs += 8;
    }

    memset(
        data.uc.as_mut_ptr() as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&data.uc),
    );
    memcpy(
        data.uc.as_mut_ptr() as *mut core::ffi::c_void,
        (*in_).Buffer.add(ofs as usize) as *const core::ffi::c_void,
        ((*in_).Length + 8 - crypt_len) as usize,
    );
    CRYPT_DEShash(
        (*out).Buffer.add(8 + ofs as usize),
        deskey.as_mut_ptr(),
        data.uc.as_mut_ptr(),
    );

    (*out).Length = crypt_len + 8;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction005(
    r#in: *const ustring,
    key: *const ustring,
    out: *mut ustring,
) -> NTSTATUS {
    #[repr(C)]
    union Data {
        uc: [core::ffi::c_uchar; 8],
        ui: [core::ffi::c_uint; 2],
    }
    let mut data: Data = Data { uc: [0; 8] };
    let mut deskey: [core::ffi::c_uchar; 7] = [0; 7];
    let mut ofs: core::ffi::c_uint;
    let crypt_len: core::ffi::c_uint;

    if (*key).Length <= 0 {
        return STATUS_INVALID_PARAMETER_2;
    }

    if ((*key).Length as usize) < core::mem::size_of_val(&deskey) {
        memset(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&deskey),
        );
        memcpy(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            (*key).Buffer as *const core::ffi::c_void,
            (*key).Length as usize,
        );
    } else {
        memcpy(
            deskey.as_mut_ptr() as *mut core::ffi::c_void,
            (*key).Buffer as *const core::ffi::c_void,
            core::mem::size_of_val(&deskey),
        );
    }

    CRYPT_DESunhash(
        data.uc.as_mut_ptr(),
        deskey.as_ptr(),
        (*r#in).Buffer,
    );

    if data.ui[1] != 1 {
        return STATUS_UNKNOWN_REVISION;
    }

    crypt_len = data.ui[0];
    if crypt_len > (*out).MaximumLength as core::ffi::c_uint {
        (*out).Length = crypt_len as _;
        return STATUS_BUFFER_TOO_SMALL;
    }

    ofs = 0;
    while (ofs + 8) < crypt_len {
        CRYPT_DESunhash(
            (*out).Buffer.add(ofs as usize),
            deskey.as_ptr(),
            (*r#in).Buffer.add(ofs as usize + 8),
        );
        ofs += 8;
    }

    if ofs < crypt_len {
        CRYPT_DESunhash(
            data.uc.as_mut_ptr(),
            deskey.as_ptr(),
            (*r#in).Buffer.add(ofs as usize + 8),
        );
        memcpy(
            (*out).Buffer.add(ofs as usize) as *mut core::ffi::c_void,
            data.uc.as_ptr() as *const core::ffi::c_void,
            (crypt_len - ofs) as usize,
        );
    }

    (*out).Length = crypt_len as _;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction007(
    string: *const _UNICODE_STRING,
    hash: LPBYTE,
) -> NTSTATUS {
    let mut ctx: MD4_CTX = core::mem::zeroed();

    MD4Init(&mut ctx);
    MD4Update(
        &mut ctx,
        (*string).Buffer as *const BYTE,
        (*string).Length as u32,
    );
    MD4Final(&mut ctx);
    memcpy(
        hash as *mut core::ffi::c_void,
        ctx.digest.as_ptr() as *const core::ffi::c_void,
        0x10,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction008(
    challenge: *const BYTE,
    hash: *const BYTE,
    response: LPBYTE,
) -> NTSTATUS {
    let mut key: [BYTE; 7 * 3] = [0; 7 * 3];

    if challenge.is_null() || response.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    memset(key.as_mut_ptr() as LPVOID, 0, core::mem::size_of_val(&key));
    memcpy(key.as_mut_ptr() as LPVOID, hash as LPCVOID, 0x10);

    CRYPT_DEShash(response, key.as_ptr(), challenge);
    CRYPT_DEShash(response.add(8), key.as_ptr().add(7), challenge);
    CRYPT_DEShash(response.add(16), key.as_ptr().add(14), challenge);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction009(
    challenge: *const BYTE,
    hash: *const BYTE,
    response: LPBYTE,
) -> NTSTATUS {
    SystemFunction008(challenge, hash, response)
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction010(
    unknown: LPVOID,
    data: *const BYTE,
    hash: LPBYTE,
) -> NTSTATUS {
    let mut ctx: MD4_CTX = core::mem::zeroed();

    MD4Init(&mut ctx);
    MD4Update(&mut ctx, data, 0x10);
    MD4Final(&mut ctx);
    memcpy(
        hash as *mut core::ffi::c_void,
        ctx.digest.as_ptr() as *const core::ffi::c_void,
        0x10,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction012(
    in_: *const u8,
    key: *const u8,
    out: LPBYTE,
) -> i32 {
    if in_.is_null() || out.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    CRYPT_DEShash(out, key, in_);
    CRYPT_DEShash(out.add(8), key.add(7), in_.add(8));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction013(in_: *const BYTE, key: *const BYTE, out: LPBYTE) -> NTSTATUS {
    if in_.is_null() || out.is_null() {
        return STATUS_UNSUCCESSFUL;
    }
    CRYPT_DESunhash(out, key, in_);
    CRYPT_DESunhash(out.add(8), key.add(7), in_.add(8));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction024(
    in_: *const BYTE,
    key: *const BYTE,
    out: LPBYTE,
) -> NTSTATUS {
    let mut deskey: [BYTE; 0x10] = [0; 0x10];

    core::ptr::copy_nonoverlapping(key, deskey.as_mut_ptr(), 4);
    core::ptr::copy_nonoverlapping(key, deskey.as_mut_ptr().add(4), 4);
    core::ptr::copy_nonoverlapping(key, deskey.as_mut_ptr().add(8), 4);
    core::ptr::copy_nonoverlapping(key, deskey.as_mut_ptr().add(12), 4);

    CRYPT_DEShash(out, deskey.as_mut_ptr(), in_);
    CRYPT_DEShash(out.add(8), deskey.as_mut_ptr().add(7), in_.add(8));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction025(
    in_: *const BYTE,
    key: *const BYTE,
    out: LPBYTE,
) -> NTSTATUS {
    let mut deskey: [BYTE; 0x10] = [0; 0x10];

    memcpy(deskey.as_mut_ptr() as LPVOID, key as LPCVOID, 4);
    memcpy(deskey.as_mut_ptr().add(4) as LPVOID, key as LPCVOID, 4);
    memcpy(deskey.as_mut_ptr().add(8) as LPVOID, key as LPCVOID, 4);
    memcpy(deskey.as_mut_ptr().add(12) as LPVOID, key as LPCVOID, 4);

    CRYPT_DESunhash(out, deskey.as_ptr(), in_);
    CRYPT_DESunhash(out.add(8), deskey.as_ptr().add(7), in_.add(8));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction028(
    ContextHandle: *mut core::ffi::c_void,
    SessionKey: LPBYTE,
) -> i32 {
    core::ptr::copy_nonoverlapping( // nolint: silent-failure (byte-array pointee — count==bytes)
        DefaultSessionKey.as_ptr(),
        SessionKey,
        core::mem::size_of_val(&DefaultSessionKey),
    );
    STATUS_LOCAL_USER_SESSION_KEY
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction029(a: INT, b: INT) -> INT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    29
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction030(b1: LPCVOID, b2: LPCVOID) -> BOOL {
    if memcmp(b1, b2, 0x10) == 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction032(data: *mut ustring, key: *const ustring) -> NTSTATUS {
    let mut a4i: RC4_CONTEXT = core::mem::zeroed();

    rc4_init(&mut a4i, (*key).Buffer, (*key).Length);
    rc4_crypt(&mut a4i, (*data).Buffer, (*data).Length);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction033(a: INT, b: INT) -> INT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    33
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction034(a: INT, b: INT) -> INT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    34
}
#[no_mangle]
pub unsafe extern "system" fn SystemFunction035(lpszDllFilePath: LPCSTR) -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction036(pbBuffer: PVOID, mut dwLen: ULONG) -> BOOLEAN {
    static mut uCounter: ULONG = 17;

    if pbBuffer.is_null() || dwLen == 0 {
        return 1u8;
    }

    let mut time: LARGE_INTEGER = core::mem::zeroed();
    QueryPerformanceCounter(&mut time);

    let counter = uCounter;
    uCounter = uCounter.wrapping_add(1);

    let mut dwSeed: ULONG = (time.__bindgen_anon_1.LowPart as ULONG)
        ^ (time.__bindgen_anon_1.HighPart as ULONG)
        ^ RtlUlongByteSwap(counter);

    let mut pBuffer: PBYTE = pbBuffer as PBYTE;

    loop {
        let uPseudoRandom: ULONG = RtlRandom(&mut dwSeed);

        loop {
            *pBuffer = ((uPseudoRandom >> (8 * (dwLen % 3))) & 0xFF) as u8;
            pBuffer = pBuffer.add(1);

            dwLen -= 1;
            if dwLen % 3 == 0 {
                break;
            }
        }

        if dwLen == 0 {
            break;
        }
    }

    1u8
}

unsafe fn KsecOpenDevice() -> NTSTATUS {
    let device_name_buffer: [u16; 15] = [
        0x005c, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005c,
        0x004b, 0x0073, 0x0065, 0x0063, 0x0044, 0x0044, 0x0000,
    ];
    let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
    DeviceName.Length = (14 * 2) as u16;
    DeviceName.MaximumLength = (15 * 2) as u16;
    DeviceName.Buffer = device_name_buffer.as_ptr() as *mut u16;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceHandle: HANDLE = core::ptr::null_mut();
    let Status: NTSTATUS;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as u32;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(DeviceName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        core::ptr::addr_of_mut!(DeviceHandle),
        FILE_READ_DATA | SYNCHRONIZE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if !InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(KsecDeviceHandle) as *mut *mut core::ffi::c_void,
        DeviceHandle,
        core::ptr::null_mut(),
    )
    .is_null()
    {
        NtClose(DeviceHandle);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CloseKsecDdHandle() {
    if KsecDeviceHandle != core::ptr::null_mut() {
        CloseHandle(KsecDeviceHandle);
        KsecDeviceHandle = core::ptr::null_mut();
    }
}

unsafe fn KsecDeviceIoControl(
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: SIZE_T,
    OutputBuffer: PVOID,
    OutputBufferLength: SIZE_T,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if KsecDeviceHandle == core::ptr::null_mut() {
        Status = KsecOpenDevice();
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Status = NtDeviceIoControlFile(
        KsecDeviceHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        IoControlCode,
        InputBuffer,
        InputBufferLength as ULONG,
        OutputBuffer,
        OutputBufferLength as ULONG,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction040(
    Memory: PVOID,
    MemoryLength: ULONG,
    OptionFlags: ULONG,
) -> NTSTATUS {
    let IoControlCode: ULONG;

    if OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS {
        IoControlCode = IOCTL_KSEC_ENCRYPT_SAME_PROCESS;
    } else if OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS {
        IoControlCode = IOCTL_KSEC_ENCRYPT_CROSS_PROCESS;
    } else if OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON {
        IoControlCode = IOCTL_KSEC_ENCRYPT_SAME_LOGON;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    KsecDeviceIoControl(
        IoControlCode,
        Memory,
        MemoryLength as SIZE_T,
        Memory,
        MemoryLength as SIZE_T,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SystemFunction041(
    Memory: PVOID,
    MemoryLength: ULONG,
    OptionFlags: ULONG,
) -> NTSTATUS {
    let IoControlCode: ULONG;

    if OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS {
        IoControlCode = IOCTL_KSEC_DECRYPT_SAME_PROCESS;
    } else if OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS {
        IoControlCode = IOCTL_KSEC_DECRYPT_CROSS_PROCESS;
    } else if OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON {
        IoControlCode = IOCTL_KSEC_DECRYPT_SAME_LOGON;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    KsecDeviceIoControl(
        IoControlCode,
        Memory,
        MemoryLength as SIZE_T,
        Memory,
        MemoryLength as SIZE_T,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/advapi32/misc/sysfunc.c
//  * PURPOSE:         advapi32.dll system functions (undocumented)
//  * PROGRAMMER:      Emanuele Aliberti
//  * UPDATE HISTORY:
//  *  19990413 EA created
//  *  19990415 EA
//  *  20080424 Ported from WINE
//  */
//
// #include <advapi32.h>
// #include <ntsecapi.h>
// #include <ksecioctl.h>
// #include <md4.h>
// #include <md5.h>
// #include <rc4.h>
//
// static const unsigned char CRYPT_LMhash_Magic[8] =
//     { 'K', 'G', 'S', '!', '@', '#', '$', '%' };
// static const unsigned char DefaultSessionKey[16] =
//     {'D', 'e', 'f', 'S', 'e', 's', 's', 'i', 'o', 'n', 'K', 'e', 'y', '!', '@', '#'};
//
// /******************************************************************************
//  * SystemFunction001  [ADVAPI32.@]
//  *
//  * Encrypts a single block of data using DES
//  *
//  * PARAMS
//  *   data    [I] data to encrypt    (8 bytes)
//  *   key     [I] key data           (7 bytes)
//  *   output  [O] the encrypted data (8 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction001(const BYTE *data, const BYTE *key, LPBYTE output)
// {
//     if (!data || !output)
//         return STATUS_UNSUCCESSFUL;
//     CRYPT_DEShash(output, key, data);
//     return STATUS_SUCCESS;
// }
//
//
// /******************************************************************************
//  * SystemFunction002  [ADVAPI32.@]
//  *
//  * Decrypts a single block of data using DES
//  *
//  * PARAMS
//  *   data    [I] data to decrypt    (8 bytes)
//  *   key     [I] key data           (7 bytes)
//  *   output  [O] the decrypted data (8 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction002(const BYTE *data, const BYTE *key, LPBYTE output)
// {
//     if (!data || !output)
//         return STATUS_UNSUCCESSFUL;
//     CRYPT_DESunhash(output, key, data);
//     return STATUS_SUCCESS;
// }
//
//
// /******************************************************************************
//  * SystemFunction003  [ADVAPI32.@]
//  *
//  * Hashes a key using DES and a fixed datablock
//  *
//  * PARAMS
//  *   key     [I] key data    (7 bytes)
//  *   output  [O] hashed key  (8 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction003(const BYTE *key, LPBYTE output)
// {
//     if (!output)
//         return STATUS_UNSUCCESSFUL;
//     CRYPT_DEShash(output, key, CRYPT_LMhash_Magic);
//     return STATUS_SUCCESS;
// }
//
//
// /******************************************************************************
//  * SystemFunction004  [ADVAPI32.@]
//  *
//  * Encrypts a block of data with DES in ECB mode, preserving the length
//  *
//  * PARAMS
//  *   data    [I] data to encrypt
//  *   key     [I] key data (up to 7 bytes)
//  *   output  [O] buffer to receive encrypted data
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_BUFFER_TOO_SMALL     if the output buffer is too small
//  *  Failure: STATUS_INVALID_PARAMETER_2  if the key is zero length
//  *
//  * NOTES
//  *  Encrypt buffer size should be input size rounded up to 8 bytes
//  *   plus an extra 8 bytes.
//  */
// NTSTATUS
// WINAPI SystemFunction004(const struct ustring *in,
//                                   const struct ustring *key,
//                                   struct ustring *out)
// {
//     union {
//         unsigned char uc[8];
//           unsigned int  ui[2];
//     } data;
//     unsigned char deskey[7];
//     unsigned int crypt_len, ofs;
//
//     if (key->Length<=0)
//         return STATUS_INVALID_PARAMETER_2;
//
//     crypt_len = ((in->Length+7)&~7);
//     if (out->MaximumLength < (crypt_len+8))
//     {
//         out->Length = crypt_len + 8;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     data.ui[0] = in->Length;
//     data.ui[1] = 1;
//
//     if (key->Length<sizeof deskey)
//     {
//         memset(deskey, 0, sizeof deskey);
//         memcpy(deskey, key->Buffer, key->Length);
//     }
//     else
//         memcpy(deskey, key->Buffer, sizeof deskey);
//
//     CRYPT_DEShash(out->Buffer, deskey, data.uc);
//
//     for(ofs=0; ofs<(crypt_len-8); ofs+=8)
//         CRYPT_DEShash(out->Buffer+8+ofs, deskey, in->Buffer+ofs);
//
//     memset(data.uc, 0, sizeof data.uc);
//     memcpy(data.uc, in->Buffer+ofs, in->Length +8-crypt_len);
//     CRYPT_DEShash(out->Buffer+8+ofs, deskey, data.uc);
//
//     out->Length = crypt_len+8;
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction005  [ADVAPI32.@]
//  *
//  * Decrypts a block of data with DES in ECB mode
//  *
//  * PARAMS
//  *   data    [I] data to decrypt
//  *   key     [I] key data (up to 7 bytes)
//  *   output  [O] buffer to receive decrypted data
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_BUFFER_TOO_SMALL     if the output buffer is too small
//  *  Failure: STATUS_INVALID_PARAMETER_2  if the key is zero length
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction005(const struct ustring *in,
//                          const struct ustring *key,
//                          struct ustring *out)
// {
//     union {
//         unsigned char uc[8];
//         unsigned int  ui[2];
//     } data;
//     unsigned char deskey[7];
//     unsigned int ofs, crypt_len;
//
//     if (key->Length<=0)
//         return STATUS_INVALID_PARAMETER_2;
//
//     if (key->Length<sizeof deskey)
//     {
//         memset(deskey, 0, sizeof deskey);
//         memcpy(deskey, key->Buffer, key->Length);
//     }
//     else
//         memcpy(deskey, key->Buffer, sizeof deskey);
//
//     CRYPT_DESunhash(data.uc, deskey, in->Buffer);
//
//     if (data.ui[1] != 1)
//         return STATUS_UNKNOWN_REVISION;
//
//     crypt_len = data.ui[0];
//     if (crypt_len > out->MaximumLength)
//     {
//         out->Length = crypt_len;
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     for (ofs=0; (ofs+8)<crypt_len; ofs+=8)
//         CRYPT_DESunhash(out->Buffer+ofs, deskey, in->Buffer+ofs+8);
//
//     if (ofs<crypt_len)
//     {
//         CRYPT_DESunhash(data.uc, deskey, in->Buffer+ofs+8);
//         memcpy(out->Buffer+ofs, data.uc, crypt_len-ofs);
//     }
//
//     out->Length = crypt_len;
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction007  [ADVAPI32.@]
//  *
//  * MD4 hash a unicode string
//  *
//  * PARAMS
//  *   string  [I] the string to hash
//  *   output  [O] the md4 hash of the string (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction007(const UNICODE_STRING *string, LPBYTE hash)
// {
//     MD4_CTX ctx;
//
//     MD4Init( &ctx );
//     MD4Update( &ctx, (const BYTE *)string->Buffer, string->Length );
//     MD4Final( &ctx );
//     memcpy( hash, ctx.digest, 0x10 );
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction008  [ADVAPI32.@]
//  *
//  * Creates a LM response from a challenge and a password hash
//  *
//  * PARAMS
//  *   challenge  [I] Challenge from authentication server
//  *   hash       [I] NTLM hash (from SystemFunction006)
//  *   response   [O] response to send back to the server
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  * NOTES
//  *  see http://davenport.sourceforge.net/ntlm.html#theLmResponse
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction008(const BYTE *challenge, const BYTE *hash, LPBYTE response)
// {
//     BYTE key[7*3];
//
//     if (!challenge || !response)
//         return STATUS_UNSUCCESSFUL;
//
//     memset(key, 0, sizeof key);
//     memcpy(key, hash, 0x10);
//
//     CRYPT_DEShash(response, key, challenge);
//     CRYPT_DEShash(response+8, key+7, challenge);
//     CRYPT_DEShash(response+16, key+14, challenge);
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction009  [ADVAPI32.@]
//  *
//  * Seems to do the same as SystemFunction008...
//  */
// NTSTATUS
// WINAPI SystemFunction009(const BYTE *challenge, const BYTE *hash, LPBYTE response)
// {
//     return SystemFunction008(challenge, hash, response);
// }
//
// /******************************************************************************
//  * SystemFunction010  [ADVAPI32.@]
//  * SystemFunction011  [ADVAPI32.@]
//  *
//  * MD4 hashes 16 bytes of data
//  *
//  * PARAMS
//  *   unknown []  seems to have no effect on the output
//  *   data    [I] pointer to data to hash (16 bytes)
//  *   output  [O] the md4 hash of the data (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  */
// NTSTATUS
// WINAPI SystemFunction010(LPVOID unknown, const BYTE *data, LPBYTE hash)
// {
//     MD4_CTX ctx;
//
//     MD4Init( &ctx );
//     MD4Update( &ctx, data, 0x10 );
//     MD4Final( &ctx );
//     memcpy( hash, ctx.digest, 0x10 );
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction012  [ADVAPI32.@]
//  * SystemFunction014  [ADVAPI32.@]
//  * SystemFunction016  [ADVAPI32.@]
//  * SystemFunction018  [ADVAPI32.@]
//  * SystemFunction020  [ADVAPI32.@]
//  * SystemFunction022  [ADVAPI32.@]
//  *
//  * Encrypts two DES blocks with two keys
//  *
//  * PARAMS
//  *   data    [I] data to encrypt (16 bytes)
//  *   key     [I] key data (two lots of 7 bytes)
//  *   output  [O] buffer to receive encrypted data (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL  if the input or output buffer is NULL
//  */
// NTSTATUS
// WINAPI SystemFunction012(const BYTE *in, const BYTE *key, LPBYTE out)
// {
//     if (!in || !out)
//         return STATUS_UNSUCCESSFUL;
//
//     CRYPT_DEShash(out, key, in);
//     CRYPT_DEShash(out+8, key+7, in+8);
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction013  [ADVAPI32.@]
//  * SystemFunction015  [ADVAPI32.@]
//  * SystemFunction017  [ADVAPI32.@]
//  * SystemFunction019  [ADVAPI32.@]
//  * SystemFunction021  [ADVAPI32.@]
//  * SystemFunction023  [ADVAPI32.@]
//  *
//  * Decrypts two DES blocks with two keys
//  *
//  * PARAMS
//  *   data    [I] data to decrypt (16 bytes)
//  *   key     [I] key data (two lots of 7 bytes)
//  *   output  [O] buffer to receive decrypted data (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL  if the input or output buffer is NULL
//  */
// NTSTATUS
// WINAPI SystemFunction013(const BYTE *in, const BYTE *key, LPBYTE out)
// {
//     if (!in || !out)
//         return STATUS_UNSUCCESSFUL;
//     CRYPT_DESunhash(out, key, in);
//     CRYPT_DESunhash(out+8, key+7, in+8);
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction024  [ADVAPI32.@]
//  *
//  * Encrypts two DES blocks with a 32 bit key...
//  *
//  * PARAMS
//  *   data    [I] data to encrypt (16 bytes)
//  *   key     [I] key data (4 bytes)
//  *   output  [O] buffer to receive encrypted data (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  */
// NTSTATUS
// WINAPI SystemFunction024(const BYTE *in, const BYTE *key, LPBYTE out)
// {
//     BYTE deskey[0x10];
//
//     memcpy(deskey, key, 4);
//     memcpy(deskey+4, key, 4);
//     memcpy(deskey+8, key, 4);
//     memcpy(deskey+12, key, 4);
//
//     CRYPT_DEShash(out, deskey, in);
//     CRYPT_DEShash(out+8, deskey+7, in+8);
//
//     return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction025  [ADVAPI32.@]
//  *
//  * Decrypts two DES blocks with a 32 bit key...
//  *
//  * PARAMS
//  *   data    [I] data to encrypt (16 bytes)
//  *   key     [I] key data (4 bytes)
//  *   output  [O] buffer to receive encrypted data (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  */
// NTSTATUS
// WINAPI SystemFunction025(const BYTE *in, const BYTE *key, LPBYTE out)
// {
//     BYTE deskey[0x10];
//
//     memcpy(deskey, key, 4);
//     memcpy(deskey+4, key, 4);
//     memcpy(deskey+8, key, 4);
//     memcpy(deskey+12, key, 4);
//
//     CRYPT_DESunhash(out, deskey, in);
//     CRYPT_DESunhash(out+8, deskey+7, in+8);
//
//     return STATUS_SUCCESS;
// }
//
// /**********************************************************************
//  * SystemFunction028  [ADVAPI32.@]
//  *
//  * Retrieves an encryption session key...
//  *
//  * PARAMS
//  *   ContextHandle [I] RPC context handle
//  *   SessionKey    [O] buffer to receive the session key (16 bytes)
//  *
//  * RETURNS
//  *  Success: STATUS_LOCAL_USER_SESSION_KEY
//  *
//  * @unimplemented
//  */
// NTSTATUS
// WINAPI
// SystemFunction028(
//      PVOID ContextHandle,
//      LPBYTE SessionKey)
// {
//     /* HACK: Always return the default key */
//     memcpy(SessionKey, DefaultSessionKey, sizeof(DefaultSessionKey));
//     return STATUS_LOCAL_USER_SESSION_KEY;
//
// }
//
//
// /**********************************************************************
//  *
//  * @unimplemented
//  */
// INT
// WINAPI
// SystemFunction029(INT a, INT b)
// {
//     //I_RpcBindingIsClientLocal()
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return 29;
// }
//
//
// /******************************************************************************
//  * SystemFunction030   (ADVAPI32.@)
//  *
//  * Tests if two blocks of 16 bytes are equal
//  *
//  * PARAMS
//  *  b1,b2   [I] block of 16 bytes
//  *
//  * RETURNS
//  *  TRUE  if blocks are the same
//  *  FALSE if blocks are different
//  */
// BOOL
// WINAPI SystemFunction030(LPCVOID b1, LPCVOID b2)
// {
//     return !memcmp(b1, b2, 0x10);
// }
//
//
// /******************************************************************************
//  * SystemFunction032  [ADVAPI32.@]
//  *
//  * Encrypts a string data using ARC4
//  *
//  * PARAMS
//  *   data    [I/O] data to encrypt
//  *   key     [I] key data
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: STATUS_UNSUCCESSFUL
//  *
//  * NOTES
//  *  see http://web.it.kth.se/~rom/ntsec.html#crypto-strongavail
//  */
// NTSTATUS
// WINAPI SystemFunction032(struct ustring *data, const struct ustring *key)
// {
//     RC4_CONTEXT a4i;
//
//     rc4_init(&a4i, key->Buffer, key->Length);
//     rc4_crypt(&a4i, data->Buffer, data->Length);
//
//     return STATUS_SUCCESS;
// }
//
//
// /**********************************************************************
//  *
//  * @unimplemented
//  */
// INT
// WINAPI
// SystemFunction033(INT a, INT b)
// {
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return 33;
// }
//
// /**********************************************************************
//  *
//  * @unimplemented
//  */
// INT
// WINAPI
// SystemFunction034(INT a, INT b)
// {
//     //RpcBindingToStringBindingW
//     //I_RpcMapWin32Status
//     //RpcStringBindingParseW
//     //RpcStringFreeW
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return 34;
// }
//
//
// /******************************************************************************
//  * SystemFunction035   (ADVAPI32.@)
//  *
//  * Described here:
// http://disc.server.com/discussion.cgi?disc=148775;article=942;title=Coding%2FASM%2FSystem
//  *
//  * NOTES
//  *  Stub, always return TRUE.
//  */
// BOOL WINAPI SystemFunction035(LPCSTR lpszDllFilePath)
// {
//     //FIXME("%s: stub\n", debugstr_a(lpszDllFilePath));
//     return TRUE;
// }
//
// /******************************************************************************
//  * SystemFunction036   (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlGenRandom and declares it in ntsecapi.h
//  *
//  * PARAMS
//  *  pbBuffer [O] Pointer to memory to receive random bytes.
//  *  dwLen    [I] Number of random bytes to fetch.
//  *
//  * RETURNS
//  *  Always TRUE in my tests
//  */
// BOOLEAN
// WINAPI
// SystemFunction036(PVOID pbBuffer, ULONG dwLen)
// {
//     ////////////////////////////////////////////////////////////////
//     //////////////////// B I G   W A R N I N G  !!! ////////////////
//     // This function will output numbers based on the tick count. //
//     // It will NOT OUTPUT CRYPTOGRAPHIC-SAFE RANDOM NUMBERS !!!    //
//     ////////////////////////////////////////////////////////////////
//
//     DWORD dwSeed;
//     PBYTE pBuffer;
//     ULONG uPseudoRandom;
//     LARGE_INTEGER time;
//     static ULONG uCounter = 17;
//
//     if(!pbBuffer || !dwLen)
//     {
//         /* This function always returns TRUE, even if invalid parameters were passed. (verified under WinXP SP2) */
//         return TRUE;
//     }
//
//     /* Get the first seed from the performance counter */
//     QueryPerformanceCounter(&time);
//     dwSeed = time.LowPart ^ time.HighPart ^ RtlUlongByteSwap(uCounter++);
//
//     /* We will access the buffer bytewise */
//     pBuffer = (PBYTE)pbBuffer;
//
//     do
//     {
//         /* Use the pseudo random number generator RtlRandom, which outputs a 4-byte value and a new seed */
//         uPseudoRandom = RtlRandom(&dwSeed);
//
//         do
//         {
//             /* Get each byte from the pseudo random number and store it in the buffer */
//             *pBuffer = (BYTE)(uPseudoRandom >> 8 * (dwLen % 3) & 0xFF);
//             ++pBuffer;
//         } while(--dwLen % 3);
//     } while(dwLen);
//
//     return TRUE;
// }
//
// HANDLE KsecDeviceHandle;
//
// static
// NTSTATUS
// KsecOpenDevice()
// {
//     UNICODE_STRING DeviceName = RTL_CONSTANT_STRING(L"\\Device\\KsecDD");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE DeviceHandle;
//     NTSTATUS Status;
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DeviceName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenFile(&DeviceHandle,
//                         FILE_READ_DATA | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     if (InterlockedCompareExchangePointer(&KsecDeviceHandle, DeviceHandle, NULL) != NULL)
//     {
//         NtClose(DeviceHandle);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// CloseKsecDdHandle(VOID)
// {
//     /* Check if we already opened a handle to ksecdd */
//     if (KsecDeviceHandle != NULL)
//     {
//         /* Close it */
//         CloseHandle(KsecDeviceHandle);
//         KsecDeviceHandle = NULL;
//     }
// }
//
// static
// NTSTATUS
// KsecDeviceIoControl(
//     ULONG IoControlCode,
//     PVOID InputBuffer,
//     SIZE_T InputBufferLength,
//     PVOID OutputBuffer,
//     SIZE_T OutputBufferLength)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* Check if we already have a handle */
//     if (KsecDeviceHandle == NULL)
//     {
//         /* Try to open the device */
//         Status = KsecOpenDevice();
//         if (!NT_SUCCESS(Status))
//         {
//             //ERR("Failed to open handle to KsecDd driver!\n");
//             return Status;
//         }
//     }
//
//     /* Call the driver */
//     Status = NtDeviceIoControlFile(KsecDeviceHandle,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    IoControlCode,
//                                    InputBuffer,
//                                    InputBufferLength,
//                                    OutputBuffer,
//                                    OutputBufferLength);
//
//     return Status;
// }
//
// /*
//    These functions have nearly identical prototypes to CryptProtectMemory and CryptUnprotectMemory,
//    in crypt32.dll.
//  */
//
// /******************************************************************************
//  * SystemFunction040   (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlEncryptMemory and declares it in ntsecapi.h.
//  *
//  * PARAMS
//  *  memory [I/O] Pointer to memory to encrypt.
//  *  length [I] Length of region to encrypt in bytes.
//  *  flags  [I] Control whether other processes are able to decrypt the memory.
//  *    RTL_ENCRYPT_OPTION_SAME_PROCESS
//  *    RTL_ENCRYPT_OPTION_CROSS_PROCESS
//  *    RTL_ENCRYPT_OPTION_SAME_LOGON
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: NTSTATUS error code
//  *
//  * NOTES
//  *  length must be a multiple of RTL_ENCRYPT_MEMORY_SIZE.
//  *  If flags are specified when encrypting, the same flag value must be given
//  *  when decrypting the memory.
//  */
// NTSTATUS
// WINAPI
// SystemFunction040(
//      PVOID Memory,
//      ULONG MemoryLength,
//      ULONG OptionFlags)
// {
//     ULONG IoControlCode;
//
//     if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS)
//     {
//         IoControlCode = IOCTL_KSEC_ENCRYPT_SAME_PROCESS;
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS)
//     {
//         IoControlCode = IOCTL_KSEC_ENCRYPT_CROSS_PROCESS;
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON)
//     {
//         IoControlCode = IOCTL_KSEC_ENCRYPT_SAME_LOGON;
//     }
//     else
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     return KsecDeviceIoControl(IoControlCode, Memory, MemoryLength, Memory, MemoryLength);
// }
//
// /******************************************************************************
//  * SystemFunction041  (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlDecryptMemory and declares it in ntsecapi.h.
//  *
//  * PARAMS
//  *  memory [I/O] Pointer to memory to decrypt.
//  *  length [I] Length of region to decrypt in bytes.
//  *  flags  [I] Control whether other processes are able to decrypt the memory.
//  *    RTL_ENCRYPT_OPTION_SAME_PROCESS
//  *    RTL_ENCRYPT_OPTION_CROSS_PROCESS
//  *    RTL_ENCRYPT_OPTION_SAME_LOGON
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: NTSTATUS error code
//  *
//  * NOTES
//  *  length must be a multiple of RTL_ENCRYPT_MEMORY_SIZE.
//  *  If flags are specified when encrypting, the same flag value must be given
//  *  when decrypting the memory.
//  */
// NTSTATUS
// WINAPI
// SystemFunction041(
//      PVOID Memory,
//      ULONG MemoryLength,
//      ULONG OptionFlags)
// {
//     ULONG IoControlCode;
//
//     if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS)
//     {
//         IoControlCode = IOCTL_KSEC_DECRYPT_SAME_PROCESS;
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS)
//     {
//         IoControlCode = IOCTL_KSEC_DECRYPT_CROSS_PROCESS;
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON)
//     {
//         IoControlCode = IOCTL_KSEC_DECRYPT_SAME_LOGON;
//     }
//     else
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     return KsecDeviceIoControl(IoControlCode, Memory, MemoryLength, Memory, MemoryLength);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
