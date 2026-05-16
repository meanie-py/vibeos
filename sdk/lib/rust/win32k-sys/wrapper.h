/* win32k-sys bindgen wrapper — internal win32k.sys FFI surface.
 *
 * Mirrors the ntkrnl-sys libclang workarounds. Critically uses
 * -fms-extensions (win32k object headers — THRDESKHEAD { THROBJHEAD; }
 * etc. — rely on the MS anonymous-typedef-member extension) but NEVER
 * -fms-compatibility (that defines _MSC_VER, sending mingw's _mingw.h
 * down the MSVC branch which never defines __CRT_INLINE → breaks the
 * <math.h> that pch.h pulls in). See docs/engineering/
 * win32k-pretranslation-audit.md.
 */
#define _WIN32K_ 1
#define __REACTOS__
#define _USE_DUMMY_PSEH 1
/* Skip compiler intrinsics — not available in libclang (ntkrnl-sys pattern).
   pch.h does #include <intrin.h>. */
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
/* ARM64 intrinsic stubs (ntkrnl-sys pattern) */
#define __wfi() ((void)0)
#define __yield() ((void)0)
#undef FORCEINLINE
#define FORCEINLINE

/* win32k master internal header — pulls pch.h: ntifs, ntddkbd/mou/vdeo,
   wingdi, winddi, winuser, ntgdi.h, ntuser.h, win32kp.h, freetype, ... */
#include <win32k.h>
