/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     MIT (https://spdx.org/licenses/MIT)
 * PURPOSE:     Implementation of _control87
 * COPYRIGHT:   Copyright 2022 Timo Kreuzer <timo.kreuzer@reactos.org>
 */

#include <xmmintrin.h>
#include <float.h>

unsigned int _get_native_fpcw(void);
void _set_native_fpcw(unsigned int value);
unsigned int _fpcw_native_to_abstract(unsigned int native);
unsigned int _fpcw_abstract_to_native(unsigned int abstract);

unsigned int __cdecl _control87(unsigned int newval, unsigned int mask)
{
    unsigned int native, oldval, updated;

    /* Sanatize the mask */
    mask &= _MCW_DN | _MCW_EM | _MCW_RC;

    /* Get native control word */
    native = _get_native_fpcw();

    /* Convert to abstract */
    oldval = _fpcw_native_to_abstract(native);

    /* Update it according to the given parameters */
    updated = (oldval & ~mask) | (newval & mask);

    /* Convert back to native, keeping the exception flags: MXCSR holds the
       status bits too, and rebuilding it from the control word alone cleared
       every pending exception on each call -- even _control87(0, 0), a read. */
    native = _fpcw_abstract_to_native(updated) | (native & _MM_EXCEPT_MASK);

    /* Set the native value */
    _set_native_fpcw(native);

    return updated;
}
