use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCSTR};

static mut KdbCommandHistoryBuffer: [i8; 2048] = [0; 2048];
static mut KdbCommandHistory: [PCSTR; 2048 / 8] = [core::ptr::null(); 2048 / 8];
static mut KdbCommandHistoryBufferIndex: LONG = 0;
static mut KdbCommandHistoryIndex: LONG = 0;

#[no_mangle]
pub unsafe extern "C" fn KdbpCommandHistoryAppend(Command: PCSTR) {
    let mut Length1 = strlen(Command) + 1;
    let mut Length2: usize = 0;
    let mut Buffer: PCHAR;

    if Length1 <= 1 ||
        (!KdbCommandHistory[KdbCommandHistoryIndex as usize].is_null() &&
         strcmp(KdbCommandHistory[KdbCommandHistoryIndex as usize], Command) == 0)
    {
        return;
    }

    Buffer = KdbCommandHistoryBuffer.as_mut_ptr().add(KdbCommandHistoryBufferIndex as usize);
    KdbCommandHistoryBufferIndex += Length1 as LONG;
    if KdbCommandHistoryBufferIndex >= KdbCommandHistoryBuffer.len() as LONG {
        KdbCommandHistoryBufferIndex -= KdbCommandHistoryBuffer.len() as LONG;
        Length2 = KdbCommandHistoryBufferIndex as usize;
        Length1 -= Length2;
    }

    let mut i = KdbCommandHistoryIndex;
    loop {
        if !KdbCommandHistory[i as usize].is_null() {
            if (Length2 > 0 &&
                (KdbCommandHistory[i as usize] >= Buffer ||
                 KdbCommandHistory[i as usize] < KdbCommandHistoryBuffer.as_ptr().add(KdbCommandHistoryBufferIndex as usize))) ||
                (Length2 <= 0 &&
                 (KdbCommandHistory[i as usize] >= Buffer &&
                  KdbCommandHistory[i as usize] < KdbCommandHistoryBuffer.as_ptr().add(KdbCommandHistoryBufferIndex as usize)))
            {
                KdbCommandHistory[i as usize] = core::ptr::null();
            }
        }

        i -= 1;
        if i < 0 {
            i = (KdbCommandHistory.len() - 1) as LONG;
        }

        if i == KdbCommandHistoryIndex {
            break;
        }
    }

    KdbCommandHistoryIndex += 1;
    KdbCommandHistoryIndex %= KdbCommandHistory.len() as LONG;
    if !KdbCommandHistory[KdbCommandHistoryIndex as usize].is_null() {
        KdbCommandHistory[KdbCommandHistoryIndex as usize] = core::ptr::null();
    }

    KdbCommandHistory[KdbCommandHistoryIndex as usize] = Buffer;
    core::ptr::copy_nonoverlapping(
        Command as *const u8,
        KdbCommandHistory[KdbCommandHistoryIndex as usize] as *mut u8,
        Length1
    );
    if Length2 > 0 {
        core::ptr::copy_nonoverlapping(
            (Command as *const u8).add(Length1),
            KdbCommandHistoryBuffer.as_mut_ptr() as *mut u8,
            Length2
        );
    }
}

unsafe fn KdbpGetPrevHistoryEntry(NextIndex: *mut LONG) -> PCSTR {
    if *NextIndex < 0 {
        *NextIndex = KdbCommandHistoryIndex;
    } else {
        let mut i = *NextIndex - 1;
        if i < 0 {
            *NextIndex = (KdbCommandHistory.len() - 1) as LONG;
        }

        if !KdbCommandHistory[i as usize].is_null() && i != KdbCommandHistoryIndex {
            *NextIndex = i;
        } else {
            return core::ptr::null();
        }
    }

    KdbCommandHistory[*NextIndex as usize]
}

unsafe fn KdbpGetNextHistoryEntry(NextIndex: *mut LONG) -> PCSTR {
    if !(*NextIndex > 0 && *NextIndex != KdbCommandHistoryIndex) {
        return core::ptr::null();
    }

    let mut i = *NextIndex + 1;
    if i >= KdbCommandHistory.len() as LONG {
        i = 0;
    }

    if !KdbCommandHistory[i as usize].is_null() {
        *NextIndex = i;
    }

    KdbCommandHistory[i as usize]
}

#[no_mangle]
pub unsafe extern "C" fn KdbGetHistoryEntry(NextIndex: *mut LONG, Next: BOOLEAN) -> PCSTR {
    if Next != 0 {
        KdbpGetNextHistoryEntry(NextIndex)
    } else {
        KdbpGetPrevHistoryEntry(NextIndex)
    }
}

unsafe fn strlen(s: PCSTR) -> usize {
    let mut len = 0;
    while *(s.add(len)) != 0 {
        len += 1;
    }
    len
}

unsafe fn strcmp(s1: PCSTR, s2: PCSTR) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *(s1.add(i));
        let c2 = *(s2.add(i));
        if c1 != c2 || c1 == 0 {
            return (c1 as i32) - (c2 as i32);
        }
        i += 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Command History helpers
//  * COPYRIGHT:   Copyright 2005 Gregor Anich <blight@blight.eu.org>
//  *              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// /* GLOBALS *******************************************************************/
//
// /**
//  * History range in ring buffer:
//  * (KdbCommandHistoryIndex; RTL_NUMBER_OF(KdbCommandHistory) - 1]
//  *   and [0; KdbCommandHistoryIndex].
//  **/
//
// /* Command history string ring buffer */
// static CHAR KdbCommandHistoryBuffer[2048];
// /* Command history ring buffer */
// static PCHAR KdbCommandHistory[sizeof(KdbCommandHistoryBuffer) / 8] = { NULL };
// static LONG KdbCommandHistoryBufferIndex = 0;
// static LONG KdbCommandHistoryIndex = 0;
//
// /* FUNCTIONS *****************************************************************/
//
// /**
//  * @brief   Appends a command to the command history.
//  *
//  * @param[in]   Command
//  * Pointer to the command to append to the history.
//  **/
// VOID
// KdbpCommandHistoryAppend(
//     _In_ PCSTR Command)
// {
//     SIZE_T Length1 = strlen(Command) + 1;
//     SIZE_T Length2 = 0;
//     LONG i;
//     PCHAR Buffer;
//
//     ASSERT(Length1 <= RTL_NUMBER_OF(KdbCommandHistoryBuffer));
//
//     /*
//      * Do not append the string if:
//      * - it is empty (just the NULL terminator);
//      * - or the last command is the same.
//      */
//     if (Length1 <= 1 ||
//         (KdbCommandHistory[KdbCommandHistoryIndex] &&
//          strcmp(KdbCommandHistory[KdbCommandHistoryIndex], Command) == 0))
//     {
//         return;
//     }
//
//     /* Calculate Length1 and Length2 */
//     Buffer = KdbCommandHistoryBuffer + KdbCommandHistoryBufferIndex;
//     KdbCommandHistoryBufferIndex += Length1;
//     if (KdbCommandHistoryBufferIndex >= (LONG)RTL_NUMBER_OF(KdbCommandHistoryBuffer))
//     {
//         KdbCommandHistoryBufferIndex -= RTL_NUMBER_OF(KdbCommandHistoryBuffer);
//         Length2 = KdbCommandHistoryBufferIndex;
//         Length1 -= Length2;
//     }
//
//     /* Remove previous commands until there is enough space to append the new command */
//     for (i = KdbCommandHistoryIndex; KdbCommandHistory[i];)
//     {
//         if ((Length2 > 0 &&
//             (KdbCommandHistory[i] >= Buffer ||
//              KdbCommandHistory[i] < (KdbCommandHistoryBuffer + KdbCommandHistoryBufferIndex))) ||
//             (Length2 <= 0 &&
//              (KdbCommandHistory[i] >= Buffer &&
//               KdbCommandHistory[i] < (KdbCommandHistoryBuffer + KdbCommandHistoryBufferIndex))))
//         {
//             KdbCommandHistory[i] = NULL;
//         }
//
//         i--;
//         if (i < 0)
//             i = RTL_NUMBER_OF(KdbCommandHistory) - 1;
//
//         if (i == KdbCommandHistoryIndex)
//             break;
//     }
//
//     /* Make sure the new command history entry is free */
//     KdbCommandHistoryIndex++;
//     KdbCommandHistoryIndex %= RTL_NUMBER_OF(KdbCommandHistory);
//     if (KdbCommandHistory[KdbCommandHistoryIndex])
//     {
//         KdbCommandHistory[KdbCommandHistoryIndex] = NULL;
//     }
//
//     /* Append command */
//     KdbCommandHistory[KdbCommandHistoryIndex] = Buffer;
//     ASSERT((KdbCommandHistory[KdbCommandHistoryIndex] + Length1) <= KdbCommandHistoryBuffer + RTL_NUMBER_OF(KdbCommandHistoryBuffer));
//     memcpy(KdbCommandHistory[KdbCommandHistoryIndex], Command, Length1);
//     if (Length2 > 0)
//     {
//         memcpy(KdbCommandHistoryBuffer, Command + Length1, Length2);
//     }
// }
//
// static PCSTR
// KdbpGetPrevHistoryEntry(
//     _Inout_ PLONG NextIndex)
// {
//     if (*NextIndex < 0)
//     {
//         /* Start at the end of the history */
//         *NextIndex = KdbCommandHistoryIndex;
//     }
//     else
//     {
//         LONG i = *NextIndex - 1;
//         if (i < 0)
//             *NextIndex = RTL_NUMBER_OF(KdbCommandHistory) - 1;
//
//         if (KdbCommandHistory[i] && i != KdbCommandHistoryIndex)
//             *NextIndex = i;
//         else
//             return NULL;
//     }
//
//     return KdbCommandHistory[*NextIndex];
// }
//
// static PCSTR
// KdbpGetNextHistoryEntry(
//     _Inout_ PLONG NextIndex)
// {
//     LONG i;
//
//     if (!(*NextIndex > 0 && *NextIndex != KdbCommandHistoryIndex))
//         return NULL;
//
//     i = *NextIndex + 1;
//     if (i >= (LONG)RTL_NUMBER_OF(KdbCommandHistory))
//         i = 0;
//
//     if (KdbCommandHistory[i])
//         *NextIndex = i;
//
//     return KdbCommandHistory[i];
// }
//
// PCSTR
// KdbGetHistoryEntry(
//     _Inout_ PLONG NextIndex,
//     _In_ BOOLEAN Next)
// {
//     if (Next)
//         return KdbpGetNextHistoryEntry(NextIndex);
//     else
//         return KdbpGetPrevHistoryEntry(NextIndex);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
