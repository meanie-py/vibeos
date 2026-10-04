use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{CSR_CREATE_API_NUMBER, InitializeObjectAttributes};

// ntddbeep.h
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BEEP_SET_PARAMETERS {
    pub Frequency: ULONG,
    pub Duration: ULONG,
}

// IOCTL_BEEP_SET = CTL_CODE(FILE_DEVICE_BEEP, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_BEEP_SET: ULONG = (FILE_DEVICE_BEEP << 16) | (0 << 14) | (0 << 2) | 0;

// NT_ERROR(Status) == ((ULONG)(Status) >> 30) == 3
#[inline]
pub fn NT_ERROR(status: NTSTATUS) -> bool {
    ((status as ULONG) >> 30) == 3
}

extern "system" {
    fn GetConsoleDisplayMode(lpModeFlags: LPDWORD) -> BOOL;
    fn SleepEx(dwMilliseconds: DWORD, bAlertable: BOOL) -> DWORD;
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn NotifySoundSentry() {
    let mut ApiMessage: BASE_API_MESSAGE = core::mem::zeroed();
    let SoundSentryRequest: PBASE_SOUND_SENTRY = &mut ApiMessage.Data.SoundSentryRequest;

    /* Get the video mode */
    if GetConsoleDisplayMode(&mut (*SoundSentryRequest).VideoMode) == 0 {
        (*SoundSentryRequest).VideoMode = 0;
    }

    /* Make sure it's not fullscreen, and send the message if not */
    if (*SoundSentryRequest).VideoMode == 0 {
        CsrClientCallServer(
            &mut ApiMessage as *mut BASE_API_MESSAGE as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSoundSentryNotification),
            core::mem::size_of::<BASE_SOUND_SENTRY>() as ULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn Beep(
    dwFreq: DWORD,
    dwDuration: DWORD,
) -> BOOL {
    let mut hBeep: HANDLE = core::ptr::null_mut();
    let mut BeepDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BeepSetParameters: BEEP_SET_PARAMETERS = core::mem::zeroed();
    let mut Status: NTSTATUS;

    static BEEP_DEVICE_NAME: [u16; 13] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16, b'\\' as u16, b'B' as u16, b'e' as u16, b'e' as u16, b'p' as u16, 0,
    ];
    RtlInitUnicodeString(&mut BeepDevice, BEEP_DEVICE_NAME.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut BeepDevice,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtCreateFile(
        &mut hBeep,
        FILE_READ_DATA | FILE_WRITE_DATA,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        0,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    if (dwFreq >= 0x25 && dwFreq <= 0x7FFF) || (dwFreq == 0x0 && dwDuration == 0x0) {
        BeepSetParameters.Frequency = dwFreq;
        BeepSetParameters.Duration = dwDuration;

        Status = NtDeviceIoControlFile(
            hBeep,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            IOCTL_BEEP_SET,
            &mut BeepSetParameters as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<BEEP_SET_PARAMETERS>() as u32,
            core::ptr::null_mut(),
            0,
        );
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    NotifySoundSentry();

    if !NT_SUCCESS(Status) {
        NtClose(hBeep);
        BaseSetLastNTError(Status);
        return 0;
    }

    if ((dwFreq != 0x0) || (dwDuration != 0x0)) && (dwDuration != MAXDWORD) {
        SleepEx(dwDuration, 1);
    }

    NtClose(hBeep);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn DeviceIoControl(
    hDevice: HANDLE,
    dwIoControlCode: DWORD,
    lpInBuffer: LPVOID,
    nInBufferSize: DWORD,
    lpOutBuffer: LPVOID,
    nOutBufferSize: DWORD,
    lpBytesReturned: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let FsIoCtl: BOOL;
    let mut Status: NTSTATUS;
    let ApcContext: PVOID;
    // `mut` and `&mut`: the kernel writes this block, so a shared borrow cast to
    // `*mut` would be UB that lets LLVM fold the Status/Information reads to 0.
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    FsIoCtl = ((dwIoControlCode >> 16) == FILE_DEVICE_FILE_SYSTEM) as BOOL;

    if lpOverlapped != core::ptr::null_mut() {
        (*lpOverlapped).Internal = STATUS_PENDING as ULONG_PTR;

        ApcContext = if (((*lpOverlapped).hEvent as ULONG_PTR) & 0x1) != 0 {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        };

        if FsIoCtl != 0 {
            Status = NtFsControlFile(
                hDevice,
                (*lpOverlapped).hEvent,
                None,
                ApcContext,
                lpOverlapped as PIO_STATUS_BLOCK,
                dwIoControlCode,
                lpInBuffer,
                nInBufferSize,
                lpOutBuffer,
                nOutBufferSize,
            );
        } else {
            Status = NtDeviceIoControlFile(
                hDevice,
                (*lpOverlapped).hEvent,
                None,
                ApcContext,
                lpOverlapped as PIO_STATUS_BLOCK,
                dwIoControlCode,
                lpInBuffer,
                nInBufferSize,
                lpOutBuffer,
                nOutBufferSize,
            );
        }

        if !(NT_ERROR(Status)) && !lpBytesReturned.is_null() {
            // _SEH2_TRY { *lpBytesReturned = lpOverlapped->InternalHigh; }
            // (DUMMY_PSEH: handler is dead code)
            *lpBytesReturned = (*lpOverlapped).InternalHigh as DWORD;
            // _SEH2_END
        }

        if !(NT_SUCCESS(Status)) || (Status == STATUS_PENDING) {
            BaseSetLastNTError(Status);
            return 0i32;
        }
    } else {
        if FsIoCtl != 0 {
            Status = NtFsControlFile(
                hDevice,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut Iosb as *mut _IO_STATUS_BLOCK,
                dwIoControlCode,
                lpInBuffer,
                nInBufferSize,
                lpOutBuffer,
                nOutBufferSize,
            );
        } else {
            Status = NtDeviceIoControlFile(
                hDevice,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut Iosb as *mut _IO_STATUS_BLOCK,
                dwIoControlCode,
                lpInBuffer,
                nInBufferSize,
                lpOutBuffer,
                nOutBufferSize,
            );
        }

        if Status == STATUS_PENDING {
            Status = NtWaitForSingleObject(hDevice, 0u8, core::ptr::null_mut());
            if NT_SUCCESS(Status) {
                Status = Iosb.__bindgen_anon_1.Status;
            }
        }

        // lpBytesReturned is optional here (upstream e1e3b8aa3de), as the overlapped path
        // above already assumed.
        if NT_SUCCESS(Status) {
            if !lpBytesReturned.is_null() {
                *lpBytesReturned = Iosb.Information as DWORD;
            }
        } else {
            if !NT_ERROR(Status) && !lpBytesReturned.is_null() {
                *lpBytesReturned = Iosb.Information as DWORD;
            }
            BaseSetLastNTError(Status);
            return 0i32;
        }
    }

    return 1i32;
}
#[no_mangle]
pub unsafe extern "system" fn CancelIo(hFile: HANDLE) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = NtCancelIoFile(hFile, &mut IoStatusBlock);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/file/deviceio.c
//  * PURPOSE:         Device I/O Base Client Functionality
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #include <ntddbeep.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// WINAPI
// NotifySoundSentry(VOID)
// {
//     BASE_API_MESSAGE ApiMessage;
//     PBASE_SOUND_SENTRY SoundSentryRequest = &ApiMessage.Data.SoundSentryRequest;
//
//     /* Get the video mode */
//     if (!GetConsoleDisplayMode(&SoundSentryRequest->VideoMode))
//     {
//         SoundSentryRequest->VideoMode = 0;
//     }
//
//     /* Make sure it's not fullscreen, and send the message if not */
//     if (SoundSentryRequest->VideoMode == 0)
//     {
//         CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                             NULL,
//                             CSR_CREATE_API_NUMBER(BASESRV_SERVERDLL_INDEX, BasepSoundSentryNotification),
//                             sizeof(*SoundSentryRequest));
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Beep(IN DWORD dwFreq,
//      IN DWORD dwDuration)
// {
//     HANDLE hBeep;
//     UNICODE_STRING BeepDevice;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     BEEP_SET_PARAMETERS BeepSetParameters;
//     NTSTATUS Status;
//
//     //
//     // On TS systems, we need to Load Winsta.dll and call WinstationBeepOpen
//     // after doing a GetProcAddress for it
//     //
//
//     /* Open the device */
//     RtlInitUnicodeString(&BeepDevice, L"\\Device\\Beep");
//     InitializeObjectAttributes(&ObjectAttributes, &BeepDevice, 0, NULL, NULL);
//     Status = NtCreateFile(&hBeep,
//                           FILE_READ_DATA | FILE_WRITE_DATA,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           0,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_OPEN_IF,
//                           0,
//                           NULL,
//                           0);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* check the parameters */
//     if ((dwFreq >= 0x25 && dwFreq <= 0x7FFF) ||
//         (dwFreq == 0x0 && dwDuration == 0x0))
//     {
//         /* Set beep data */
//         BeepSetParameters.Frequency = dwFreq;
//         BeepSetParameters.Duration = dwDuration;
//
//         /* Send the beep */
//         Status = NtDeviceIoControlFile(hBeep,
//                                        NULL,
//                                        NULL,
//                                        NULL,
//                                        &IoStatusBlock,
//                                        IOCTL_BEEP_SET,
//                                        &BeepSetParameters,
//                                        sizeof(BeepSetParameters),
//                                        NULL,
//                                        0);
//     }
//     else
//     {
//         /* We'll fail the call, but still notify the sound sentry */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* Notify the sound sentry */
//     NotifySoundSentry();
//
//     /* Bail out if the hardware beep failed */
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(hBeep);
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* If an actual beep was emitted, wait for it */
//     if (((dwFreq != 0x0) || (dwDuration != 0x0)) && (dwDuration != MAXDWORD))
//     {
//         SleepEx(dwDuration, TRUE);
//     }
//
//     /* Close the handle and return success */
//     NtClose(hBeep);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeviceIoControl(IN HANDLE hDevice,
//                 IN DWORD dwIoControlCode,
//                 IN LPVOID lpInBuffer OPTIONAL,
//                 IN DWORD nInBufferSize OPTIONAL,
//                 OUT LPVOID lpOutBuffer OPTIONAL,
//                 IN DWORD nOutBufferSize OPTIONAL,
//                 OUT LPDWORD lpBytesReturned OPTIONAL,
//                 IN LPOVERLAPPED lpOverlapped OPTIONAL)
// {
//     BOOL FsIoCtl;
//     NTSTATUS Status;
//     PVOID ApcContext;
//     IO_STATUS_BLOCK Iosb;
//
//     //
//     // Note: on a TS Machine, we should call IsTSAppCompatEnabled and unless the
//     // IOCTLs are IOCTL_STORAGE_EJECT_MEDIA, IOCTL_DISK_EJECT_MEDIA, FSCTL_DISMOUNT_VOLUME
//     // we should call IsCallerAdminOrSystem and return STATUS_ACCESS_DENIED for
//     // any other IOCTLs.
//     //
//
//     /* Check what kind of IOCTL to send */
//     FsIoCtl = ((dwIoControlCode >> 16) == FILE_DEVICE_FILE_SYSTEM);
//
//     /* CHeck for async */
//     if (lpOverlapped != NULL)
//     {
//         /* Set pending status */
//         lpOverlapped->Internal = STATUS_PENDING;
//
//         /* Check if there's an APC context */
//         ApcContext = (((ULONG_PTR)lpOverlapped->hEvent & 0x1) ? NULL : lpOverlapped);
//
//         /* Send file system control? */
//         if (FsIoCtl)
//         {
//             /* Send it */
//             Status = NtFsControlFile(hDevice,
//                                      lpOverlapped->hEvent,
//                                      NULL,
//                                      ApcContext,
//                                      (PIO_STATUS_BLOCK)lpOverlapped,
//                                      dwIoControlCode,
//                                      lpInBuffer,
//                                      nInBufferSize,
//                                      lpOutBuffer,
//                                      nOutBufferSize);
//         }
//         else
//         {
//             /* Otherwise send a device control */
//             Status = NtDeviceIoControlFile(hDevice,
//                                            lpOverlapped->hEvent,
//                                            NULL,
//                                            ApcContext,
//                                            (PIO_STATUS_BLOCK)lpOverlapped,
//                                            dwIoControlCode,
//                                            lpInBuffer,
//                                            nInBufferSize,
//                                            lpOutBuffer,
//                                            nOutBufferSize);
//         }
//
//         /* Check for or information instead of failure */
//         if (!(NT_ERROR(Status)) && (lpBytesReturned))
//         {
//             /* Protect with SEH */
//             _SEH2_TRY
//             {
//                 /* Return the bytes */
//                 *lpBytesReturned = lpOverlapped->InternalHigh;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return zero bytes */
//                 *lpBytesReturned = 0;
//             }
//             _SEH2_END;
//         }
//
//         /* Now check for any kind of failure except pending*/
//         if (!(NT_SUCCESS(Status)) || (Status == STATUS_PENDING))
//         {
//             /* Fail */
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* Sync case -- send file system code? */
//         if (FsIoCtl)
//         {
//             /* Do it */
//             Status = NtFsControlFile(hDevice,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      &Iosb,
//                                      dwIoControlCode,
//                                      lpInBuffer,
//                                      nInBufferSize,
//                                      lpOutBuffer,
//                                      nOutBufferSize);
//         }
//         else
//         {
//             /* Send device code instead */
//             Status = NtDeviceIoControlFile(hDevice,
//                                            NULL,
//                                            NULL,
//                                            NULL,
//                                            &Iosb,
//                                            dwIoControlCode,
//                                            lpInBuffer,
//                                            nInBufferSize,
//                                            lpOutBuffer,
//                                            nOutBufferSize);
//         }
//
//         /* Now check if the operation isn't done yet */
//         if (Status == STATUS_PENDING)
//         {
//             /* Wait for it and get the final status */
//             Status = NtWaitForSingleObject(hDevice, FALSE, NULL);
//             if (NT_SUCCESS(Status)) Status = Iosb.Status;
//         }
//
//         /* Check for success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Return the byte count */
//             *lpBytesReturned = Iosb.Information;
//         }
//         else
//         {
//             /* Check for informational or warning failure */
//             if (!NT_ERROR(Status)) *lpBytesReturned = Iosb.Information;
//
//             /* Return a failure */
//             BaseSetLastNTError(Status);
//             return FALSE;
//         }
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CancelIo(IN HANDLE hFile)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     Status = NtCancelIoFile(hFile, &IoStatusBlock);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
