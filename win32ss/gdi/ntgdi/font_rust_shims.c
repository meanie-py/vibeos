/*
 * PROJECT:     ReactOS win32 kernel mode subsystem
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     cdecl shims for FORCEINLINE helpers that font.rs calls.
 *
 * TEXTOBJ_LockText / TEXTOBJ_UnlockText are FORCEINLINE wrappers (text.h) over the
 * push-lock acquire/release path (ExAcquire/ReleasePushLockExclusive — themselves
 * FORCEINLINE atomics) plus GDIOBJ ref/deref. bindgen emits them as undefined extern
 * declarations, so the Rust translation can't link against them directly. Re-export the
 * tested C inline logic through plain-cdecl w32r_* wrappers instead of reimplementing
 * the push-lock atomics in Rust.
 */

#include <win32k.h>

PTEXTOBJ
w32r_TEXTOBJ_LockText(HFONT hfont)
{
    return TEXTOBJ_LockText(hfont);
}

VOID
w32r_TEXTOBJ_UnlockText(PTEXTOBJ pto)
{
    TEXTOBJ_UnlockText((PLFONT)pto);
}
