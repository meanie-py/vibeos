/*
 * Minimal <wine/asm.h>.
 *
 * string.c's only users of __ASM_GLOBAL_FUNC (the hand-written SSE2
 * memmove/memcpy/memset) sit inside `#ifndef __REACTOS__` — and the
 * reference is compiled WITH __REACTOS__ (as the OS build does), so none
 * of that assembly is reachable.  ReactOS's msvcrt takes memmove from
 * elsewhere, which is why `memmove` shows up as an *undefined* symbol in
 * the real string.c.obj.  We therefore only need the macros to exist.
 */
#ifndef __WINE_ASM_H
#define __WINE_ASM_H

#define __ASM_GLOBAL_FUNC(name,code)
#define __ASM_STDCALL_FUNC(name,args,code)
#define __ASM_DEFINE_FUNC(name,code)
#define __ASM_SEH(str)
#define __ASM_CFI(str)
#define __ASM_NAME(name) name

#endif /* __WINE_ASM_H */
