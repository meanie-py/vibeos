/*
 * PROJECT:     ReactOS win32 kernel mode subsystem
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     cdecl shims for FORCEINLINE helpers that ntbitblt.rs calls.
 *
 * DC_pmxWorldToDevice (coord.h) is a FORCEINLINE that lazily refreshes the DC's
 * world->device matrix (calls DC_vUpdateWorldToDevice when the xform is dirty) and
 * returns &pdc->dclevel.mxWorldToDevice. bindgen emits it as an undefined extern;
 * reuse the tested C inline through a plain-cdecl w32r_* wrapper.
 */

#include <win32k.h>

PMATRIX
w32r_DC_pmxWorldToDevice(PDC pdc)
{
    return DC_pmxWorldToDevice(pdc);
}
