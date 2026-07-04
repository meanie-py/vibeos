/*
 * ReactOS Explorer — Rust cohabitation shim for util.cpp
 *
 * util.cpp's bodies live in Rust (sdk/lib/rust/explorer-rust/src/explorer/
 * util.rs). FormatMenuString is C-variadic and stable Rust cannot DEFINE
 * C-variadic functions, so its va_start shell stays here and forwards the
 * va_list to the Rust worker FormatMenuStringV (same language-limitation
 * pattern as rtl-diag's DbgPrint trampolines in sdk/lib/rtl).
 */

#include "precomp.h"

extern "C" BOOL FormatMenuStringV(IN HMENU hMenu,
                                  IN UINT uPosition,
                                  IN UINT uFlags,
                                  IN va_list vl);

extern "C" BOOL
FormatMenuString(IN HMENU hMenu,
                 IN UINT uPosition,
                 IN UINT uFlags,
                 ...)
{
    va_list vl;
    BOOL bRet;

    va_start(vl, uFlags);
    bRet = FormatMenuStringV(hMenu, uPosition, uFlags, vl);
    va_end(vl);

    return bRet;
}
