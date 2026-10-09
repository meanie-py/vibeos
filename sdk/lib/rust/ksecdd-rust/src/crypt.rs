use ntkrnl_sys::globals::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use crate::types::*;
use crate::glue::*;
use crate::glue::RtlSecureZeroMemory; // explicit: shadow ddk::* phantom (FORCEINLINE)
use crate::glue::RtlAssert; // explicit: shadow ddk::* phantom (the kernel export, via glue)
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

// cryptlib (tomcrypt/md5) primitives — cdecl, provided by the cryptlib rlib.
extern "C" {
    fn des3_setup(key: *mut UCHAR, keylen: ::core::ffi::c_int, num_rounds: ::core::ffi::c_int, skey: *mut DES3_KEY);
    fn des3_ecb_encrypt(pt: *const UCHAR, ct: *mut UCHAR, skey: PDES3_KEY);
    fn des3_ecb_decrypt(input: *const core::ffi::c_uchar, output: *mut core::ffi::c_uchar, key: PDES3_KEY);
    fn aes_setup(key: *mut UCHAR, keylen: i32, rounds: i32, skey: *mut AES_KEY);
    fn aes_ecb_encrypt( input: *const ::core::ffi::c_uchar, output: *mut ::core::ffi::c_uchar, key: *const AES_KEY, );
    fn aes_ecb_decrypt( input: *const core::ffi::c_uchar, output: *mut core::ffi::c_uchar, key: *mut AES_KEY, );
}
// NTAPI stdcall: KsecGatherEntropyData is crate-internal (random.rs);
// SeQueryAuthenticationIdToken is an ntoskrnl export. (RtlAssert -> glue no-op.)
extern "system" {
    fn KsecGatherEntropyData(EntropyData: *mut KSEC_ENTROPY_DATA) -> NTSTATUS;
    fn SeQueryAuthenticationIdToken(Token: PACCESS_TOKEN, AuthenticationId: *mut LUID) -> NTSTATUS;
    // MD5* are ReactOS crypt-lib NTAPI (stdcall on i386 -> _MD5Init@4), unlike the
    // cdecl tomcrypt aes_*/des3_* above.
    fn MD5Init(context: *mut MD5_CTX);
    fn MD5Update(context: *mut MD5_CTX, input: *const u8, len: u32);
    fn MD5Final(context: *mut MD5_CTX);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut KsecLoadTimeStartMd5s: [MD5_CTX; 2] = [MD5_CTX {
    i: [0; 2],
    buf: [0; 4],
    in_0: [0; 64],
    digest: [0; 16],
}; 2];

pub static mut KsecGlobalDes3Key: DES3_KEY = DES3_KEY {
    ek: [[0; 32]; 3],
    dk: [[0; 32]; 3],
};

pub static mut KsecGlobalAesKey: AES_KEY = AES_KEY {
    eK: [0; 64],
    dK: [0; 64],
    Nr: 0,
};

#[repr(C)]
pub struct _KSEC_PROCESS_DATA {
    pub Process: PEPROCESS,
    pub ProcessId: HANDLE,
    pub CreateTime: LONGLONG,
    pub DirectoryTableBase: ULONG_PTR,
}
pub type KSEC_PROCESS_DATA = _KSEC_PROCESS_DATA;
pub type PKSEC_PROCESS_DATA = *mut _KSEC_PROCESS_DATA;

#[repr(C)]
pub struct _KSEC_LOGON_DATA {
    pub LogonId: LUID,
}
pub type KSEC_LOGON_DATA = _KSEC_LOGON_DATA;
pub type PKSEC_LOGON_DATA = *mut _KSEC_LOGON_DATA;

// ksecdd.h:25-27 / ntsecapi.h:700: SAME_PROCESS 0, CROSS_PROCESS 1, SAME_LOGON 2.
const RTL_ENCRYPT_OPTION_SAME_LOGON: u32 = 2;

#[no_mangle]
pub unsafe extern "system" fn KsecInitializeEncryptionSupport() {

    let mut EntropyData: KSEC_ENTROPY_DATA = core::mem::zeroed();
    let mut Md5Context: MD5_CTX = core::mem::zeroed();
    let mut KeyDataBuffer: [UCHAR; 32] = [0; 32];

    KsecGatherEntropyData(core::ptr::addr_of_mut!(EntropyData));
    MD5Init(core::ptr::addr_of_mut!(Md5Context));
    MD5Update(
        core::ptr::addr_of_mut!(Md5Context),
        core::ptr::addr_of_mut!(EntropyData) as *const u8,
        core::mem::size_of::<KSEC_ENTROPY_DATA>() as u32,
    );
    KsecLoadTimeStartMd5s[0usize] = Md5Context;
    MD5Final(core::ptr::addr_of_mut!(Md5Context));
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of_mut!(Md5Context.digest) as *const u8,
        core::ptr::addr_of_mut!(KeyDataBuffer) as *mut u8,
        16,
    );

    KsecGatherEntropyData(core::ptr::addr_of_mut!(EntropyData));
    Md5Context = KsecLoadTimeStartMd5s[0usize];
    MD5Update(
        core::ptr::addr_of_mut!(Md5Context),
        core::ptr::addr_of_mut!(EntropyData) as *const u8,
        core::mem::size_of::<KSEC_ENTROPY_DATA>() as u32,
    );
    KsecLoadTimeStartMd5s[1usize] = Md5Context;
    MD5Final(core::ptr::addr_of_mut!(Md5Context));
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of_mut!(Md5Context.digest) as *const u8,
        (core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR).offset(16) as *mut u8,
        16,
    );

    aes_setup(
        core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR,
        32,
        0,
        core::ptr::addr_of_mut!(KsecGlobalAesKey),
    );
    des3_setup(
        core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR,
        24,
        0,
        core::ptr::addr_of_mut!(KsecGlobalDes3Key),
    );

    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(KeyDataBuffer) as PVOID,
        core::mem::size_of::<[UCHAR; 32]>() as SIZE_T,
    );
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(Md5Context) as PVOID,
        core::mem::size_of::<MD5_CTX>() as SIZE_T,
    );
}

unsafe fn KsecGetKeyData(mut KeyData: *mut UCHAR, mut OptionFlags: ULONG) {
    // ksecdd.h:25-27 / ntsecapi.h:700: SAME_PROCESS 0, CROSS_PROCESS 1, SAME_LOGON 2. All THREE were
    // permuted here -- the set of names is right and every value is in the wrong slot,
    // which is the recall failure mode a seal cannot catch (AGENTS.md, msvcrt _tzset).
    const RTL_ENCRYPT_OPTION_SAME_PROCESS: u32 = 0x00;
    const RTL_ENCRYPT_OPTION_CROSS_PROCESS: u32 = 0x01;
    const RTL_ENCRYPT_OPTION_SAME_LOGON: u32 = 0x02;

    let mut Md5Contexts: [MD5_CTX; 2] = [MD5_CTX {
        i: [0; 2],
        buf: [0; 4],
        in_0: [0; 64],
        digest: [0; 16],
    }; 2];
    let mut ProcessData: KSEC_PROCESS_DATA = KSEC_PROCESS_DATA {
        Process: core::ptr::null_mut(),
        ProcessId: core::ptr::null_mut(),
        CreateTime: 0,
        DirectoryTableBase: 0,
    };
    let mut LogonData: KSEC_LOGON_DATA = KSEC_LOGON_DATA {
        LogonId: LUID {
            LowPart: 0,
            HighPart: 0,
        },
    };
    let mut CurrentProcess: PEPROCESS;
    let mut Token: PACCESS_TOKEN;

    /* We need to generate the key, start with our load MD5s */
    Md5Contexts[0] = KsecLoadTimeStartMd5s[0];
    Md5Contexts[1] = KsecLoadTimeStartMd5s[1];

    /* Get the current process */
    CurrentProcess = PsGetCurrentProcess();

    if OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS {
        /* Hash some process specific data to generate the key */
        RtlZeroMemory(
            core::ptr::addr_of_mut!(ProcessData) as PVOID,
            core::mem::size_of::<KSEC_PROCESS_DATA>(),
        );
        ProcessData.Process = CurrentProcess;
        ProcessData.ProcessId = (*CurrentProcess).UniqueProcessId;
        ProcessData.CreateTime = PsGetProcessCreateTimeQuadPart(CurrentProcess);
        ProcessData.DirectoryTableBase = (*CurrentProcess).Pcb.DirectoryTableBase[0];
        MD5Update(
            core::ptr::addr_of_mut!(Md5Contexts[0]),
            core::ptr::addr_of_mut!(ProcessData) as PVOID as *const ::core::ffi::c_uchar,
            core::mem::size_of::<KSEC_PROCESS_DATA>() as ::core::ffi::c_uint,
        );
        MD5Update(
            core::ptr::addr_of_mut!(Md5Contexts[1]),
            core::ptr::addr_of_mut!(ProcessData) as PVOID as *const ::core::ffi::c_uchar,
            core::mem::size_of::<KSEC_PROCESS_DATA>() as ::core::ffi::c_uint,
        );
    } else if OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON {
        /* Hash the logon id to generate the key */
        RtlZeroMemory(
            core::ptr::addr_of_mut!(LogonData) as PVOID,
            core::mem::size_of::<KSEC_LOGON_DATA>(),
        );
        Token = PsReferencePrimaryToken(CurrentProcess) as PACCESS_TOKEN;
        SeQueryAuthenticationIdToken(Token, core::ptr::addr_of_mut!(LogonData.LogonId));
        PsDereferencePrimaryToken(Token);
        MD5Update(
            core::ptr::addr_of_mut!(Md5Contexts[0]),
            core::ptr::addr_of_mut!(LogonData) as PVOID as *const ::core::ffi::c_uchar,
            core::mem::size_of::<KSEC_LOGON_DATA>() as ::core::ffi::c_uint,
        );
        MD5Update(
            core::ptr::addr_of_mut!(Md5Contexts[1]),
            core::ptr::addr_of_mut!(LogonData) as PVOID as *const ::core::ffi::c_uchar,
            core::mem::size_of::<KSEC_LOGON_DATA>() as ::core::ffi::c_uint,
        );
    } else if !(OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS) {
        /* Must not pass anything else */
        if 0 == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                b"drivers/crypto/ksecdd/crypt.c\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
                122 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }

    /* Finalize the MD5s */
    MD5Final(core::ptr::addr_of_mut!(Md5Contexts[0]));
    MD5Final(core::ptr::addr_of_mut!(Md5Contexts[1]));

    /* Copy the md5 data */
    RtlCopyMemory(
        KeyData as PVOID,
        core::ptr::addr_of_mut!(Md5Contexts[0].digest) as PVOID,
        16,
    );
    RtlCopyMemory(
        (KeyData as PUCHAR).offset(16) as PVOID,
        core::ptr::addr_of_mut!(Md5Contexts[1].digest) as PVOID,
        16,
    );

    /* Erase the temp data */
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(Md5Contexts) as PVOID,
        core::mem::size_of::<[MD5_CTX; 2]>() as SIZE_T,
    );
}
unsafe fn KsecGetDes3Key(mut Des3Key: PDES3_KEY, mut OptionFlags: ULONG) {
    const RTL_ENCRYPT_OPTION_CROSS_PROCESS: ULONG = 0x1;
    let mut KeyDataBuffer: [UCHAR; 32] = [0; 32];

    if OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS {
        *Des3Key = core::ptr::read(core::ptr::addr_of!(KsecGlobalDes3Key));
    } else {
        KsecGetKeyData(core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR, OptionFlags);
        des3_setup(
            core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR,
            24,
            0,
            Des3Key,
        );
        RtlSecureZeroMemory(
            core::ptr::addr_of_mut!(KeyDataBuffer) as *mut UCHAR as PVOID,
            core::mem::size_of::<[UCHAR; 32]>() as SIZE_T,
        );
    }
}

const RTL_ENCRYPT_OPTION_CROSS_PROCESS: ULONG = 1;
unsafe fn KsecGetAesKey(mut AesKey: PAES_KEY, mut OptionFlags: ULONG) {
    let mut KeyDataBuffer: [UCHAR; 32] = [0; 32];

    if OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS {
        *AesKey = core::ptr::read(core::ptr::addr_of!(KsecGlobalAesKey));
    } else {
        KsecGetKeyData(KeyDataBuffer.as_mut_ptr() as *mut UCHAR, OptionFlags);
        aes_setup(
            KeyDataBuffer.as_mut_ptr() as *mut UCHAR,
            32 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            AesKey,
        );

        RtlSecureZeroMemory(
            KeyDataBuffer.as_mut_ptr() as *mut UCHAR as PVOID,
            ::core::mem::size_of::<[UCHAR; 32]>() as SIZE_T,
        );
    };
}
unsafe fn KsecEncryptMemoryDes3(
    mut Buffer: PVOID,
    mut Length: ULONG,
    mut OptionFlags: ULONG,
) {
    let mut EncryptedBlockData: [UCHAR; 8] = [0; 8];
    let mut Des3Key: DES3_KEY = core::mem::zeroed();

    /* Get they triple DES key */
    KsecGetDes3Key(core::ptr::addr_of_mut!(Des3Key), OptionFlags);

    /* Do the triple DES encryption */
    while Length as usize >= core::mem::size_of::<[UCHAR; 8]>() {
        des3_ecb_encrypt(
            Buffer as *const UCHAR,
            core::ptr::addr_of_mut!(EncryptedBlockData) as *mut UCHAR,
            core::ptr::addr_of_mut!(Des3Key),
        );
        RtlCopyMemory(
            Buffer,
            core::ptr::addr_of_mut!(EncryptedBlockData) as PVOID,
            core::mem::size_of::<[UCHAR; 8]>() as _,
        );
        Buffer = (Buffer as PUCHAR).offset(core::mem::size_of::<[UCHAR; 8]>() as isize) as PVOID;
        Length = (Length as usize).wrapping_sub(core::mem::size_of::<[UCHAR; 8]>()) as ULONG;
    }

    /* Erase the key data */
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(Des3Key) as PVOID,
        core::mem::size_of::<DES3_KEY>() as SIZE_T,
    );
}
unsafe fn KsecDecryptMemoryDes3(
    mut Buffer: PVOID,
    mut Length: ULONG,
    mut OptionFlags: ULONG,
) {
    let mut BlockData: [UCHAR; 8] = [0; 8];
    let mut Des3Key: DES3_KEY = core::mem::zeroed();

    /* Get they triple DES key */
    KsecGetDes3Key(core::ptr::addr_of_mut!(Des3Key), OptionFlags);

    /* Do the triple DES decryption */
    while Length as usize >= core::mem::size_of::<[UCHAR; 8]>() {
        des3_ecb_decrypt(
            Buffer as *const core::ffi::c_uchar,
            core::ptr::addr_of_mut!(BlockData) as *mut core::ffi::c_uchar,
            core::ptr::addr_of_mut!(Des3Key),
        );
        RtlCopyMemory(
            Buffer,
            core::ptr::addr_of_mut!(BlockData) as *const core::ffi::c_void,
            core::mem::size_of::<[UCHAR; 8]>() as _,
        );
        Buffer = (Buffer as PUCHAR).offset(core::mem::size_of::<[UCHAR; 8]>() as isize) as PVOID;
        Length = (Length as core::ffi::c_ulonglong)
            .wrapping_sub(core::mem::size_of::<[UCHAR; 8]>() as core::ffi::c_ulonglong)
            as ULONG;
    }

    /* Erase the key data */
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(Des3Key) as PVOID,
        core::mem::size_of::<DES3_KEY>() as SIZE_T,
    );
}
unsafe fn KsecEncryptMemoryAes(
    mut Buffer: PVOID,
    mut Length: ULONG,
    OptionFlags: ULONG,
) {

    let mut EncryptedBlockData: [UCHAR; 16] = [0; 16];
    let mut AesKey: AES_KEY = core::mem::zeroed();

    /* Get they AES key */
    KsecGetAesKey(core::ptr::addr_of_mut!(AesKey), OptionFlags);

    /* Do the AES encryption */
    while Length as usize >= core::mem::size_of::<[UCHAR; 16]>() {
        aes_ecb_encrypt(
            Buffer as *const ::core::ffi::c_uchar,
            core::ptr::addr_of_mut!(EncryptedBlockData) as *mut ::core::ffi::c_uchar,
            core::ptr::addr_of_mut!(AesKey),
        );
        RtlCopyMemory(
            Buffer,
            core::ptr::addr_of_mut!(EncryptedBlockData) as *const core::ffi::c_void,
            core::mem::size_of::<[UCHAR; 16]>() as _,
        );
        Buffer = (Buffer as PUCHAR).offset(core::mem::size_of::<[UCHAR; 16]>() as isize) as PVOID;
        Length = (Length as core::ffi::c_ulonglong)
            .wrapping_sub(core::mem::size_of::<[UCHAR; 16]>() as core::ffi::c_ulonglong)
            as ULONG;
    }

    /* Erase the key data */
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(AesKey) as PVOID,
        core::mem::size_of::<AES_KEY>() as SIZE_T,
    );
}
unsafe fn KsecDecryptMemoryAes(
    mut Buffer: PVOID,
    mut Length: ULONG,
    OptionFlags: ULONG,
) {

    let mut BlockData: [UCHAR; 16] = [0; 16];
    let mut AesKey: AES_KEY = AES_KEY {
        eK: [0; 64],
        dK: [0; 64],
        Nr: 0,
    };

    /* Get they AES key */
    KsecGetAesKey(core::ptr::addr_of_mut!(AesKey), OptionFlags);

    /* Do the AES decryption */
    while Length as usize >= core::mem::size_of::<[UCHAR; 16]>() {
        aes_ecb_decrypt(
            Buffer as *const core::ffi::c_uchar,
            core::ptr::addr_of_mut!(BlockData) as *mut core::ffi::c_uchar,
            core::ptr::addr_of_mut!(AesKey),
        );
        RtlCopyMemory(
            Buffer,
            core::ptr::addr_of_mut!(BlockData) as *const core::ffi::c_void,
            core::mem::size_of::<[UCHAR; 16]>() as _,
        );
        Buffer = (Buffer as PUCHAR).offset(core::mem::size_of::<[UCHAR; 16]>() as isize) as PVOID;
        Length = (Length as core::ffi::c_ulonglong)
            .wrapping_sub(core::mem::size_of::<[UCHAR; 16]>() as core::ffi::c_ulonglong)
            as ULONG;
    }

    /* Erase the key data */
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!(AesKey) as PVOID,
        core::mem::size_of::<AES_KEY>() as SIZE_T,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsecEncryptMemory(
    Buffer: PVOID,
    Length: ULONG,
    OptionFlags: ULONG,
) -> NTSTATUS {
    // ksecdd.h:25-27 / ntsecapi.h:700: SAME_PROCESS 0, CROSS_PROCESS 1, SAME_LOGON 2.
    const RTL_ENCRYPT_OPTION_SAME_LOGON: u32 = 2;

    /* Validate parameter */
    if OptionFlags > RTL_ENCRYPT_OPTION_SAME_LOGON {
        return STATUS_INVALID_PARAMETER;
    }

    /* Check if the length is not 16 bytes aligned */
    if Length & 15 != 0 {
        /* Is it at least 8 bytes aligned? */
        if Length & 7 != 0 {
            /* No, we can't deal with it! */
            return STATUS_INVALID_PARAMETER;
        }

        /* Use triple DES encryption */
        KsecEncryptMemoryDes3(Buffer, Length, OptionFlags);
    } else {
        /* Use AES encryption */
        KsecEncryptMemoryAes(Buffer, Length, OptionFlags);
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn KsecDecryptMemory(
    Buffer: PVOID,
    Length: ULONG,
    OptionFlags: ULONG,
) -> NTSTATUS {
    /* Validate parameter */
    if OptionFlags > RTL_ENCRYPT_OPTION_SAME_LOGON {
        return STATUS_INVALID_PARAMETER;
    }

    /* Check if the length is not 16 bytes aligned */
    if Length & 15 != 0 {
        /* Is it at least 8 bytes aligned? */
        if Length & 7 != 0 {
            /* No, we can't deal with it! */
            return STATUS_INVALID_PARAMETER;
        }

        /* Use triple DES encryption */
        KsecDecryptMemoryDes3(Buffer, Length, OptionFlags);
    } else {
        /* Use AES encryption */
        KsecDecryptMemoryAes(Buffer, Length, OptionFlags);
    }

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Drivers
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PURPOSE:         Kernel Security Support Provider Interface Driver
//  *
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ksecdd.h"
//
// MD5_CTX KsecLoadTimeStartMd5s[2];
// DES3_KEY KsecGlobalDes3Key;
// AES_KEY KsecGlobalAesKey;
//
// typedef struct _KSEC_PROCESS_DATA
// {
//     PEPROCESS Process;
//     HANDLE ProcessId;
//     LONGLONG CreateTime;
//     ULONG_PTR DirectoryTableBase;
// } KSEC_PROCESS_DATA, *PKSEC_PROCESS_DATA;
//
// typedef struct _KSEC_LOGON_DATA
// {
//     LUID LogonId;
// } KSEC_LOGON_DATA, *PKSEC_LOGON_DATA;
//
//
// VOID
// NTAPI
// KsecInitializeEncryptionSupport (
//     VOID)
// {
//     KSEC_ENTROPY_DATA EntropyData;
//     MD5_CTX Md5Context;
//     UCHAR KeyDataBuffer[32];
//
//     KsecGatherEntropyData(&EntropyData);
//     MD5Init(&Md5Context);
//     MD5Update(&Md5Context, (PVOID)&EntropyData, sizeof(EntropyData));
//     KsecLoadTimeStartMd5s[0] = Md5Context;
//     MD5Final(&Md5Context);
//     RtlCopyMemory(KeyDataBuffer, &Md5Context.digest, 16);
//
//     KsecGatherEntropyData(&EntropyData);
//     Md5Context = KsecLoadTimeStartMd5s[0];
//     MD5Update(&Md5Context, (PVOID)&EntropyData, sizeof(EntropyData));
//     KsecLoadTimeStartMd5s[1] = Md5Context;
//     MD5Final(&Md5Context);
//     RtlCopyMemory(&KeyDataBuffer[16], &Md5Context.digest, 16);
//
//     /* Create the global keys */
//     aes_setup(KeyDataBuffer, 32, 0, &KsecGlobalAesKey);
//     des3_setup(KeyDataBuffer, 24, 0, &KsecGlobalDes3Key);
//
//     /* Erase the temp data */
//     RtlSecureZeroMemory(KeyDataBuffer, sizeof(KeyDataBuffer));
//     RtlSecureZeroMemory(&Md5Context, sizeof(Md5Context));
// }
//
// static
// VOID
// KsecGetKeyData (
//      UCHAR KeyData[32],
//      ULONG OptionFlags)
// {
//     MD5_CTX Md5Contexts[2];
//     KSEC_PROCESS_DATA ProcessData;
//     KSEC_LOGON_DATA LogonData;
//     PEPROCESS CurrentProcess;
//     PACCESS_TOKEN Token;
//
//     /* We need to generate the key, start with our load MD5s */
//     Md5Contexts[0] = KsecLoadTimeStartMd5s[0];
//     Md5Contexts[1] = KsecLoadTimeStartMd5s[1];
//
//     /* Get the current process */
//     CurrentProcess = PsGetCurrentProcess();
//
//     if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_PROCESS)
//     {
//         /* Hash some process specific data to generate the key */
//         RtlZeroMemory(&ProcessData, sizeof(ProcessData));
//         ProcessData.Process = CurrentProcess;
//         ProcessData.ProcessId = CurrentProcess->UniqueProcessId;
//         ProcessData.CreateTime = PsGetProcessCreateTimeQuadPart(CurrentProcess);
//         ProcessData.DirectoryTableBase = CurrentProcess->Pcb.DirectoryTableBase[0];
//         MD5Update(&Md5Contexts[0], (PVOID)&ProcessData, sizeof(ProcessData));
//         MD5Update(&Md5Contexts[1], (PVOID)&ProcessData, sizeof(ProcessData));
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_SAME_LOGON)
//     {
//         /* Hash the logon id to generate the key */
//         RtlZeroMemory(&LogonData, sizeof(LogonData));
//         Token = PsReferencePrimaryToken(CurrentProcess);
//         SeQueryAuthenticationIdToken(Token, &LogonData.LogonId);
//         PsDereferencePrimaryToken(Token);
//         MD5Update(&Md5Contexts[0], (PVOID)&LogonData, sizeof(LogonData));
//         MD5Update(&Md5Contexts[1], (PVOID)&LogonData, sizeof(LogonData));
//     }
//     else if (OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS)
//     {
//         /* Use the original MD5s to generate the global key */
//         NOTHING;
//     }
//     else
//     {
//         /* Must not pass anything else */
//         ASSERT(FALSE);
//     }
//
//     /* Finalize the MD5s */
//     MD5Final(&Md5Contexts[0]);
//     MD5Final(&Md5Contexts[1]);
//
//     /* Copy the md5 data */
//     RtlCopyMemory(KeyData, &Md5Contexts[0].digest, 16);
//     RtlCopyMemory((PUCHAR)KeyData + 16, &Md5Contexts[1].digest, 16);
//
//     /* Erase the temp data */
//     RtlSecureZeroMemory(&Md5Contexts, sizeof(Md5Contexts));
// }
//
// static
// VOID
// KsecGetDes3Key (
//      PDES3_KEY Des3Key,
//      ULONG OptionFlags)
// {
//     UCHAR KeyDataBuffer[32];
//
//     /* Check if the caller allows cross process encryption */
//     if (OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS)
//     {
//         /* Return our global cached DES3 key */
//         *Des3Key = KsecGlobalDes3Key;
//     }
//     else
//     {
//         /* Setup the key */
//         KsecGetKeyData(KeyDataBuffer, OptionFlags);
//         des3_setup(KeyDataBuffer, 24, 0, Des3Key);
//
//         /* Erase the temp data */
//         RtlSecureZeroMemory(KeyDataBuffer, sizeof(KeyDataBuffer));
//     }
// }
//
// static
// VOID
// KsecGetAesKey (
//      PAES_KEY AesKey,
//      ULONG OptionFlags)
// {
//     UCHAR KeyDataBuffer[32];
//
//     /* Check if the caller allows cross process encryption */
//     if (OptionFlags == RTL_ENCRYPT_OPTION_CROSS_PROCESS)
//     {
//         /* Return our global cached AES key */
//         *AesKey = KsecGlobalAesKey;
//     }
//     else
//     {
//         /* Setup the key */
//         KsecGetKeyData(KeyDataBuffer, OptionFlags);
//         aes_setup(KeyDataBuffer, 32, 0, AesKey);
//
//         /* Erase the temp data */
//         RtlSecureZeroMemory(KeyDataBuffer, sizeof(KeyDataBuffer));
//     }
// }
//
// static
// VOID
// KsecEncryptMemoryDes3 (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     UCHAR EncryptedBlockData[8];
//     DES3_KEY Des3Key;
//
//     /* Get they triple DES key */
//     KsecGetDes3Key(&Des3Key, OptionFlags);
//
//     /* Do the triple DES encryption */
//     while (Length >= sizeof(EncryptedBlockData))
//     {
//         des3_ecb_encrypt(Buffer, EncryptedBlockData, &Des3Key);
//         RtlCopyMemory(Buffer, EncryptedBlockData, sizeof(EncryptedBlockData));
//         Buffer = (PUCHAR)Buffer + sizeof(EncryptedBlockData);
//         Length -= sizeof(EncryptedBlockData);
//     }
//
//     /* Erase the key data */
//     RtlSecureZeroMemory(&Des3Key, sizeof(Des3Key));
// }
//
// static
// VOID
// KsecDecryptMemoryDes3 (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     UCHAR BlockData[8];
//     DES3_KEY Des3Key;
//
//     /* Get they triple DES key */
//     KsecGetDes3Key(&Des3Key, OptionFlags);
//
//     /* Do the triple DES decryption */
//     while (Length >= sizeof(BlockData))
//     {
//         des3_ecb_decrypt(Buffer, BlockData, &Des3Key);
//         RtlCopyMemory(Buffer, BlockData, sizeof(BlockData));
//         Buffer = (PUCHAR)Buffer + sizeof(BlockData);
//         Length -= sizeof(BlockData);
//     }
//
//     /* Erase the key data */
//     RtlSecureZeroMemory(&Des3Key, sizeof(Des3Key));
// }
//
// static
// VOID
// KsecEncryptMemoryAes (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     UCHAR EncryptedBlockData[16];
//     AES_KEY AesKey;
//
//     /* Get they AES key */
//     KsecGetAesKey(&AesKey, OptionFlags);
//
//     /* Do the AES encryption */
//     while (Length >= sizeof(EncryptedBlockData))
//     {
//         aes_ecb_encrypt(Buffer, EncryptedBlockData, &AesKey);
//         RtlCopyMemory(Buffer, EncryptedBlockData, sizeof(EncryptedBlockData));
//         Buffer = (PUCHAR)Buffer + sizeof(EncryptedBlockData);
//         Length -= sizeof(EncryptedBlockData);
//     }
//
//     /* Erase the key data */
//     RtlSecureZeroMemory(&AesKey, sizeof(AesKey));
// }
//
// static
// VOID
// KsecDecryptMemoryAes (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     UCHAR BlockData[16];
//     AES_KEY AesKey;
//
//     /* Get they AES key */
//     KsecGetAesKey(&AesKey, OptionFlags);
//
//     /* Do the AES decryption */
//     while (Length >= sizeof(BlockData))
//     {
//         aes_ecb_decrypt(Buffer, BlockData, &AesKey);
//         RtlCopyMemory(Buffer, BlockData, sizeof(BlockData));
//         Buffer = (PUCHAR)Buffer + sizeof(BlockData);
//         Length -= sizeof(BlockData);
//     }
//
//     /* Erase the key data */
//     RtlSecureZeroMemory(&AesKey, sizeof(AesKey));
// }
//
// NTSTATUS
// NTAPI
// KsecEncryptMemory (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     /* Validate parameter */
//     if (OptionFlags > RTL_ENCRYPT_OPTION_SAME_LOGON)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if the length is not 16 bytes aligned */
//     if (Length & 15)
//     {
//         /* Is it at least 8 bytes aligned? */
//         if (Length & 7)
//         {
//             /* No, we can't deal with it! */
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Use triple DES encryption */
//         KsecEncryptMemoryDes3(Buffer, Length, OptionFlags);
//     }
//     else
//     {
//         /* Use AES encryption */
//         KsecEncryptMemoryAes(Buffer, Length, OptionFlags);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// KsecDecryptMemory (
//      PVOID Buffer,
//      ULONG Length,
//      ULONG OptionFlags)
// {
//     /* Validate parameter */
//     if (OptionFlags > RTL_ENCRYPT_OPTION_SAME_LOGON)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if the length is not 16 bytes aligned */
//     if (Length & 15)
//     {
//         /* Is it at least 8 bytes aligned? */
//         if (Length & 7)
//         {
//             /* No, we can't deal with it! */
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Use triple DES encryption */
//         KsecDecryptMemoryDes3(Buffer, Length, OptionFlags);
//     }
//     else
//     {
//         /* Use AES encryption */
//         KsecDecryptMemoryAes(Buffer, Length, OptionFlags);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
