
#include <math.h>

_Check_return_
float
__cdecl
exp2f(
    _In_ float x)
{
    /* Keep the base opaque so clang cannot fold `pow(2, x)` back into a call to
       THIS function. `static const` was not enough: constant propagation reads
       through it, the fold fires, and exp2f compiles to a single branch-to-self —
       one arm64 instruction, an unconditional hang for every caller. It shipped
       that way (scan_self_recursive had it baselined; scan_ud2's driver sweep only
       looks at Rust-containing binaries, so it never saw ucrtbase until ucrtbase
       gained Rust). `volatile` forces a real load, so the value is no longer a
       compile-time 2 and there is nothing to fold. Same fold class as the
       memcpy/memset self-recursion in AGENTS.md. */
    static const volatile float TWO = 2.0f;
    return powf(TWO, x);
}
