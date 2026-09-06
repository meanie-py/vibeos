use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NTLM_ALLOC_TAG: *const u8 = b"NTLM\0".as_ptr();
pub const NTLM_ALLOC_TAG_SIZE: usize = 4;
#[no_mangle]
pub unsafe extern "C" fn NtlmAllocate(
    Size: usize,
    UsePrivateLsaHeap: bool,
) -> *mut core::ffi::c_void {
    let mut buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    if Size == 0 {
        return core::ptr::null_mut();
    }

    let Size = Size + NTLM_ALLOC_TAG_SIZE;

    match NtlmMode {
        NtlmLsaMode => {
            if UsePrivateLsaHeap {
                // AllocatePrivateHeap takes SIZE_T, AllocateLsaHeap below takes ULONG
                // (ntsecpkg.h:282 vs :246) — the C's implicit conversions differ per arm.
                buffer = ((*LsaFunctions).AllocatePrivateHeap.unwrap_unchecked())(Size as SIZE_T);
            } else {
                buffer = ((*LsaFunctions).AllocateLsaHeap.unwrap_unchecked())(Size as ULONG);
            }

            if !buffer.is_null() {
                RtlZeroMemory!(buffer, Size);
            }
        }
        NtlmUserMode => {
            buffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T);
        }
        _ => {}
    }

    memcpy(buffer, (NTLM_ALLOC_TAG).cast(), NTLM_ALLOC_TAG_SIZE);
    buffer = (buffer as PBYTE).add(NTLM_ALLOC_TAG_SIZE) as *mut core::ffi::c_void;

    buffer
}
#[no_mangle]
pub unsafe extern "C" fn NtlmFree(
    Buffer: PVOID,
    FromPrivateLsaHeap: bool,
) {
    if !Buffer.is_null() {
        let mut Buffer = (Buffer as PBYTE).offset(-((NTLM_ALLOC_TAG_SIZE) as isize));

        *(Buffer as *mut i8) = b'D' as i8;

        match NtlmMode {
            _NTLM_MODE_NtlmLsaMode => {
                if FromPrivateLsaHeap {
                    ((*LsaFunctions).FreePrivateHeap.unwrap_unchecked())(Buffer as PVOID);
                } else {
                    ((*LsaFunctions).FreeLsaHeap.unwrap_unchecked())(Buffer as PVOID);
                }
            }
            _NTLM_MODE_NtlmUserMode => {
                HeapFree(GetProcessHeap(), HEAP_ZERO_MEMORY, Buffer as PVOID);
            }
            _ => {
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn NtlmUStrAlloc(
    Dst: PUNICODE_STRING,
    SizeInBytes: UINT16,
    InitLength: UINT16,
) -> bool {
    (*Dst).Length = InitLength;
    (*Dst).MaximumLength = SizeInBytes;
    (*Dst).Buffer = NtlmAllocate(SizeInBytes as usize, false) as *mut u16;
    (*Dst).Buffer != core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn NtlmUStrFree(
    String: PUNICODE_STRING,
) {
    if String.is_null() || (*String).Buffer.is_null() || (*String).MaximumLength == 0 {
        return;
    }

    NtlmFree((*String).Buffer as PVOID, false);
    (*String).Buffer = core::ptr::null_mut();
    (*String).MaximumLength = 0;
}
unsafe extern "C" fn NtlmStructWriteStrW(
    DataStart: PVOID,
    DataSize: ULONG,
    DstDataWPtr: *mut PWCHAR,
    SrcDataW: *const WCHAR,
    mut SrcDataLen: ULONG,
    AbsoluteOffsetPtr: *mut PBYTE,
    TerminateWith0: bool,
) -> bool {
    let mut SrcDataMaxLen: ULONG;

    if SrcDataLen == 0 {
        SrcDataLen = (wcslen(SrcDataW) * (core::mem::size_of::<WCHAR>() as ULONG) as usize) as u32;
    }

    SrcDataMaxLen = SrcDataLen;
    if TerminateWith0 {
        SrcDataMaxLen += core::mem::size_of::<WCHAR>() as ULONG;
    }

    if (*AbsoluteOffsetPtr as PBYTE) < (DataStart as PBYTE) {
        return false;
    }

    if (*AbsoluteOffsetPtr as PBYTE).add(SrcDataMaxLen as usize) > (DataStart as PBYTE).add(DataSize as usize) {
        return false;
    }

    memcpy((*AbsoluteOffsetPtr).cast(), SrcDataW as *const core::ffi::c_void, SrcDataLen as usize);
    *DstDataWPtr = *AbsoluteOffsetPtr as *mut WCHAR;
    if TerminateWith0 {
        (*(*DstDataWPtr).add((SrcDataLen / core::mem::size_of::<WCHAR>() as ULONG) as usize)) = 0;
    }
    *AbsoluteOffsetPtr = ((*AbsoluteOffsetPtr as PBYTE).add(SrcDataMaxLen as usize) as *mut core::ffi::c_void).cast();

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn NtlmUStrWriteToStruct(
    DataStart: PVOID,
    DataSize: ULONG,
    DstData: PUNICODE_STRING,
    SrcData: PUNICODE_STRING,
    AbsoluteOffsetPtr: *mut PBYTE,
    TerminateWith0: bool,
) -> bool {
    if !NtlmStructWriteStrW(
        DataStart,
        DataSize,
        core::ptr::addr_of_mut!((*DstData).Buffer),
        (*SrcData).Buffer,
        ((*SrcData).Length) as u32,
        AbsoluteOffsetPtr,
        TerminateWith0,
    ) {
        return false;
    }

    (*DstData).Length = (*SrcData).Length;
    (*DstData).MaximumLength = (*SrcData).Length;
    if TerminateWith0 {
        (*SrcData).MaximumLength = (*SrcData).MaximumLength.wrapping_add(core::mem::size_of::<WCHAR>() as UINT16);
    }

    true
}
#[no_mangle]
pub unsafe extern "C" fn NtlmFixupAndValidateUStr(
    String: PUNICODE_STRING,
    FixupOffset: usize,
) -> bool {
    let Status: i32;

    if (*String).Length != 0 {
        (*String).Buffer = FIXUP_POINTER!((*String).Buffer, FixupOffset);
        (*String).MaximumLength = (*String).Length;
    } else {
        (*String).Buffer = core::ptr::null_mut();
        (*String).MaximumLength = 0;
    }

    Status = RtlValidateUnicodeString(0, String);
    NT_SUCCESS(Status)
}
#[no_mangle]
pub unsafe extern "C" fn NtlmFixupAStr(
    String: PSTRING,
    FixupOffset: usize,
) -> bool {
    if (*String).Length != 0 {
        (*String).Buffer = FIXUP_POINTER!((*String).Buffer, FixupOffset) as PCHAR;
        (*String).MaximumLength = (*String).Length;
    } else {
        (*String).Buffer = core::ptr::null_mut();
        (*String).MaximumLength = 0;
    }

    true
}
#[no_mangle]
pub unsafe extern "C" fn NtlmAllocateClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    BufferLength: u32,
    Buffer: PNTLM_CLIENT_BUFFER,
) -> i32 {
    let mut Status: i32 = STATUS_SUCCESS;

    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*Buffer).LocalBuffer = NtlmAllocate(BufferLength as usize, false);
    if (*Buffer).LocalBuffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    if ClientRequest as HANDLE == INVALID_HANDLE_VALUE {
        (*Buffer).ClientBaseAddress = (*Buffer).LocalBuffer;
    } else {
        Status = (DispatchTable.AllocateClientBuffer.unwrap_unchecked())(
            ClientRequest,
            BufferLength,
            core::ptr::addr_of_mut!((*Buffer).ClientBaseAddress),
        );
        if !NT_SUCCESS(Status) {
            NtlmFree((*Buffer).LocalBuffer, false);
            (*Buffer).LocalBuffer = core::ptr::null_mut();
        }
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn NtlmCopyToClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    BufferLength: ULONG,
    Buffer: PNTLM_CLIENT_BUFFER,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if ClientRequest as HANDLE == INVALID_HANDLE_VALUE {
        if (*Buffer).ClientBaseAddress != (*Buffer).LocalBuffer {
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        if (*Buffer).ClientBaseAddress.is_null() || (*Buffer).LocalBuffer.is_null() {
            return STATUS_NO_MEMORY;
        }
        Status = (DispatchTable.CopyToClientBuffer.unwrap_unchecked())(
            ClientRequest,
            BufferLength,
            (*Buffer).ClientBaseAddress,
            (*Buffer).LocalBuffer,
        );
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn NtlmFreeClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    FreeClientBuffer: bool,
    Buffer: PNTLM_CLIENT_BUFFER,
) {
    if (*Buffer).ClientBaseAddress.is_null() {
        return;
    }

    if ClientRequest as HANDLE == INVALID_HANDLE_VALUE {
        if (*Buffer).ClientBaseAddress != (*Buffer).LocalBuffer {
            return;
        }
        (*Buffer).LocalBuffer = core::ptr::null_mut();
        if FreeClientBuffer {
            NtlmFree((*Buffer).ClientBaseAddress, false);
            (*Buffer).ClientBaseAddress = core::ptr::null_mut();
        }
    } else {
        NtlmFree((*Buffer).LocalBuffer, false);
        (*Buffer).LocalBuffer = core::ptr::null_mut();
        if FreeClientBuffer {
            (DispatchTable.FreeClientBuffer.unwrap_unchecked())(ClientRequest, (*Buffer).ClientBaseAddress);
        }
        (*Buffer).ClientBaseAddress = core::ptr::null_mut();
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Utils for msv1_0
//  * COPYRIGHT:   Copyright 2011 Samuel Serapión
//  *              Copyright 2020 Andreas Maier <staubim@quantentunnel.de>
//  */
//
// #include "../precomp.h"
//
// #include "wine/debug.h"
// WINE_DEFAULT_DEBUG_CHANNEL(ntlm);
//
// #define NTLM_ALLOC_TAG "NTLM"
// #define NTLM_ALLOC_TAG_SIZE strlen(NTLM_ALLOC_TAG)
//
// PVOID
// NtlmAllocate(
//      size_t Size,
//      bool UsePrivateLsaHeap)
// {
//     PVOID buffer = NULL;
//
//     if (Size == 0)
//     {
//         ERR("Allocating 0 bytes!\n");
//         return NULL;
//     }
//
//     Size += NTLM_ALLOC_TAG_SIZE;
//
//     switch (NtlmMode)
//     {
//         case NtlmLsaMode:
//         {
//             if (UsePrivateLsaHeap)
//                 buffer = LsaFunctions->AllocatePrivateHeap(Size);
//             else
//                 buffer = LsaFunctions->AllocateLsaHeap(Size);
//
//             if (buffer != NULL)
//                 RtlZeroMemory!(buffer, Size);
//             break;
//         }
//         case NtlmUserMode:
//         {
//             buffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T);
//             break;
//         }
//         default:
//         {
//             ERR("NtlmState unknown!\n");
//             break;
//         }
//     }
//
//     memcpy(buffer, NTLM_ALLOC_TAG, NTLM_ALLOC_TAG_SIZE);
//     buffer = (PBYTE)buffer + NTLM_ALLOC_TAG_SIZE;
//
//     return buffer;
// }
//
// VOID
// NtlmFree(
//      PVOID Buffer,
//      bool FromPrivateLsaHeap)
// {
//     if (Buffer)
//     {
//         Buffer = (PBYTE)Buffer - NTLM_ALLOC_TAG_SIZE;
//         ASSERT(memcmp(Buffer, NTLM_ALLOC_TAG, NTLM_ALLOC_TAG_SIZE) == 0);
//         *(char*)Buffer = 'D';
//
//         switch (NtlmMode)
//         {
//             case NtlmLsaMode:
//             {
//                 if (FromPrivateLsaHeap)
//                     LsaFunctions->FreePrivateHeap(Buffer);
//                 else
//                     LsaFunctions->FreeLsaHeap(Buffer);
//                 break;
//             }
//             case NtlmUserMode:
//             {
//                 HeapFree(GetProcessHeap(), HEAP_ZERO_MEMORY, Buffer);
//                 break;
//             }
//             default:
//             {
//                 ERR("NtlmState unknown!\n");
//                 break;
//             }
//         }
//     }
//     else
//     {
//         ERR("Trying to free NULL!\n");
//     }
// }
//
// bool
// NtlmUStrAlloc(
//      PUNICODE_STRING Dst,
//      UINT16 SizeInBytes,
//      UINT16 InitLength)
// {
//     Dst->Length = InitLength;
//     Dst->MaximumLength = SizeInBytes;
//     Dst->Buffer = NtlmAllocate(SizeInBytes, false);
//     return (Dst->Buffer != NULL);
// }
//
// VOID
// NtlmUStrFree(
//      PUNICODE_STRING String)
// {
//     if (String == NULL || String->Buffer == NULL || String->MaximumLength == 0)
//         return;
//
//     NtlmFree(String->Buffer, false);
//     String->Buffer = NULL;
//     String->MaximumLength = 0;
// }
//
// /**
//  * @brief Helper to fill a WCHAR-String in a struct.
//  *        The stringdata is appended to the struct.
//  *        The function does not allocate memory.
//  *
//  * @param[in] DataStart
//  * Start address of the struct
//  *
//  * @param[in] DataSize
//  * Size of allocated memory (including payload)
//  *
//  * @param[out] DstDataWPtr
//  * Pointer to the WCHAR* datafield. The address of the data will be written to it.
//  *
//  * @param[in] SrcDataW
//  * Data to write/append at pOffset (payload). pOffset will be increased after writing data.
//  *
//  * @param[in] SrcDataLen
//  * SrcDataLen is the length in bytes without terminator.
//  * if 0 it will be autodetected by assuming a 0-terminating string.
//  *
//  * @param[in,out] AbsoluteOffsetPtr
//  * Current absolute offset. Will be increased by data length.
//  *
//  * @param[in] TerminateWith0
//  * Whether to terminate the string with a NULL-char.
//  *
//  * @return FALSE if something went wrong
//  */
// static
// bool
// NtlmStructWriteStrW(
//      PVOID DataStart,
//      ULONG DataSize,
//      PWCHAR* DstDataWPtr,
//      const WCHAR* SrcDataW,
//      ULONG SrcDataLen,
//      PBYTE* AbsoluteOffsetPtr,
//      bool TerminateWith0)
// {
//     ULONG SrcDataMaxLen;
//
//     if (SrcDataLen == 0)
//         SrcDataLen = wcslen(SrcDataW) * sizeof(WCHAR);
//
//     SrcDataMaxLen = SrcDataLen;
//     if (TerminateWith0)
//         SrcDataMaxLen += sizeof(WCHAR);
//
//     if (*AbsoluteOffsetPtr < (PBYTE)DataStart)
//     {
//         ERR("Invalid offset\n");
//         return false;
//     }
//
//     if (*AbsoluteOffsetPtr + SrcDataMaxLen > (PBYTE)DataStart + DataSize)
//     {
//         ERR("Out of bounds!\n");
//         return false;
//     }
//
//     memcpy(*AbsoluteOffsetPtr, SrcDataW, SrcDataLen);
//     *DstDataWPtr = (WCHAR*)*AbsoluteOffsetPtr;
//     if (TerminateWith0)
//         (*DstDataWPtr)[SrcDataLen / sizeof(WCHAR)] = 0;
//     *AbsoluteOffsetPtr += SrcDataMaxLen;
//
//     return true;
// }
//
// bool
// NtlmUStrWriteToStruct(
//      PVOID DataStart,
//      ULONG DataSize,
//      PUNICODE_STRING DstData,
//      const PUNICODE_STRING SrcData,
//      PBYTE* AbsoluteOffsetPtr,
//      bool TerminateWith0)
// {
//     if (!NtlmStructWriteStrW(DataStart,
//                              DataSize,
//                              &DstData->Buffer,
//                              SrcData->Buffer,
//                              SrcData->Length,
//                              AbsoluteOffsetPtr,
//                              TerminateWith0))
//         return false;
//
//     DstData->Length = SrcData->Length;
//     DstData->MaximumLength = SrcData->Length;
//     if (TerminateWith0)
//         SrcData->MaximumLength += sizeof(WCHAR);
//
//     return true;
// }
//
// bool
// NtlmFixupAndValidateUStr(
//      PUNICODE_STRING String,
//      ULONG_PTR FixupOffset)
// {
//     NTSTATUS Status;
//
//     if (String->Length)
//     {
//         String->Buffer = FIXUP_POINTER!(String->Buffer, FixupOffset);
//         String->MaximumLength = String->Length;
//     }
//     else
//     {
//         String->Buffer = NULL;
//         String->MaximumLength = 0;
//     }
//
//     Status = RtlValidateUnicodeString(0, String);
//     return NT_SUCCESS(Status);
// }
//
// bool
// NtlmFixupAStr(
//      PSTRING String,
//      ULONG_PTR FixupOffset)
// {
//     if (String->Length)
//     {
//         String->Buffer = (PCHAR)FIXUP_POINTER!(String->Buffer, FixupOffset);
//         String->MaximumLength = String->Length;
//     }
//     else
//     {
//         String->Buffer = NULL;
//         String->MaximumLength = 0;
//     }
//
//     return true;
// }
//
// NTSTATUS
// NtlmAllocateClientBuffer(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      ULONG BufferLength,
//      PNTLM_CLIENT_BUFFER Buffer)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (!Buffer)
//         return STATUS_NO_MEMORY;
//
//     Buffer->LocalBuffer = NtlmAllocate(BufferLength, false);
//     if (!Buffer->LocalBuffer)
//         return STATUS_NO_MEMORY;
//
//     if ((HANDLE)ClientRequest == INVALID_HANDLE_VALUE)
//     {
//         Buffer->ClientBaseAddress = Buffer->LocalBuffer;
//         //if (!ClientBaseAddress)
//         //    return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         Status = DispatchTable.AllocateClientBuffer.unwrap_unchecked()(ClientRequest,
//                                                     BufferLength,
//                                                     &Buffer->ClientBaseAddress);
//         if (!NT_SUCCESS(Status))
//         {
//             NtlmFree(Buffer->LocalBuffer, false);
//             Buffer->LocalBuffer = NULL;
//         }
//         //FIXME: Maybe we have to free ClientBaseAddress if something
//         //       goes wrong ...? I'm not sure about that ...
//     }
//     return Status;
// }
//
// NTSTATUS
// NtlmCopyToClientBuffer(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      ULONG BufferLength,
//      PNTLM_CLIENT_BUFFER Buffer)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if ((HANDLE)ClientRequest == INVALID_HANDLE_VALUE)
//     {
//         // If ClientRequest ist INVALID_HANDLE_VALUE
//         // Buffer->LocalBuffer == Buffer->ClientBaseAddress
//         if (Buffer->ClientBaseAddress != Buffer->LocalBuffer)
//         {
//             ERR("Buffer->ClientBaseAddress != Buffer->LocalBuffer (something must be wrong!)\n");
//             return STATUS_INTERNAL_ERROR;
//         }
//     }
//     else
//     {
//         if (!Buffer->ClientBaseAddress ||
//             !Buffer->LocalBuffer)
//         {
//             ERR("Invalid Buffer - not allocated!\n");
//             return STATUS_NO_MEMORY;
//         }
//         Status = DispatchTable.CopyToClientBuffer.unwrap_unchecked()(ClientRequest,
//                                                   BufferLength,
//                                                   Buffer->ClientBaseAddress,
//                                                   Buffer->LocalBuffer);
//     }
//     return Status;
// }
//
// VOID
// NtlmFreeClientBuffer(
//      PLSA_CLIENT_REQUEST ClientRequest,
//      bool FreeClientBuffer,
//      PNTLM_CLIENT_BUFFER Buffer)
// {
//     if (!Buffer->ClientBaseAddress)
//         return;
//
//     if ((HANDLE)ClientRequest == INVALID_HANDLE_VALUE)
//     {
//         if (Buffer->ClientBaseAddress != Buffer->LocalBuffer)
//         {
//             ERR("Buffer->ClientBaseAddress != Buffer->LocalBuffer (something must be wrong!)\n");
//             return;
//         }
//         // LocalBuffer and ClientBaseAddress is the same
//         // so we have only to free it if FreeClientBuffer is TRUE.
//         Buffer->LocalBuffer = NULL;
//         if (FreeClientBuffer)
//         {
//             NtlmFree(Buffer->ClientBaseAddress, false);
//             Buffer->ClientBaseAddress = NULL;
//         }
//     }
//     else
//     {
//         NtlmFree(Buffer->LocalBuffer, false);
//         Buffer->LocalBuffer = NULL;
//         if (FreeClientBuffer)
//             DispatchTable.FreeClientBuffer.unwrap_unchecked()(ClientRequest, Buffer->ClientBaseAddress);
//         Buffer->ClientBaseAddress = NULL;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
