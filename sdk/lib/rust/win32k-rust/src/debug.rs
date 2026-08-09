use win32k_sys::win32k::*;

extern "system" {
    fn vDbgPrintExWithPrefix( Prefix: PCHAR, ComponentId: i32, Level: u32, Format: PCHAR, arglist: va_list, );
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;
#[no_mangle]
pub unsafe extern "system" fn EngDebugPrint(
    StandardPrefix: PCHAR,
    DebugMessage: PCHAR,
    ap: va_list,
) {

    vDbgPrintExWithPrefix(
        StandardPrefix,
        -1,
        DPFLTR_ERROR_LEVEL,
        DebugMessage,
        ap,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:
//  * FILE:              win32ss/gdi/eng/debug.c
//  * PROGRAMER:         Jason Filby
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngDebugPrint(
//     _In_z_ PCHAR StandardPrefix,
//     _In_z_ PCHAR DebugMessage,
//     _In_ va_list ap)
// {
//     vDbgPrintExWithPrefix(StandardPrefix,
//                           -1,
//                           DPFLTR_ERROR_LEVEL,
//                           DebugMessage,
//                           ap);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
