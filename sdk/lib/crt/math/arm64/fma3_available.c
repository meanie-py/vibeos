/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     FMA3 stubs for ARM64 (FMA3 is x86-specific)
 */

typedef int (__cdecl *_PIFV)(void);

int __fma3_is_available = 0;
int __use_fma3_lib = 0;

int __cdecl _set_FMA3_enable(int flag)
{
    (void)flag;
    return 0;
}

int __fma3_lib_init(void)
{
    return 0;
}
