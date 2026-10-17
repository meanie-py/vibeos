use ntdll_sys::nt::*;
// The crate's own functions: the definition is the declaration.
use crate::client::handle::CloseHandle;
use crate::prelude::*;
use core::ffi::c_void;

// ---- cohab glue: kernel32 Rust exports this module calls (real stdcall @N exports) ----
extern "system" {
    fn CreateEventA(lpEventAttributes: *mut c_void, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCSTR) -> HANDLE;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}

// ---- cohab glue: IOCTL_TAPE_* control codes (winioctl.h) ----
//   CTL_CODE(DeviceType, Function, Method, Access) macro, expanded faithfully.
const fn CTL_CODE(device: ULONG, function: ULONG, method: ULONG, access: ULONG) -> ULONG {
    (device << 16) | (access << 14) | (function << 2) | method
}
const FILE_DEVICE_TAPE: ULONG = 31;       // IOCTL_TAPE_BASE
const METHOD_BUFFERED:  ULONG = 0;
const FILE_READ_ACCESS: ULONG = 0x0001;
const FILE_WRITE_ACCESS: ULONG = 0x0002;
const RW: ULONG = FILE_READ_ACCESS | FILE_WRITE_ACCESS;
const R: ULONG = FILE_READ_ACCESS;

pub const IOCTL_TAPE_ERASE:            ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0000, METHOD_BUFFERED, RW);
pub const IOCTL_TAPE_PREPARE:          ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0001, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_WRITE_MARKS:      ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0002, METHOD_BUFFERED, RW);
pub const IOCTL_TAPE_GET_POSITION:     ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0003, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_SET_POSITION:     ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0004, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_GET_DRIVE_PARAMS: ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0005, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_SET_DRIVE_PARAMS: ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0006, METHOD_BUFFERED, RW);
pub const IOCTL_TAPE_GET_MEDIA_PARAMS: ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0007, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_SET_MEDIA_PARAMS: ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0008, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_GET_STATUS:       ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x0009, METHOD_BUFFERED, R);
pub const IOCTL_TAPE_CREATE_PARTITION: ULONG = CTL_CODE(FILE_DEVICE_TAPE, 0x000a, METHOD_BUFFERED, RW);

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn BasepDoTapeOperation(
    DeviceHandle: HANDLE,
    Ioctl: ULONG,
    Input: PVOID,
    InputLength: ULONG,
    Output: PVOID,
    OutputLength: ULONG,
) -> DWORD {
    let mut TapeEvent: HANDLE;
    let mut ErrorCode: DWORD;
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    TapeEvent = CreateEventA(core::ptr::null_mut(), 0, 0, core::ptr::null());
    if TapeEvent.is_null() {
        return GetLastError();
    }

    Status = NtDeviceIoControlFile(
        DeviceHandle,
        TapeEvent,
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        Ioctl,
        Input,
        InputLength,
        Output,
        OutputLength,
    );
    if Status == STATUS_PENDING {
        WaitForSingleObject(TapeEvent, INFINITE);
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    CloseHandle(TapeEvent);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        ErrorCode = GetLastError();
    } else {
        ErrorCode = ERROR_SUCCESS;
    }

    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn CreateTapePartition(
    hDevice: HANDLE,
    dwPartitionMethod: DWORD,
    dwCount: DWORD,
    dwSize: DWORD,
) -> DWORD {
    let mut TapeCreatePartition: TAPE_CREATE_PARTITION = TAPE_CREATE_PARTITION {
        Method: 0,
        Count: 0,
        Size: 0,
    };
    TapeCreatePartition.Method = dwPartitionMethod;
    TapeCreatePartition.Count = dwCount;
    TapeCreatePartition.Size = dwSize;
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_CREATE_PARTITION as ULONG,
        &raw mut TapeCreatePartition as PVOID,
        core::mem::size_of::<TAPE_CREATE_PARTITION>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EraseTape(
    hDevice: HANDLE,
    dwEraseType: DWORD,
    bImmediate: BOOL,
) -> DWORD {
    let mut TapeErase: TAPE_ERASE = core::mem::zeroed();
    TapeErase.Type = dwEraseType;
    TapeErase.Immediate = bImmediate as BOOLEAN;
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_ERASE,
        &mut TapeErase as *mut TAPE_ERASE as PVOID,
        core::mem::size_of::<TAPE_ERASE>() as ULONG,
        core::ptr::null_mut(),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetTapeParameters(
    hDevice: HANDLE,
    dwOperation: DWORD,
    lpdwSize: LPDWORD,
    lpTapeInformation: LPVOID,
) -> DWORD {
    if dwOperation == GET_TAPE_MEDIA_INFORMATION as DWORD {
        if (*lpdwSize as usize) < core::mem::size_of::<TAPE_GET_MEDIA_PARAMETERS>() as usize {
            *lpdwSize = core::mem::size_of::<TAPE_GET_MEDIA_PARAMETERS>() as DWORD;
            return ERROR_MORE_DATA as DWORD;
        }

        return BasepDoTapeOperation(
            hDevice,
            IOCTL_TAPE_GET_MEDIA_PARAMS as ULONG,
            core::ptr::null_mut(),
            0 as ULONG,
            lpTapeInformation as LPVOID,
            core::mem::size_of::<TAPE_GET_MEDIA_PARAMETERS>() as ULONG,
        );
    } else if dwOperation == GET_TAPE_DRIVE_INFORMATION as DWORD {
        if (*lpdwSize as usize) < core::mem::size_of::<TAPE_GET_DRIVE_PARAMETERS>() as usize {
            *lpdwSize = core::mem::size_of::<TAPE_GET_DRIVE_PARAMETERS>() as DWORD;
            return ERROR_MORE_DATA as DWORD;
        }

        return BasepDoTapeOperation(
            hDevice,
            IOCTL_TAPE_GET_DRIVE_PARAMS as ULONG,
            core::ptr::null_mut(),
            0 as ULONG,
            lpTapeInformation as LPVOID,
            core::mem::size_of::<TAPE_GET_DRIVE_PARAMETERS>() as ULONG,
        );
    }

    return ERROR_INVALID_FUNCTION as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetTapePosition(
    hDevice: HANDLE,
    dwPositionType: DWORD,
    lpdwPartition: LPDWORD,
    lpdwOffsetLow: LPDWORD,
    lpdwOffsetHigh: LPDWORD,
) -> DWORD {
    let mut TapeGetPosition: TAPE_GET_POSITION = core::mem::zeroed();
    let mut Result: DWORD;

    TapeGetPosition.Type = dwPositionType;
    Result = BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_GET_POSITION,
        &mut TapeGetPosition as *mut _ as PVOID,
        core::mem::size_of::<TAPE_GET_POSITION>() as ULONG,
        &mut TapeGetPosition as *mut _ as PVOID,
        core::mem::size_of::<TAPE_GET_POSITION>() as ULONG,
    );

    if Result != 0 {
        *lpdwPartition = 0;
        *lpdwOffsetLow = 0;
        *lpdwOffsetHigh = 0;
    } else {
        *lpdwPartition = TapeGetPosition.Partition;
        *lpdwOffsetLow = TapeGetPosition.Offset.u.LowPart;
        *lpdwOffsetHigh = TapeGetPosition.Offset.u.HighPart as DWORD;
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn GetTapeStatus(hDevice: HANDLE) -> DWORD {
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_GET_STATUS as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PrepareTape(
    hDevice: HANDLE,
    dwOperation: DWORD,
    bImmediate: BOOL,
) -> DWORD {
    let mut TapePrepare: TAPE_PREPARE = TAPE_PREPARE {
        Operation: 0,
        Immediate: 0,
    };
    TapePrepare.Operation = dwOperation;
    TapePrepare.Immediate = bImmediate as BOOLEAN;
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_PREPARE as ULONG,
        &raw mut TapePrepare as PVOID,
        core::mem::size_of::<TAPE_PREPARE>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetTapeParameters(
    hDevice: HANDLE,
    dwOperation: DWORD,
    lpTapeInformation: LPVOID,
) -> DWORD {
    if dwOperation == SET_TAPE_MEDIA_INFORMATION as DWORD {
        return BasepDoTapeOperation(
            hDevice,
            IOCTL_TAPE_SET_MEDIA_PARAMS as ULONG,
            lpTapeInformation as PVOID,
            core::mem::size_of::<TAPE_SET_MEDIA_PARAMETERS>() as ULONG,
            core::ptr::null_mut(),
            0 as ULONG,
        );
    } else if dwOperation == SET_TAPE_DRIVE_INFORMATION as DWORD {
        return BasepDoTapeOperation(
            hDevice,
            IOCTL_TAPE_SET_DRIVE_PARAMS as ULONG,
            lpTapeInformation as PVOID,
            core::mem::size_of::<TAPE_SET_DRIVE_PARAMETERS>() as ULONG,
            core::ptr::null_mut(),
            0 as ULONG,
        );
    }

    return ERROR_INVALID_FUNCTION as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn SetTapePosition(
    hDevice: HANDLE,
    dwPositionMethod: DWORD,
    dwPartition: DWORD,
    dwOffsetLow: DWORD,
    dwOffsetHigh: DWORD,
    bImmediate: BOOL,
) -> DWORD {
    let mut TapeSetPosition: TAPE_SET_POSITION = core::mem::zeroed();

    TapeSetPosition.Method = dwPositionMethod;
    TapeSetPosition.Partition = dwPartition;
    TapeSetPosition.Offset.u.LowPart = dwOffsetLow;
    TapeSetPosition.Offset.u.HighPart = dwOffsetHigh as LONG;
    TapeSetPosition.Immediate = bImmediate as BOOLEAN;
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_SET_POSITION as ULONG,
        &raw mut TapeSetPosition as PVOID,
        core::mem::size_of::<TAPE_SET_POSITION>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WriteTapemark(
    hDevice: HANDLE,
    dwTapemarkType: DWORD,
    dwTapemarkCount: DWORD,
    bImmediate: BOOL,
) -> DWORD {
    let mut TapeWriteMarks: TAPE_WRITE_MARKS = TAPE_WRITE_MARKS {
        Type: 0,
        Count: 0,
        Immediate: 0,
    };
    TapeWriteMarks.Type = dwTapemarkType;
    TapeWriteMarks.Count = dwTapemarkCount;
    TapeWriteMarks.Immediate = bImmediate as BOOLEAN;
    return BasepDoTapeOperation(
        hDevice,
        IOCTL_TAPE_WRITE_MARKS as ULONG,
        &raw mut TapeWriteMarks as PVOID,
        core::mem::size_of::<TAPE_WRITE_MARKS>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/tape.c
//  * PURPOSE:         Tape functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// DWORD
// WINAPI
// BasepDoTapeOperation(IN HANDLE DeviceHandle,
//                      IN ULONG Ioctl,
//                      IN PVOID Input,
//                      IN ULONG InputLength,
//                      IN PVOID Output,
//                      IN ULONG OutputLength)
// {
//     HANDLE TapeEvent;
//     DWORD ErrorCode;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Create the wait event */
//     TapeEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
//     if (!TapeEvent) return GetLastError();
//
//     /* Send the IOCTL */
//     Status = NtDeviceIoControlFile(DeviceHandle,
//                                    TapeEvent,
//                                    0,
//                                    0,
//                                    &IoStatusBlock,
//                                    Ioctl,
//                                    Input,
//                                    InputLength,
//                                    Output,
//                                    OutputLength);
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for its completion */
//         WaitForSingleObject(TapeEvent, INFINITE);
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Get rid of the wait event and check status */
//     CloseHandle(TapeEvent);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Convert to Win32 */
//         BaseSetLastNTError(Status);
//         ErrorCode = GetLastError();
//     }
//     else
//     {
//         /* Set success */
//         ErrorCode = ERROR_SUCCESS;
//     }
//
//     /* Return the Win32 error code */
//     return ErrorCode;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CreateTapePartition(IN HANDLE hDevice,
//                     IN DWORD dwPartitionMethod,
//                     IN DWORD dwCount,
//                     IN DWORD dwSize)
// {
//     TAPE_CREATE_PARTITION TapeCreatePartition;
//
//     TapeCreatePartition.Method = dwPartitionMethod;
//     TapeCreatePartition.Count = dwCount;
//     TapeCreatePartition.Size = dwSize;
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_CREATE_PARTITION,
//                                 &TapeCreatePartition,
//                                 sizeof(TapeCreatePartition),
//                                 NULL,
//                                 0);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// EraseTape(IN HANDLE hDevice,
//           IN DWORD dwEraseType,
//           IN BOOL bImmediate)
// {
//     TAPE_ERASE TapeErase;
//
//     TapeErase.Type = dwEraseType;
//     TapeErase.Immediate = (BOOLEAN)bImmediate;
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_ERASE,
//                                 &TapeErase,
//                                 sizeof(TapeErase),
//                                 NULL,
//                                 0);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTapeParameters(IN HANDLE hDevice,
//                   IN DWORD dwOperation,
//                   IN LPDWORD lpdwSize,
//                   IN LPVOID lpTapeInformation)
// {
//     if (dwOperation == GET_TAPE_MEDIA_INFORMATION)
//     {
//         if (*lpdwSize < sizeof(TAPE_GET_MEDIA_PARAMETERS))
//         {
//             *lpdwSize = sizeof(TAPE_GET_MEDIA_PARAMETERS);
//             return ERROR_MORE_DATA;
//         }
//
//         return BasepDoTapeOperation(hDevice,
//                                     IOCTL_TAPE_GET_MEDIA_PARAMS,
//                                     NULL,
//                                     0,
//                                     lpTapeInformation,
//                                     sizeof(TAPE_GET_MEDIA_PARAMETERS));
//     }
//     else if (dwOperation == GET_TAPE_DRIVE_INFORMATION)
//     {
//         if (*lpdwSize < sizeof(TAPE_GET_DRIVE_PARAMETERS))
//         {
//             *lpdwSize = sizeof(TAPE_GET_DRIVE_PARAMETERS);
//             return ERROR_MORE_DATA;
//         }
//
//         return BasepDoTapeOperation(hDevice,
//                                     IOCTL_TAPE_GET_DRIVE_PARAMS,
//                                     NULL,
//                                     0,
//                                     lpTapeInformation,
//                                     sizeof(TAPE_GET_DRIVE_PARAMETERS));
//     }
//
//     return ERROR_INVALID_FUNCTION;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTapePosition(IN HANDLE hDevice,
//                 IN DWORD dwPositionType,
//                 IN LPDWORD lpdwPartition,
//                 IN LPDWORD lpdwOffsetLow,
//                 IN LPDWORD lpdwOffsetHigh)
// {
//     TAPE_GET_POSITION TapeGetPosition;
//     DWORD Result;
//
//     TapeGetPosition.Type = dwPositionType;
//     Result = BasepDoTapeOperation(hDevice,
//                                   IOCTL_TAPE_GET_POSITION,
//                                   &TapeGetPosition,
//                                   sizeof(TapeGetPosition),
//                                   &TapeGetPosition,
//                                   sizeof(TapeGetPosition));
//
//     if (Result)
//     {
//         *lpdwPartition = 0;
//         *lpdwOffsetLow = 0;
//         *lpdwOffsetHigh = 0;
//     }
//     else
//     {
//         *lpdwPartition = TapeGetPosition.Partition;
//         *lpdwOffsetLow = TapeGetPosition.Offset.u.LowPart;
//         *lpdwOffsetHigh = TapeGetPosition.Offset.u.HighPart as DWORD;
//     }
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTapeStatus(IN HANDLE hDevice)
// {
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_GET_STATUS,
//                                 NULL,
//                                 0,
//                                 NULL,
//                                 0);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// PrepareTape(IN HANDLE hDevice,
//             IN DWORD dwOperation,
//             IN BOOL bImmediate)
// {
//     TAPE_PREPARE TapePrepare;
//
//     TapePrepare.Operation = dwOperation;
//     TapePrepare.Immediate = (BOOLEAN)bImmediate;
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_PREPARE,
//                                 &TapePrepare,
//                                 sizeof(TapePrepare),
//                                 NULL,
//                                 0);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetTapeParameters(IN HANDLE hDevice,
//                   IN DWORD dwOperation,
//                   IN LPVOID lpTapeInformation)
// {
//     if (dwOperation == SET_TAPE_MEDIA_INFORMATION)
//     {
//         return BasepDoTapeOperation(hDevice,
//                                     IOCTL_TAPE_SET_MEDIA_PARAMS,
//                                     lpTapeInformation,
//                                     sizeof(TAPE_SET_MEDIA_PARAMETERS),
//                                     NULL,
//                                     0);
//     }
//     else if (dwOperation == SET_TAPE_DRIVE_INFORMATION)
//     {
//         return BasepDoTapeOperation(hDevice,
//                                     IOCTL_TAPE_SET_DRIVE_PARAMS,
//                                     lpTapeInformation,
//                                     sizeof(TAPE_SET_DRIVE_PARAMETERS),
//                                     NULL,
//                                     0);
//     }
//
//     return ERROR_INVALID_FUNCTION;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetTapePosition(IN HANDLE hDevice,
//                 IN DWORD dwPositionMethod,
//                 IN DWORD dwPartition,
//                 IN DWORD dwOffsetLow,
//                 IN DWORD dwOffsetHigh,
//                 IN BOOL bImmediate)
// {
//     TAPE_SET_POSITION TapeSetPosition;
//
//     TapeSetPosition.Method = dwPositionMethod;
//     TapeSetPosition.Partition = dwPartition;
//     TapeSetPosition.Offset.u.LowPart = dwOffsetLow;
//     TapeSetPosition.Offset.u.HighPart = dwOffsetHigh;
//     TapeSetPosition.Immediate = (BOOLEAN)bImmediate;
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_SET_POSITION,
//                                 &TapeSetPosition,
//                                 sizeof(TapeSetPosition),
//                                 NULL,
//                                 0);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WriteTapemark(IN HANDLE hDevice,
//               IN DWORD dwTapemarkType,
//               IN DWORD dwTapemarkCount,
//               IN BOOL bImmediate)
// {
//     TAPE_WRITE_MARKS TapeWriteMarks;
//
//     TapeWriteMarks.Type = dwTapemarkType;
//     TapeWriteMarks.Count = dwTapemarkCount;
//     TapeWriteMarks.Immediate = (BOOLEAN)bImmediate;
//     return BasepDoTapeOperation(hDevice,
//                                 IOCTL_TAPE_WRITE_MARKS,
//                                 &TapeWriteMarks,
//                                 sizeof(TapeWriteMarks),
//                                 NULL,
//                                 0);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
