use ntkrnl_sys::globals::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use crate::types::*;
use crate::glue::*;
use crate::glue::RtlSecureZeroMemory; // explicit: shadow ddk::* phantom (FORCEINLINE)
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
// IRP-stack accessor is a FORCEINLINE (no export symbol) — route via inline::tier1
// (explicit import shadows the ddk::* glob's never-linking bindgen extern).
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use core::ffi::c_void;

// Ksec* are NTAPI (= stdcall on i386) and defined `extern "system"` in random.rs /
// crypt.rs — declare them with matching ABI so i386 resolves `_Name@N`, not `_Name`.
extern "system" {
    fn KsecGenRandom(Buffer: PVOID, Length: SIZE_T) -> NTSTATUS;
    fn KsecEncryptMemory(Buffer: PVOID, Length: ULONG, Option: ULONG) -> NTSTATUS;
    fn KsecDecryptMemory(Buffer: PVOID, Length: ULONG, Option: ULONG) -> NTSTATUS;
}
// DbgPrint is cdecl-variadic; __debugbreak is the MSVC intrinsic (cdecl).
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
    fn __debugbreak();
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG
unsafe fn KsecQueryFileInformation(
    mut InfoBuffer: PVOID,
    mut FileInformationClass: FILE_INFORMATION_CLASS,
    mut BufferLength: PSIZE_T,
) -> NTSTATUS {
    let mut StandardInformation: PFILE_STANDARD_INFORMATION = core::ptr::null_mut();
    if FileInformationClass as u32 != FileStandardInformation as u32 {
        return STATUS_INVALID_INFO_CLASS;
    }
    if *BufferLength < core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _ {
        *BufferLength = core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _;
        return STATUS_INFO_LENGTH_MISMATCH;
    }
    StandardInformation = InfoBuffer as PFILE_STANDARD_INFORMATION;
    (*StandardInformation).AllocationSize.QuadPart = 0;
    (*StandardInformation).EndOfFile.QuadPart = 0;
    (*StandardInformation).NumberOfLinks = 1;
    (*StandardInformation).DeletePending = 0;
    (*StandardInformation).Directory = 0;
    *BufferLength = core::mem::size_of::<FILE_STANDARD_INFORMATION>() as _;
    return STATUS_SUCCESS;
}
unsafe fn KsecQueryVolumeInformation(
    InfoBuffer: PVOID,
    FsInformationClass: FS_INFORMATION_CLASS,
    BufferLength: PSIZE_T,
) -> NTSTATUS {
    const FILE_DEVICE_NULL: ULONG = 0x00000015;

    let mut DeviceInformation: PFILE_FS_DEVICE_INFORMATION = core::ptr::null_mut();

    /* Only FileFsDeviceInformation is supported */
    if FsInformationClass as u32 != FileFsDeviceInformation as u32 {
        return STATUS_INVALID_INFO_CLASS;
    }

    /* Validate buffer size */
    if *BufferLength < core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as _ {
        *BufferLength = core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as _;
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    /* Fill the structure */
    DeviceInformation = InfoBuffer as PFILE_FS_DEVICE_INFORMATION;
    (*DeviceInformation).DeviceType = FILE_DEVICE_NULL;
    (*DeviceInformation).Characteristics = 0;
    *BufferLength = core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as _;

    return STATUS_SUCCESS;
}

const IOCTL_KSEC_REGISTER_LSA_PROCESS: u32 = 3768320;
const IOCTL_KSEC_RANDOM_FILL_BUFFER: u32 = 3735560;
const IOCTL_KSEC_ENCRYPT_SAME_PROCESS: u32 = 3735566;
const IOCTL_KSEC_DECRYPT_SAME_PROCESS: u32 = 3735570;
const IOCTL_KSEC_ENCRYPT_CROSS_PROCESS: u32 = 3735574;
const IOCTL_KSEC_DECRYPT_CROSS_PROCESS: u32 = 3735578;
const IOCTL_KSEC_ENCRYPT_SAME_LOGON: u32 = 3735582;
const IOCTL_KSEC_DECRYPT_SAME_LOGON: u32 = 3735586;
const RTL_ENCRYPT_OPTION_SAME_PROCESS: u32 = 0;
const RTL_ENCRYPT_OPTION_CROSS_PROCESS: u32 = 1;
const RTL_ENCRYPT_OPTION_SAME_LOGON: u32 = 2;
unsafe fn KsecDeviceControl(
    IoControlCode: ULONG,
    Buffer: PVOID,
    InputLength: SIZE_T,
    OutputLength: PSIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if (IoControlCode == IOCTL_KSEC_RANDOM_FILL_BUFFER)
        || (IoControlCode == IOCTL_KSEC_ENCRYPT_SAME_PROCESS)
        || (IoControlCode == IOCTL_KSEC_DECRYPT_SAME_PROCESS)
        || (IoControlCode == IOCTL_KSEC_ENCRYPT_CROSS_PROCESS)
        || (IoControlCode == IOCTL_KSEC_DECRYPT_CROSS_PROCESS)
        || (IoControlCode == IOCTL_KSEC_ENCRYPT_SAME_LOGON)
        || (IoControlCode == IOCTL_KSEC_DECRYPT_SAME_LOGON)
    {
        /* Make sure we have a valid output buffer */
        if Buffer.is_null() || OutputLength.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        /* Check if the input is smaller than the output */
        if InputLength < *OutputLength {
            /* We might have uninitialized memory, zero it out */
            RtlSecureZeroMemory(
                (Buffer as PUCHAR).offset(InputLength as isize) as PVOID,
                (*OutputLength).wrapping_sub(InputLength),
            );
        }
    }

    /* Check ioctl code */
    match IoControlCode {
        IOCTL_KSEC_REGISTER_LSA_PROCESS => {
            Status = STATUS_SUCCESS;
        }
        IOCTL_KSEC_RANDOM_FILL_BUFFER => {
            Status = KsecGenRandom(Buffer, *OutputLength);
        }
        IOCTL_KSEC_ENCRYPT_SAME_PROCESS => {
            Status = KsecEncryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_SAME_PROCESS as ULONG,
            );
        }
        IOCTL_KSEC_DECRYPT_SAME_PROCESS => {
            Status = KsecDecryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_SAME_PROCESS as ULONG,
            );
        }
        IOCTL_KSEC_ENCRYPT_CROSS_PROCESS => {
            Status = KsecEncryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_CROSS_PROCESS as ULONG,
            );
        }
        IOCTL_KSEC_DECRYPT_CROSS_PROCESS => {
            Status = KsecDecryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_CROSS_PROCESS as ULONG,
            );
        }
        IOCTL_KSEC_ENCRYPT_SAME_LOGON => {
            Status = KsecEncryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_SAME_LOGON as ULONG,
            );
        }
        IOCTL_KSEC_DECRYPT_SAME_LOGON => {
            Status = KsecDecryptMemory(
                Buffer,
                *OutputLength as ULONG,
                RTL_ENCRYPT_OPTION_SAME_LOGON as ULONG,
            );
        }
        _ => {
            if DbgPrint(
                b"(%s:%d) Unhandled control code 0x%lx\n\0" as *const u8 as PCSTR,
                b"drivers/crypto/ksecdd/dispatch.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                174 as ::core::ffi::c_int,
                IoControlCode,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"drivers/crypto/ksecdd/dispatch.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    174 as ::core::ffi::c_int,
                );
            }
            __debugbreak();
            return STATUS_INVALID_PARAMETER;
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KsecDdDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStackLocation: PIO_STACK_LOCATION;
    // c2rust's goto-emulation (`current_block_33` magic-number match) defeats the
    // borrow-checker's init analysis even though every reachable path assigns both
    // before the read at the tail. Initialize to 0 to satisfy the checker without
    // changing behavior (the C declares these uninitialized and assigns in every arm).
    let mut Information: ULONG_PTR = 0;
    let mut Status: NTSTATUS = 0;
    let mut Buffer: PVOID;
    let mut InputLength: SIZE_T = 0;
    let mut OutputLength: SIZE_T;
    let mut FileInfoClass: FILE_INFORMATION_CLASS;
    let mut FsInfoClass: FS_INFORMATION_CLASS;
    let mut IoControlCode: ULONG;

    IoStackLocation = IoGetCurrentIrpStackLocation(Irp);

    let mut current_block_33: u64;
    match (*IoStackLocation).MajorFunction as u8 {
        IRP_MJ_CREATE | IRP_MJ_CLOSE => {
            /* Just return success */
            Status = STATUS_SUCCESS;
            Information = 0 as ULONG_PTR;
        }
        IRP_MJ_READ => {
            /* There is nothing to read */
            Status = STATUS_END_OF_FILE;
            Information = 0 as ULONG_PTR;
        }
        IRP_MJ_WRITE => {
            /* Pretend to have written everything */
            Status = STATUS_SUCCESS;
            Information = (*IoStackLocation).Parameters.Write.Length as ULONG_PTR;
        }
        IRP_MJ_QUERY_INFORMATION => {
            /* Extract the parameters */
            Buffer = (*Irp).AssociatedIrp.SystemBuffer;
            OutputLength = (*IoStackLocation).Parameters.QueryFile.Length as SIZE_T;
            FileInfoClass = (*IoStackLocation).Parameters.QueryFile.FileInformationClass;

            /* Call the internal function */
            Status = KsecQueryFileInformation(
                Buffer,
                FileInfoClass,
                core::ptr::addr_of_mut!(OutputLength),
            );
            Information = OutputLength as ULONG_PTR;
        }
        IRP_MJ_QUERY_VOLUME_INFORMATION => {
            /* Extract the parameters */
            Buffer = (*Irp).AssociatedIrp.SystemBuffer;
            OutputLength = (*IoStackLocation).Parameters.QueryVolume.Length as SIZE_T;
            FsInfoClass = (*IoStackLocation).Parameters.QueryVolume.FsInformationClass;

            /* Call the internal function */
            Status = KsecQueryVolumeInformation(
                Buffer,
                FsInfoClass,
                core::ptr::addr_of_mut!(OutputLength),
            );
            Information = OutputLength as ULONG_PTR;
        }
        IRP_MJ_DEVICE_CONTROL => {
            /* Extract the parameters */
            InputLength = (*IoStackLocation).Parameters.DeviceIoControl.InputBufferLength as SIZE_T;
            OutputLength = (*IoStackLocation).Parameters.DeviceIoControl.OutputBufferLength as SIZE_T;
            IoControlCode = (*IoStackLocation).Parameters.DeviceIoControl.IoControlCode;

            /* Check for METHOD_OUT_DIRECT method */
            if (IoControlCode & 3) == METHOD_OUT_DIRECT && OutputLength != 0 {
                /* Use the provided MDL */
                OutputLength = (*(*Irp).MdlAddress).ByteCount as SIZE_T;
                Buffer = MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority) as PVOID;
                if Buffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    Information = 0 as ULONG_PTR;
                    current_block_33 = 18153031941552419006;
                } else {
                    current_block_33 = 14818589718467733107;
                }
            } else {
                /* Otherwise this is METHOD_BUFFERED, use the SystemBuffer */
                Buffer = (*Irp).AssociatedIrp.SystemBuffer;
                current_block_33 = 14818589718467733107;
            }
            match current_block_33 {
                18153031941552419006 => {}
                _ => {
                    /* Call the internal function */
                    Status = KsecDeviceControl(
                        IoControlCode,
                        Buffer,
                        InputLength,
                        core::ptr::addr_of_mut!(OutputLength),
                    );
                    Information = OutputLength as ULONG_PTR;
                }
            }
        }
        _ => {
            // DPRINT1("Unhandled major function %lu!\n", IoStackLocation->MajorFunction);
            // ASSERT(FALSE);
            return STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    /* Return the information */
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = Information;

    /* Complete the request */
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
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
// #include <ksecioctl.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS ******************************************************************/
//
// static
// NTSTATUS
// KsecQueryFileInformation(
//     PVOID InfoBuffer,
//     FILE_INFORMATION_CLASS FileInformationClass,
//     PSIZE_T BufferLength)
// {
//     PFILE_STANDARD_INFORMATION StandardInformation;
//
//     /* Only FileStandardInformation is supported */
//     if (FileInformationClass != FileStandardInformation)
//     {
//         return STATUS_INVALID_INFO_CLASS;
//     }
//
//     /* Validate buffer size */
//     if (*BufferLength < sizeof(FILE_STANDARD_INFORMATION))
//     {
//         *BufferLength = sizeof(FILE_STANDARD_INFORMATION);
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     /* Fill the structure */
//     StandardInformation = (PFILE_STANDARD_INFORMATION)InfoBuffer;
//     StandardInformation->AllocationSize.QuadPart = 0;
//     StandardInformation->EndOfFile.QuadPart = 0;
//     StandardInformation->NumberOfLinks = 1;
//     StandardInformation->DeletePending = FALSE;
//     StandardInformation->Directory = FALSE;
//     *BufferLength = sizeof(FILE_STANDARD_INFORMATION);
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// KsecQueryVolumeInformation(
//     PVOID InfoBuffer,
//     FS_INFORMATION_CLASS FsInformationClass,
//     PSIZE_T BufferLength)
// {
//     PFILE_FS_DEVICE_INFORMATION DeviceInformation;
//
//     /* Only FileFsDeviceInformation is supported */
//     if (FsInformationClass != FileFsDeviceInformation)
//     {
//         return STATUS_INVALID_INFO_CLASS;
//     }
//
//     /* Validate buffer size */
//     if (*BufferLength < sizeof(FILE_FS_DEVICE_INFORMATION))
//     {
//         *BufferLength = sizeof(FILE_FS_DEVICE_INFORMATION);
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     /* Fill the structure */
//     DeviceInformation = (PFILE_FS_DEVICE_INFORMATION)InfoBuffer;
//     DeviceInformation->DeviceType = FILE_DEVICE_NULL;
//     DeviceInformation->Characteristics = 0;
//     *BufferLength = sizeof(FILE_FS_DEVICE_INFORMATION);
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// KsecDeviceControl(
//     ULONG IoControlCode,
//     PVOID Buffer,
//     SIZE_T InputLength,
//     PSIZE_T OutputLength)
// {
//     NTSTATUS Status;
//
//     if ((IoControlCode == IOCTL_KSEC_RANDOM_FILL_BUFFER) ||
//         (IoControlCode == IOCTL_KSEC_ENCRYPT_SAME_PROCESS) ||
//         (IoControlCode == IOCTL_KSEC_DECRYPT_SAME_PROCESS) ||
//         (IoControlCode == IOCTL_KSEC_ENCRYPT_CROSS_PROCESS) ||
//         (IoControlCode == IOCTL_KSEC_DECRYPT_CROSS_PROCESS) ||
//         (IoControlCode == IOCTL_KSEC_ENCRYPT_SAME_LOGON) ||
//         (IoControlCode == IOCTL_KSEC_DECRYPT_SAME_LOGON))
//     {
//         /* Make sure we have a valid output buffer */
//         if ((Buffer == NULL) || (OutputLength == NULL))
//         {
//             return STATUS_INVALID_PARAMETER;
//         }
//
//         /* Check if the input is smaller than the output */
//         if (InputLength < *OutputLength)
//         {
//             /* We might have uninitialized memory, zero it out */
//             RtlSecureZeroMemory((PUCHAR)Buffer + InputLength,
//                                 *OutputLength - InputLength);
//         }
//     }
//
//     /* Check ioctl code */
//     switch (IoControlCode)
//     {
//         case IOCTL_KSEC_REGISTER_LSA_PROCESS:
//
//             Status = STATUS_SUCCESS;
//             break;
//
//         case IOCTL_KSEC_RANDOM_FILL_BUFFER:
//
//             Status = KsecGenRandom(Buffer, *OutputLength);
//             break;
//
//         case IOCTL_KSEC_ENCRYPT_SAME_PROCESS:
//
//             Status = KsecEncryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_SAME_PROCESS);
//             break;
//
//         case IOCTL_KSEC_DECRYPT_SAME_PROCESS:
//
//             Status = KsecDecryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_SAME_PROCESS);
//             break;
//
//         case IOCTL_KSEC_ENCRYPT_CROSS_PROCESS:
//
//             Status = KsecEncryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_CROSS_PROCESS);
//             break;
//
//         case IOCTL_KSEC_DECRYPT_CROSS_PROCESS:
//
//             Status = KsecDecryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_CROSS_PROCESS);
//             break;
//
//         case IOCTL_KSEC_ENCRYPT_SAME_LOGON:
//
//             Status = KsecEncryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_SAME_LOGON);
//             break;
//
//         case IOCTL_KSEC_DECRYPT_SAME_LOGON:
//
//             Status = KsecDecryptMemory(Buffer,
//                                        *OutputLength,
//                                        RTL_ENCRYPT_OPTION_SAME_LOGON);
//             break;
//
//         default:
//             DPRINT1("Unhandled control code 0x%lx\n", IoControlCode);
//             __debugbreak();
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// KsecDdDispatch(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStackLocation;
//     ULONG_PTR Information;
//     NTSTATUS Status;
//     PVOID Buffer;
//     SIZE_T InputLength, OutputLength;
//     FILE_INFORMATION_CLASS FileInfoClass;
//     FS_INFORMATION_CLASS FsInfoClass;
//     ULONG IoControlCode;
//
//     IoStackLocation = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (IoStackLocation->MajorFunction)
//     {
//         case IRP_MJ_CREATE:
//         case IRP_MJ_CLOSE:
//
//             /* Just return success */
//             Status = STATUS_SUCCESS;
//             Information = 0;
//             break;
//
//         case IRP_MJ_READ:
//
//             /* There is nothing to read */
//             Status = STATUS_END_OF_FILE;
//             Information = 0;
//             break;
//
//         case IRP_MJ_WRITE:
//
//             /* Pretend to have written everything */
//             Status = STATUS_SUCCESS;
//             Information = IoStackLocation->Parameters.Write.Length;
//             break;
//
//         case IRP_MJ_QUERY_INFORMATION:
//
//             /* Extract the parameters */
//             Buffer = Irp->AssociatedIrp.SystemBuffer;
//             OutputLength = IoStackLocation->Parameters.QueryFile.Length;
//             FileInfoClass = IoStackLocation->Parameters.QueryFile.FileInformationClass;
//
//             /* Call the internal function */
//             Status = KsecQueryFileInformation(Buffer,
//                                               FileInfoClass,
//                                               &OutputLength);
//             Information = OutputLength;
//             break;
//
//         case IRP_MJ_QUERY_VOLUME_INFORMATION:
//
//             /* Extract the parameters */
//             Buffer = Irp->AssociatedIrp.SystemBuffer;
//             OutputLength = IoStackLocation->Parameters.QueryVolume.Length;
//             FsInfoClass = IoStackLocation->Parameters.QueryVolume.FsInformationClass;
//
//             /* Call the internal function */
//             Status = KsecQueryVolumeInformation(Buffer,
//                                                 FsInfoClass,
//                                                 &OutputLength);
//             Information = OutputLength;
//             break;
//
//         case IRP_MJ_DEVICE_CONTROL:
//
//             /* Extract the parameters */
//             InputLength = IoStackLocation->Parameters.DeviceIoControl.InputBufferLength;
//             OutputLength = IoStackLocation->Parameters.DeviceIoControl.OutputBufferLength;
//             IoControlCode = IoStackLocation->Parameters.DeviceIoControl.IoControlCode;
//
//             /* Check for METHOD_OUT_DIRECT method */
//             if ((METHOD_FROM_CTL_CODE(IoControlCode) == METHOD_OUT_DIRECT) &&
//                 (OutputLength != 0))
//             {
//                 /* Use the provided MDL */
//                 OutputLength = Irp->MdlAddress->ByteCount;
//                 Buffer = MmGetSystemAddressForMdlSafe(Irp->MdlAddress,
//                                                       NormalPagePriority);
//                 if (Buffer == NULL)
//                 {
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     Information = 0;
//                     break;
//                 }
//             }
//             else
//             {
//                 /* Otherwise this is METHOD_BUFFERED, use the SystemBuffer */
//                 Buffer = Irp->AssociatedIrp.SystemBuffer;
//             }
//
//             /* Call the internal function */
//             Status = KsecDeviceControl(IoControlCode,
//                                        Buffer,
//                                        InputLength,
//                                        &OutputLength);
//             Information = OutputLength;
//             break;
//
//         default:
//             DPRINT1("Unhandled major function %lu!\n",
//                     IoStackLocation->MajorFunction);
//             ASSERT(FALSE);
//             return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Return the information */
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = Information;
//
//     /* Complete the request */
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
