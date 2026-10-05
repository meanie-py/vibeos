use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor, PCSTR};

// ============================================================================
// Local declarations for symbols not yet wired up in ntkrnl-sys
// (kdterminal.c is excluded from the C build — Rust owns these globals)
// ============================================================================

/// SIZE struct (Win32 GDI) — `tagSIZE { LONG cx; LONG cy; }`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SIZE {
    pub cx: LONG,
    pub cy: LONG,
}
#[allow(non_camel_case_types)]
pub type PSIZE = *mut SIZE;

/// KD private debug-mode flags (mirror of `KDP_DEBUG_MODE` union in
/// `ntoskrnl/kd/kd.h`). The C type is a `union { struct { UCHAR Screen:1;
/// UCHAR Serial:1; UCHAR File:1; }; ULONG Value; }`. We model the bitfield
/// variant as plain `UCHAR` fields here; the layout matches because each is
/// in the lowest byte of the union and the union is `ULONG`-sized.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct KDP_DEBUG_MODE {
    pub Screen: u8,
    pub Serial: u8,
    pub File:   u8,
    pub _pad:   u8,
}

impl KDP_DEBUG_MODE {
    #[inline]
    pub fn Value(&self) -> u32 {
        u32::from_le_bytes([self.Screen, self.Serial, self.File, self._pad])
    }
}

// KD terminal key/mode constants (from ntoskrnl/kd/kdterminal.h)
pub const KEY_ESC:        CHAR  = 27;
pub const KEY_SCAN_UP:    ULONG = 72;
pub const KEY_SCAN_DOWN:  ULONG = 80;
pub const KD_DEBUG_KDSERIAL: ULONG = 1;
#[allow(dead_code)]
pub const KD_DEBUG_KDNOECHO: ULONG = 2;

pub const ANSI_NULL: CHAR = 0;

// Bootvid constants (from sdk/include/reactos/drivers/bootvid/display.h)
pub const SCREEN_WIDTH:  ULONG = 640;
pub const SCREEN_HEIGHT: ULONG = 480;

// ----------------------------------------------------------------------------
// Globals owned by kdterminal.c — kdterminal.c is excluded from the build, so
// these are the canonical definitions. C consumers see the unmangled symbols.
// ----------------------------------------------------------------------------

#[no_mangle]
pub static mut KdbDebugState: ULONG = 0;

#[no_mangle]
pub static mut KdTermSize: SIZE = SIZE { cx: 0, cy: 0 };

#[no_mangle]
pub static mut KdTermConnected: BOOLEAN = FALSE;

#[no_mangle]
pub static mut KdTermSerial: BOOLEAN = FALSE;

#[no_mangle]
pub static mut KdTermReportsSize: BOOLEAN = TRUE;

// File-static in C. Keep file-private here.
static mut KdTermNextKey: CHAR = ANSI_NULL;

// ----------------------------------------------------------------------------
// Globals owned by other kd/* files (extern-declared here)
// ----------------------------------------------------------------------------
extern "C" {
    static mut KdpDebugMode: KDP_DEBUG_MODE; // owned by kdio.rs/.c
}

extern "system" {
    fn KdbpTryGetCharKeyboard(ScanCode: *mut ULONG, Retry: ULONG) -> LONG;
    #[link_name = "KdbpTryGetCharSerial"]
    fn KdbpTryGetCharSerial__c(Retry: ULONG) -> CHAR;
    fn KdbpSendCommandSerial(Command: PCSTR);
}

// kdserial.rs returns CHAR: through a LONG decl the bits above the byte were never written, so
// its -1 (no input) need not compare equal to -1. Sign-extended here, as the C caller would.
#[allow(non_snake_case)]
unsafe fn KdbpTryGetCharSerial(Retry: ULONG) -> LONG {
    KdbpTryGetCharSerial__c(Retry) as LONG
}

// KdbpGetCharSerial is a C macro `#define KdbpGetCharSerial() KdbpTryGetCharSerial(0)`
// in kdterminal.h — inline it. C signature: CHAR KdbpTryGetCharSerial(...).
// Returns CHAR; -1 cast as CHAR is 0xFFi8.
#[inline(always)]
unsafe fn KdbpGetCharSerial() -> CHAR {
    KdbpTryGetCharSerial(0) as CHAR
}

extern "C" {
    fn isalpha(c: i32) -> i32;
    fn strtoul(s: *const CHAR, endptr: *mut *mut CHAR, base: i32) -> ULONG;
}

// `#define KdbpGetCharKeyboard(ScanCode) KdbpTryGetCharKeyboard((ScanCode), 0)`
#[inline]
unsafe fn KdbpGetCharKeyboard(ScanCode: PULONG) -> CHAR {
    KdbpTryGetCharKeyboard(ScanCode, 0) as CHAR
}

// ============================================================================
// Functions
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn KdpInitTerminal() -> BOOLEAN {

    KdTermSerial = if (KdpDebugMode.Serial != 0)
        && (((KdbDebugState & KD_DEBUG_KDSERIAL) != 0) || (KdpDebugMode.Screen == 0))
    { TRUE } else { FALSE };

    KdpFlushTerminalInput();

    if KdTermSerial != 0 {
        let mut Length: ULONG;

        KdbpSendCommandSerial(b"\x1b[?7h\0".as_ptr() as PCSTR);

        KdbpSendCommandSerial(b"\x1b[c\0".as_ptr() as PCSTR);
        KeStallExecutionProcessor(100000);

        Length = 0;
        loop {
            if KdbpTryGetCharSerial(5000) == -1 {
                break;
            }
            Length += 1;
        }

        KdTermConnected = if Length > 0 { TRUE } else { FALSE };
    } else {
        KdTermConnected = FALSE;
    }
    KdTermConnected
}

#[no_mangle]
pub unsafe extern "C" fn KdpUpdateTerminalSize(
    TermSize: PSIZE
) -> BOOLEAN {

    static mut Buffer: [CHAR; 128] = [0; 128];
    let mut c: CHAR;
    let mut NumberOfCols: LONG = -1;
    let mut NumberOfRows: LONG = -1;

    if KdTermConnected != 0 && KdTermSerial != 0 && KdTermReportsSize != 0 {
        KdpFlushTerminalInput();

        KdTermReportsSize = FALSE;
        KdbpSendCommandSerial(b"\x1b[18t\0".as_ptr() as PCSTR);
        KeStallExecutionProcessor(100000);

        c = KdbpTryGetCharSerial(5000) as CHAR;
        if c == KEY_ESC {
            c = KdbpTryGetCharSerial(5000) as CHAR;
            if c == b'[' as CHAR {
                let mut Length: ULONG = 0;
                loop {
                    let raw = KdbpTryGetCharSerial(5000);
                    if raw == -1 {
                        break;
                    }
                    c = raw as CHAR;

                    Buffer[Length as usize] = c;
                    Length += 1;
                    if isalpha(c as i32) != 0 || Length >= (128 - 1) {
                        break;
                    }
                }
                Buffer[Length as usize] = ANSI_NULL;

                if Buffer[0] == b'8' as CHAR && Buffer[1] == b';' as CHAR {
                    let mut i: usize = 2;
                    while i < Length as usize && Buffer[i] != b';' as CHAR {
                        i += 1;
                    }

                    if Buffer[i] == b';' as CHAR {
                        Buffer[i] = ANSI_NULL;
                        i += 1;

                        NumberOfRows = strtoul((*(&raw const Buffer)).as_ptr().add(2), core::ptr::null_mut(), 0) as LONG;
                        NumberOfCols = strtoul((*(&raw const Buffer)).as_ptr().add(i), core::ptr::null_mut(), 0) as LONG;
                        KdTermReportsSize = TRUE;
                    }
                }
            }
            while KdbpTryGetCharSerial(5000) != -1 {}
        }
    }

    if NumberOfCols <= 0 {
        if KdpDebugMode.Screen != 0 && KdTermSerial == 0 {
            NumberOfCols = (SCREEN_WIDTH / 8) as LONG;
        } else {
            NumberOfCols = 80;
        }
    }
    if NumberOfRows <= 0 {
        if KdpDebugMode.Screen != 0 && KdTermSerial == 0 {
            NumberOfRows = (SCREEN_HEIGHT / (13 + 1)) as LONG;
        } else {
            NumberOfRows = 24;
        }
    }

    (*TermSize).cx = NumberOfCols;
    (*TermSize).cy = NumberOfRows;

    KdTermReportsSize
}

#[no_mangle]
pub unsafe extern "C" fn KdpFlushTerminalInput() {

    KdTermNextKey = ANSI_NULL;
    if (KdbDebugState & KD_DEBUG_KDSERIAL) != 0 {
        while KdbpTryGetCharSerial(1) != -1 {}
    } else {
        let mut ScanCode: ULONG = 0;
        while KdbpTryGetCharKeyboard(&mut ScanCode, 1) != -1 {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdpReadTermKey(
    ScanCode: PULONG
) -> CHAR
{

    let mut Key: CHAR;

    *ScanCode = 0;

    if (KdbDebugState & KD_DEBUG_KDSERIAL) != 0
    {
        Key = if KdTermNextKey == 0 { KdbpGetCharSerial() } else { KdTermNextKey };
        KdTermNextKey = ANSI_NULL;
        if Key == KEY_ESC
        {
            Key = KdbpGetCharSerial();
            if Key == b'[' as CHAR
            {
                Key = KdbpGetCharSerial();
                match Key as u8
                {
                    b'A' =>
                    {
                        *ScanCode = KEY_SCAN_UP;
                    }
                    b'B' =>
                    {
                        *ScanCode = KEY_SCAN_DOWN;
                    }
                    b'C' =>
                    {
                    }
                    b'D' =>
                    {
                    }
                    _ => {}
                }
            }
        }
    }
    else
    {
        Key = if KdTermNextKey == 0 { KdbpGetCharKeyboard(ScanCode) } else { KdTermNextKey };
        KdTermNextKey = ANSI_NULL;
    }

    if Key == b'\r' as CHAR
    {
        KeStallExecutionProcessor(100000);

        if (KdbDebugState & KD_DEBUG_KDSERIAL) != 0
        {
            KdTermNextKey = KdbpTryGetCharSerial(5) as CHAR;
        }
        else
        {
            KdTermNextKey = KdbpTryGetCharKeyboard(ScanCode, 5) as CHAR;
        }

        if KdTermNextKey == b'\n' as CHAR || KdTermNextKey == -1
        {
            KdTermNextKey = ANSI_NULL;
        }
    }

    return Key;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger Terminal Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     KD Terminal Management
//  * COPYRIGHT:   Copyright 2005 Gregor Anich <blight@blight.eu.org>
//  *              Copyright 2022-2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kd.h"
// #include "kdterminal.h"
//
// #define KdbpGetCharKeyboard(ScanCode) KdbpTryGetCharKeyboard((ScanCode), 0)
// CHAR
// KdbpTryGetCharKeyboard(PULONG ScanCode, ULONG Retry);
//
// #define KdbpGetCharSerial()  KdbpTryGetCharSerial(0)
// CHAR
// KdbpTryGetCharSerial(
//     _In_ ULONG Retry);
//
// VOID
// KdbpSendCommandSerial(
//     _In_ PCSTR Command);
//
//
// /* GLOBALS *******************************************************************/
//
// /* KD Controlling Terminal */
// ULONG KdbDebugState = 0; /* KDBG Settings (NOECHO, KDSERIAL) */
// SIZE KdTermSize = {0,0};
// BOOLEAN KdTermConnected = FALSE;
// BOOLEAN KdTermSerial = FALSE;
// BOOLEAN KdTermReportsSize = TRUE;
//
// static CHAR KdTermNextKey = ANSI_NULL; /* 1-character input queue buffer */
//
//
// /* FUNCTIONS *****************************************************************/
//
// /**
//  * @brief   Initializes the controlling terminal.
//  *
//  * @return
//  * TRUE if the controlling terminal is serial and detected
//  * as being connected, or FALSE if not.
//  **/
// BOOLEAN
// KdpInitTerminal(VOID)
// {
//     /* Determine whether the controlling terminal is a serial terminal:
//      * serial output is enabled *and* KDSERIAL is set (i.e. user input
//      * through serial). */
//     KdTermSerial =
// #if 0
//     // Old logic where KDSERIAL also enables serial output.
//     (KdbDebugState & KD_DEBUG_KDSERIAL) ||
//     (KdpDebugMode.Serial && !KdpDebugMode.Screen);
// #else
//     // New logic where KDSERIAL does not necessarily enable serial output.
//     KdpDebugMode.Serial &&
//     ((KdbDebugState & KD_DEBUG_KDSERIAL) || !KdpDebugMode.Screen);
// #endif
//
//     /* Flush the input buffer */
//     KdpFlushTerminalInput();
//
//     if (KdTermSerial)
//     {
//         ULONG Length;
//
//         /* Enable line-wrap */
//         KdbpSendCommandSerial("\x1b[?7h");
//
//         /*
//          * Query terminal type.
//          * Historically it was done with CTRL-E ('\x05'), however nowadays
//          * terminals respond to it with an empty (or a user-configurable)
//          * string. Instead, use the VT52-compatible 'ESC Z' sequence or the
//          * VT100-compatible 'ESC[c' one.
//          */
//         KdbpSendCommandSerial("\x1b[c");
//         KeStallExecutionProcessor(100000);
//
//         Length = 0;
//         for (;;)
//         {
//             /* Verify we get an answer, but don't care about it */
//             if (KdbpTryGetCharSerial(5000) == -1)
//                 break;
//             ++Length;
//         }
//
//         /* Terminal is connected (TRUE) or not connected (FALSE) */
//         KdTermConnected = (Length > 0);
//     }
//     else
//     {
//         /* Terminal is not serial, assume it's *not* connected */
//         KdTermConnected = FALSE;
//     }
//     return KdTermConnected;
// }
//
// BOOLEAN
// KdpUpdateTerminalSize(
//     _Out_ PSIZE TermSize)
// {
//     static CHAR Buffer[128];
//     CHAR c;
//     LONG NumberOfCols = -1; // Or initialize to TermSize->cx ??
//     LONG NumberOfRows = -1; // Or initialize to TermSize->cy ??
//
//     /* Retrieve the size of the controlling terminal only when it is serial */
//     if (KdTermConnected && KdTermSerial && KdTermReportsSize)
//     {
//         /* Flush the input buffer */
//         KdpFlushTerminalInput();
//
//         /* Try to query the terminal size. A reply looks like "\x1b[8;24;80t" */
//         KdTermReportsSize = FALSE;
//         KdbpSendCommandSerial("\x1b[18t");
//         KeStallExecutionProcessor(100000);
//
//         c = KdbpTryGetCharSerial(5000);
//         if (c == KEY_ESC)
//         {
//             c = KdbpTryGetCharSerial(5000);
//             if (c == '[')
//             {
//                 ULONG Length = 0;
//                 for (;;)
//                 {
//                     c = KdbpTryGetCharSerial(5000);
//                     if (c == -1)
//                         break;
//
//                     Buffer[Length++] = c;
//                     if (isalpha(c) || Length >= (sizeof(Buffer) - 1))
//                         break;
//                 }
//                 Buffer[Length] = ANSI_NULL;
//
//                 if (Buffer[0] == '8' && Buffer[1] == ';')
//                 {
//                     SIZE_T i;
//                     for (i = 2; (i < Length) && (Buffer[i] != ';'); i++);
//
//                     if (Buffer[i] == ';')
//                     {
//                         Buffer[i++] = ANSI_NULL;
//
//                         /* Number of rows is now at Buffer + 2
//                          * and number of columns at Buffer + i */
//                         NumberOfRows = strtoul(Buffer + 2, NULL, 0);
//                         NumberOfCols = strtoul(Buffer + i, NULL, 0);
//                         KdTermReportsSize = TRUE;
//                     }
//                 }
//             }
//             /* Clear further characters */
//             while (KdbpTryGetCharSerial(5000) != -1);
//         }
//     }
//
//     if (NumberOfCols <= 0)
//     {
//         /* Set the number of columns to the default */
//         if (KdpDebugMode.Screen && !KdTermSerial)
//             NumberOfCols = (SCREEN_WIDTH / 8 /*BOOTCHAR_WIDTH*/);
//         else
//             NumberOfCols = 80;
//     }
//     if (NumberOfRows <= 0)
//     {
//         /* Set the number of rows to the default */
//         if (KdpDebugMode.Screen && !KdTermSerial)
//             NumberOfRows = (SCREEN_HEIGHT / (13 /*BOOTCHAR_HEIGHT*/ + 1));
//         else
//             NumberOfRows = 24;
//     }
//
//     TermSize->cx = NumberOfCols;
//     TermSize->cy = NumberOfRows;
//
//     // KdIoPrintf("Cols/Rows: %dx%d\n", TermSize->cx, TermSize->cy);
//
//     return KdTermReportsSize;
// }
//
// /**
//  * @brief   Flushes terminal input (either serial or PS/2).
//  **/
// VOID
// KdpFlushTerminalInput(VOID)
// {
//     KdTermNextKey = ANSI_NULL;
//     if (KdbDebugState & KD_DEBUG_KDSERIAL)
//     {
//         while (KdbpTryGetCharSerial(1) != -1);
//     }
//     else
//     {
//         ULONG ScanCode;
//         while (KdbpTryGetCharKeyboard(&ScanCode, 1) != -1);
//     }
// }
//
// /**
//  * @brief
//  * Reads one character from the terminal. This function returns
//  * a scan code even when reading is done from a serial terminal.
//  **/
// CHAR
// KdpReadTermKey(
//     _Out_ PULONG ScanCode)
// {
//     CHAR Key;
//
//     *ScanCode = 0;
//
//     if (KdbDebugState & KD_DEBUG_KDSERIAL)
//     {
//         Key = (!KdTermNextKey ? KdbpGetCharSerial() : KdTermNextKey);
//         KdTermNextKey = ANSI_NULL;
//         if (Key == KEY_ESC) /* ESC */
//         {
//             Key = KdbpGetCharSerial();
//             if (Key == '[')
//             {
//                 Key = KdbpGetCharSerial();
//                 switch (Key)
//                 {
//                     case 'A':
//                         *ScanCode = KEY_SCAN_UP;
//                         break;
//                     case 'B':
//                         *ScanCode = KEY_SCAN_DOWN;
//                         break;
//                     case 'C':
//                         break;
//                     case 'D':
//                         break;
//                 }
//             }
//         }
//     }
//     else
//     {
//         Key = (!KdTermNextKey ? KdbpGetCharKeyboard(ScanCode) : KdTermNextKey);
//         KdTermNextKey = ANSI_NULL;
//     }
//
//     /* Check for return */
//     if (Key == '\r')
//     {
//         /*
//          * We might need to discard the next '\n' which most clients
//          * should send after \r. Wait a bit to make sure we receive it.
//          */
//         KeStallExecutionProcessor(100000);
//
//         if (KdbDebugState & KD_DEBUG_KDSERIAL)
//             KdTermNextKey = KdbpTryGetCharSerial(5);
//         else
//             KdTermNextKey = KdbpTryGetCharKeyboard(ScanCode, 5);
//
//         if (KdTermNextKey == '\n' || KdTermNextKey == -1) /* \n or no response at all */
//             KdTermNextKey = ANSI_NULL;
//     }
//
//     return Key;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
