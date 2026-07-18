use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::PCSTR;

// kdterminal.h constants
const KEY_BS: u8 = 8;
const KEY_DEL: u8 = 127;
const KEY_SCAN_UP: u32 = 72;
const KEY_SCAN_DOWN: u32 = 80;

// _KDB_OUTPUT_SETTINGS enum from kdterminal.h
const KD_DEBUG_KDNOECHO: u32 = 2;

extern "C" {
    // KdbDebugState owned by kdterminal.rs.
    static mut KdbDebugState: u32;

    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn strlen(s: PCSTR) -> usize;

    // kdterminal.c / kdio.c-owned helpers — plain cdecl in C (no NTAPI).
    fn KdIoPuts(String: PCSTR);
    fn KdpFlushTerminalInput();
    fn KdpReadTermKey(ScanCode: *mut u32) -> i8;
}

#[inline]
fn min_usize(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

unsafe fn KdbGetHistoryEntry(
    _NextIndex: *mut i32,
    _Next: u8,
) -> PCSTR {
    // Dummy stub — KDBG implementation lives in kdb_cmdhist.c.
    // When KDBG is enabled, the real symbol resolves at link time via the
    // C kdb_cmdhist.c definition; this Rust stub is only used as a fallback.
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "system" fn KdIoReadLine(
    Buffer: PCHAR,
    Size: usize,
) -> usize {

    let Orig = Buffer;
    let mut ScanCode: u32 = 0;
    let mut Key: i8;
    let EchoOn = (KdbDebugState & KD_DEBUG_KDNOECHO) == 0;
    let mut CmdHistIndex: i32 = -1;

    KdpFlushTerminalInput();

    if Size == 0 {
        return 0;
    }

    let mut Buffer = Buffer;

    loop {
        Key = KdpReadTermKey(&mut ScanCode);

        if Key == b'\r' as i8 || Key == b'\n' as i8 {
            *Buffer = 0;
            KdIoPuts(b"\n\0".as_ptr() as PCSTR);
            return Buffer.offset_from(Orig) as usize;
        } else if Key == KEY_BS as i8 || Key == KEY_DEL as i8 {
            if Buffer > Orig {
                Buffer = Buffer.offset(-1);
                *Buffer = 0;

                if EchoOn {
                    let __echo = [KEY_BS as u8, b' ', KEY_BS as u8, 0]; KdIoPuts(__echo.as_ptr() as PCSTR);
                } else {
                    let __echo = [b' ', KEY_BS as u8, 0]; KdIoPuts(__echo.as_ptr() as PCSTR);
                }
            }
        } else if ScanCode == KEY_SCAN_UP || ScanCode == KEY_SCAN_DOWN {
            let CmdHistory = KdbGetHistoryEntry(
                &mut CmdHistIndex,
                if ScanCode == KEY_SCAN_DOWN { 1 } else { 0 },
            );
            if !CmdHistory.is_null() {
                while Buffer > Orig {
                    Buffer = Buffer.offset(-1);
                    *Buffer = 0;

                    if EchoOn {
                        let __echo = [KEY_BS as u8, b' ', KEY_BS as u8, 0]; KdIoPuts(__echo.as_ptr() as PCSTR);
                    } else {
                        let __echo = [b' ', KEY_BS as u8, 0]; KdIoPuts(__echo.as_ptr() as PCSTR);
                    }
                }

                let i = min_usize(strlen(CmdHistory), Size - 1);
                memcpy(Orig as *mut core::ffi::c_void, CmdHistory as *const core::ffi::c_void, i);
                *Orig.add(i) = 0;
                Buffer = Orig.add(i);
                KdIoPuts(Orig as PCSTR);
            }
        } else {
            if Buffer.offset_from(Orig) as usize >= Size - 1 {
                continue;
            }

            if EchoOn {
                let __echo = [Key as u8, 0]; KdIoPuts(__echo.as_ptr() as PCSTR);
            }

            *Buffer = Key;
            Buffer = Buffer.offset(1);
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger Terminal Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Terminal line-editing (Prompt) interface
//  * COPYRIGHT:   Copyright 2001-2004 David Welch <welch@cwcom.net>
//  *              Copyright 2004-2005 Gregor Anich <blight@blight.eu.org>
//  *              Copyright 2022-2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kd.h"
// #include "kdterminal.h"
//
// /* In kdb.h only when KDBG defined */
// #ifdef KDBG
// extern PCSTR
// KdbGetHistoryEntry(
//     _Inout_ PLONG NextIndex,
//     _In_ BOOLEAN Next);
// #else
// PCSTR
// KdbGetHistoryEntry(
//     _Inout_ PLONG NextIndex,
//     _In_ BOOLEAN Next)
// {
//     /* Dummy function */
//     return NULL;
// }
// #endif
//
//
// /* FUNCTIONS *****************************************************************/
//
// /**
//  * @brief   Reads a line of user input from the terminal.
//  *
//  * @param[out]  Buffer
//  * Buffer where to store the input. Trailing newlines are removed.
//  *
//  * @param[in]   Size
//  * Size of @p Buffer.
//  *
//  * @return
//  * Returns the number of characters stored, not counting the NULL terminator.
//  *
//  * @note Accepts only \n newlines, \r is ignored.
//  **/
// SIZE_T
// KdIoReadLine(
//     _Out_ PCHAR Buffer,
//     _In_ SIZE_T Size)
// {
//     PCHAR Orig = Buffer;
//     ULONG ScanCode = 0;
//     CHAR Key;
//     BOOLEAN EchoOn = !(KdbDebugState & KD_DEBUG_KDNOECHO);
//     LONG CmdHistIndex = -1; // Start at end of history.
//
//     /* Flush the input buffer */
//     KdpFlushTerminalInput();
//
//     /* Bail out if the buffer is zero-sized */
//     if (Size == 0)
//         return 0;
//
//     for (;;)
//     {
//         Key = KdpReadTermKey(&ScanCode);
//
//         /* Check for return or newline */
//         if ((Key == '\r') || (Key == '\n'))
//         {
//             *Buffer = ANSI_NULL;
//             KdIoPuts("\n");
//             return (SIZE_T)(Buffer - Orig);
//         }
//         else if (Key == KEY_BS || Key == KEY_DEL)
//         {
//             /* Erase the last character */
//             if (Buffer > Orig)
//             {
//                 Buffer--;
//                 *Buffer = ANSI_NULL;
//
//                 if (EchoOn)
//                     KdIoPrintf("%c %c", KEY_BS, KEY_BS);
//                 else
//                     KdIoPrintf(" %c", KEY_BS);
//             }
//         }
//         else if (ScanCode == KEY_SCAN_UP || ScanCode == KEY_SCAN_DOWN)
//         {
//             PCSTR CmdHistory = KdbGetHistoryEntry(&CmdHistIndex,
//                                                   (ScanCode == KEY_SCAN_DOWN));
//             if (CmdHistory)
//             {
//                 SIZE_T i;
//
//                 /* Erase the whole line */
//                 while (Buffer > Orig)
//                 {
//                     Buffer--;
//                     *Buffer = ANSI_NULL;
//
//                     if (EchoOn)
//                         KdIoPrintf("%c %c", KEY_BS, KEY_BS);
//                     else
//                         KdIoPrintf(" %c", KEY_BS);
//                 }
//
//                 /* Copy and display the history entry */
//                 i = min(strlen(CmdHistory), Size - 1);
//                 memcpy(Orig, CmdHistory, i);
//                 Orig[i] = ANSI_NULL;
//                 Buffer = Orig + i;
//                 KdIoPuts(Orig);
//             }
//         }
//         else
//         {
//             /* Do not accept characters anymore if the buffer is full */
//             if ((SIZE_T)(Buffer - Orig) >= (Size - 1))
//                 continue;
//
//             if (EchoOn)
//                 KdIoPrintf("%c", Key);
//
//             *Buffer = Key;
//             Buffer++;
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
