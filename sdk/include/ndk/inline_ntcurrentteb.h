#ifndef _INLINE_NT_CURRENTTEB_H_
#define _INLINE_NT_CURRENTTEB_H_

#ifdef __cplusplus
extern "C" {
#endif

FORCEINLINE struct _TEB * NtCurrentTeb(VOID)
{
#if defined(_M_IX86)
    return (struct _TEB *)__readfsdword(0x18);
#elif defined(_M_AMD64)
    return (struct _TEB *)__readgsqword(FIELD_OFFSET(NT_TIB, Self));
#elif defined(_M_ARM)
    // return (struct _TEB *)KeGetPcr()->Used_Self;
    return (struct _TEB *)(ULONG_PTR)_MoveFromCoprocessor(CP15_TPIDRURW);
#elif defined (_M_ARM64)
    /* ARM64 Windows ABI: x18 is the platform register holding the TEB pointer.
     * Set by KiSwapContextInternal (kernel context switch) and
     * KiInitializeContextThread (initial thread setup in trap frame). */
    {
        struct _TEB *_teb;
        __asm__ __volatile__("mov %0, x18" : "=r"(_teb));
        return _teb;
    }
// #elif defined(_M_PPC)
//     return (struct _TEB *)_read_teb_dword(0x18);
#else
#error Unsupported architecture
#endif
}

#ifdef __cplusplus
}
#endif

#endif // _INLINE_NT_CURRENTTEB_H_
