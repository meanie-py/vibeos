/*
 * The static half of the file.c reference-TU prelude, part 2 of 2.
 *
 * Part 1 is msvcrt_file_pre.h, which has to land before ANY host header (it
 * works by claiming glibc's include guards).  This half lands AFTER the union
 * `ref_prelude.h`, because everything in it is built out of the Windows base
 * types that refstub/msvcrt.h defines.
 *
 * file.c cannot ride the union prelude alone for one reason: it reads and
 * writes FILE's members directly (_ptr/_cnt/_base/_flag/_file/_charbuf/
 * _bufsiz/_tmpfname), so msvcrt's `struct _iobuf` LAYOUT is part of the
 * contract under test — where the shared prelude hands out glibc's
 * `struct _IO_FILE`.  scanf.c already set the precedent that a TU with its own
 * type model gets its own prelude (ref_scanf_prelude.h).
 *
 * build.rs force-includes a GENERATED prelude that pulls both halves and adds
 * the `#define <name> ref_<name>` rename wall between them.  Everything that is
 * not a rename lives here, so it is reviewable as source rather than as a
 * string literal inside build.rs.
 */
#ifndef MSVCRT_DIFFTEST_FILE_PRELUDE_H
#define MSVCRT_DIFFTEST_FILE_PRELUDE_H

/* ── 1. the CRT/Win32 types file.c needs, and the proof they are right ───
 * Types are transcribed (a layout is not an integer); the guardian that
 * follows is generated from what the pinned cross-compiler measured against
 * msvcrt's real headers.  Include order matters: the assertions reference the
 * definitions. */
#include <msvcrt_file_types.h>
#include <msvcrt_crt_layout.h>

/* ── 2. the constants, every value computed by the cross-compiler ─────── */
#include <msvcrt_crt_consts.h>

/* ── 3. msvcrt internals the real msvcrt.h supplies at _MSVCR_VER=0 ─────
 * These live in the part of dll/win32/msvcrt/msvcrt.h that refstub/msvcrt.h
 * suppressed, so they are restored here rather than left to C's
 * implicit-declaration rule.  Values are from that header; they are plain
 * spellings of the _IO* bits and of file.c's own fd sentinels, not independent
 * constants. */
#define MSVCRT__NOBUF     _IONBF
#define MSVCRT__USERBUF   0x0100
#define MSVCRT__IOCOMMIT  0x4000
#define MSVCRT_NO_CONSOLE_FD (-2)
#define MSVCRT_NO_CONSOLE ((HANDLE)MSVCRT_NO_CONSOLE_FD)
#define _MAX__TIME64_T    (((__time64_t)0x00000007 << 32) | 0x93406FFF)
#define _IOB_ENTRIES      20

extern unsigned int MSVCRT__commode;

/* The UTF-8 <-> wide helpers are `static inline` in the real msvcrt.h
 * (lines 422-464), so suppressing that header dropped them.  Restored as
 * inlines rather than as imports: they are pure glue over
 * MultiByteToWideChar/WideCharToMultiByte, which refimpl.c already supplies,
 * and an out-of-line copy would be a body the real build never has.
 *
 * VERBATIM from that header with the `_MSVCR_VER>=140` arm collapsed — which
 * is the only reason get_aw_cp is a constant here.  Writing what the NAMES
 * suggest instead (a locale-sensitive codepage, flags of 0) is the §9.66 error
 * this harness exists to catch: at _MSVCR_VER=0 msvcrt never consults the
 * locale for these, and the narrow direction passes MB_PRECOMPOSED. */
static inline UINT get_aw_cp(void)
{
    return CP_ACP;
}

static inline int convert_acp_utf8_to_wcs(const char *str, wchar_t *wstr, int len)
{
    return MultiByteToWideChar(get_aw_cp(), MB_PRECOMPOSED, str, -1, wstr, len);
}

static inline int convert_wcs_to_acp_utf8(const wchar_t *wstr, char *str, int len)
{
    return WideCharToMultiByte(get_aw_cp(), 0, wstr, -1, str, len, NULL, NULL);
}

static inline wchar_t* wstrdupa_utf8(const char *str)
{
    int len = convert_acp_utf8_to_wcs(str, NULL, 0);
    wchar_t *wstr;

    if (!len) return NULL;
    wstr = malloc(len * sizeof(wchar_t));
    if (!wstr) return NULL;
    convert_acp_utf8_to_wcs(str, wstr, len);
    return wstr;
}

static inline char* astrdupw_utf8(const wchar_t *wstr)
{
    int len = convert_wcs_to_acp_utf8(wstr, NULL, 0);
    char *str;

    if (!len) return NULL;
    str = malloc(len * sizeof(char));
    if (!str) return NULL;
    convert_wcs_to_acp_utf8(wstr, str, len);
    return str;
}

/* errno.c's doserrno accessor.  errno.c is not a reference TU, so this is
 * ENVIRONMENT — supplied by refimpl.c — and takes the msvcrt_ref_* import
 * namespace for the same anti-circularity reason as _errno and GetLastError:
 * the reference must never be able to reach the code under test.  Declared
 * rather than left implicit because it returns a POINTER, which the implicit
 * `int()` rule would truncate to 32 bits before the first dereference. */
#define __doserrno msvcrt_ref___doserrno
__msvcrt_ulong * CDECL __doserrno(void);

/* ── 4. stdin/stdout/stderr — the §9.66 trap in its purest form ──────────
 *
 * file.c reaches for `fgetc(stdin)`, `fputc(c, stdout)`, `_lock_file(stdin)`
 * &c. in eleven places.  Left alone, the three names resolve to glibc's
 * `extern FILE *stdin` — a `struct _IO_FILE *` this TU has just been told to
 * treat as an `_iobuf *`.  It compiles, it links, and every getchar/puts/
 * _flushall row would read a punned pointer's interior as _flag/_file.
 * Nothing about that failure is loud.
 *
 * Both halves below are VERBATIM from the real build, and neither is a
 * function call: <corecrt_wstdio.h>:50-52 spells the three over
 * `__acrt_iob_func(N)`, and dll/win32/msvcrt/msvcrt.h:331-332 — inside the
 * `_MSVCR_VER < 140` arm, the one this harness compiles — turns that accessor
 * into plain pointer arithmetic on the array file.c itself defines.  That
 * pairing is why the real file.c.obj carries no undefined `iob` reference at
 * all, and reproducing it exactly is what keeps this reference honest.
 *
 * This must run AFTER the host headers, not alongside the FILE override in
 * msvcrt_file_pre.h, and the reason is worth keeping: glibc writes
 * `extern FILE *stdin;` and only then `#define stdin stdin`.  With our macro
 * already in scope, that declaration macro-expands to
 * `extern FILE *(MSVCRT__iob+(0));` — a syntax error at the point of
 * declaration, three times over.  So let glibc declare its own, and displace
 * it here. */
#undef stdin
#undef stdout
#undef stderr
extern FILE MSVCRT__iob[];
#define __acrt_iob_func(idx) (MSVCRT__iob+(idx))
#define stdin  (__acrt_iob_func(0))
#define stdout (__acrt_iob_func(1))
#define stderr (__acrt_iob_func(2))

#endif /* MSVCRT_DIFFTEST_FILE_PRELUDE_H */
