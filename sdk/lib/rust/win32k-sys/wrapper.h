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
/* ARM64: vcruntime/intrin.h declares __wfi/__yield as `void f(void)` since the upstream
   merge, so the old `#define __yield() ((void)0)` stubs now break the parse ("too many
   arguments provided to function-like macro invocation") and are not needed. */
#undef FORCEINLINE
#define FORCEINLINE

/* win32k master internal header — pulls pch.h: ntifs, ntddkbd/mou/vdeo,
   wingdi, winddi, winuser, ntgdi.h, ntuser.h, win32kp.h, freetype, ... */
#include <win32k.h>

/* freetype.c reaches into FreeType headers + the ntgdi font.h that win32k.h
   does not pull. Include them here so win32k-sys carries the complete FreeType
   (FT and TT) surface, consistent with the FT fields already embedded in win32k
   structs, plus the ntgdi-private FONT_ENTRY / FONT_CACHE_ENTRY / GDI_LOAD_FONT /
   FONTSUBST_ENTRY types, so freetype.c's Rust translation resolves everything
   from one crate. font.h must come after the FT headers (it uses FT_Face etc.).
   FT2_BUILD_LIBRARY + internal.h match how freetype.c itself includes FreeType. */
#ifndef FT2_BUILD_LIBRARY
#define FT2_BUILD_LIBRARY
#endif
/* FreeType 2.10.4 (upstream #8671) deleted internal/internal.h, which only defined the
   FT_INTERNAL_* include-path macros; ft2build.h brings the public FT_*_H macros used below. */
#include <ft2build.h>
#include FT_GLYPH_H
#include FT_BITMAP_H
#include FT_OUTLINE_H
#include FT_TYPE1_TABLES_H
#include FT_TRUETYPE_TABLES_H
#include FT_TRUETYPE_TAGS_H
#include FT_TRUETYPE_IDS_H
#include FT_TRIGONOMETRY_H
#include FT_WINFONTS_H
#include FT_SFNT_NAMES_H
#include FT_SYNTHESIS_H
#include <gdi/ntgdi/font.h>
