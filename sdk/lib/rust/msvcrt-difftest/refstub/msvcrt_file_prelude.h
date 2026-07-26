/*
 * The static half of the file.c reference-TU prelude.
 *
 * file.c cannot ride the shared ref_prelude.h for one reason: it reads and
 * writes FILE's members directly (_ptr/_cnt/_base/_flag/_file/_charbuf/
 * _bufsiz/_tmpfname), so msvcrt's `struct _iobuf` LAYOUT is part of the
 * contract under test — where the shared prelude hands out glibc's
 * `struct _IO_FILE`.  scanf.c already set the precedent that a TU with its own
 * type model gets its own prelude (ref_scanf_prelude.h).
 *
 * build.rs force-includes a GENERATED prelude that pulls this file and then
 * adds the `#define <name> ref_<name>` rename wall.  Everything that is not a
 * rename lives here, so it is reviewable as source rather than as a string
 * literal inside build.rs.
 *
 * Order below is load-bearing and each step says why.
 */
#ifndef MSVCRT_DIFFTEST_FILE_PRELUDE_H
#define MSVCRT_DIFFTEST_FILE_PRELUDE_H

/* ── 1. msvcrt's FILE must win over glibc's ──────────────────────────────
 * Claim glibc's own guards BEFORE anything can pull <stdio.h>, exactly the
 * trick refstub/msvcrt.h already uses for wint_t and mbstate_t.  glibc then
 * skips its FILE definitions and declares its stdio entry points against
 * ours.  That is a compile-time convenience only: an msvcrt FILE must never
 * be handed to a glibc stdio function at runtime, and does not need to be —
 * file.c's reference implements the whole stream layer itself.
 *
 * Layout from dll/win32/msvcrt/include/corecrt_wstdio.h, which is where FILE
 * comes from at _MSVCR_VER=0 — NOT from dll/win32/msvcrt/msvcrt.h, whose
 * `_MSVCR_VER >= 140` block declares the same eight fields in a DIFFERENT
 * ORDER (_ptr, _base, _cnt, … there; _ptr, _cnt, _base, … here) and is not
 * compiled.  The first draft of this file took that block's order and was
 * wrong in a way no test would have named: every _cnt read would have
 * returned a pointer's low half.  msvcrt_crt_layout.h asserts all eight
 * offsets, which is what caught it. */
#define __FILE_defined 1
#define __struct_FILE_defined 1
#define _FILE_DEFINED 1
/* Same reasoning for fpos_t: msvcrt's is a plain 64-bit integer, glibc's is a
 * struct wrapping __off64_t and a mbstate_t.  file.c's fgetpos/fsetpos compare
 * and assign it as a scalar. */
/* fpos_t needs a different lever.  msvcrt's is a plain 64-bit integer (file.c
 * does `*pos == -1` on it); glibc's is a struct wrapping __off64_t and a
 * mbstate_t.  But glibc's `typedef __fpos_t fpos_t;` has NO guard of its own,
 * so claiming one cannot suppress it — instead claim the guard on the
 * UNDERLYING type and define that, and glibc's own typedef then lands on
 * ours.  `long long`, not `__int64`, because this runs before <msvcrt.h> has
 * defined the MSVC spelling. */
#define _____fpos_t_defined 1
#define _____fpos64_t_defined 1
typedef long long __fpos_t;
typedef long long __fpos64_t;
#pragma pack(push, 8)
typedef struct _iobuf {
  char* _ptr;
  int   _cnt;
  char* _base;
  int   _flag;
  int   _file;
  int   _charbuf;
  int   _bufsiz;
  char* _tmpfname;
} FILE;
#pragma pack(pop)

/* ── 2. the shared refstub ───────────────────────────────────────────────
 * MSVC-width wint_t/mbstate_t, the host libc headers, the Windows base type
 * set (BOOL/DWORD/HANDLE/LONG/…), thread_data_t, and the msvcrt externals.
 * Claims __WINE_MSVCRT_H, so file.c's own `#include "msvcrt.h"` — which would
 * otherwise resolve to the real same-dir header and drag in the whole NT/PSDK
 * chain — expands to nothing. */
#include <msvcrt.h>

/* ── 3. the CRT/Win32 types file.c needs, and the proof they are right ───
 * Types are transcribed (a layout is not an integer); the guardian that
 * follows is generated from what the pinned cross-compiler measured against
 * msvcrt's real headers.  Include order matters: the assertions reference the
 * definitions. */
#include <msvcrt_file_types.h>
#include <msvcrt_crt_layout.h>

/* ── 4. the constants, every value computed by the cross-compiler ─────── */
#include <msvcrt_crt_consts.h>

/* ── 5. msvcrt internals the real msvcrt.h supplies at _MSVCR_VER=0 ─────
 * These live in the part of dll/win32/msvcrt/msvcrt.h that step 2 suppressed,
 * so they are restored here rather than left to C's implicit-declaration rule.
 * Values are from that header; they are plain spellings of the _IO* bits and
 * of file.c's own fd sentinels, not independent constants. */
#define MSVCRT__NOBUF     _IONBF
#define MSVCRT__USERBUF   0x0100
#define MSVCRT__IOCOMMIT  0x4000
#define MSVCRT_NO_CONSOLE_FD (-2)
#define MSVCRT_NO_CONSOLE ((HANDLE)MSVCRT_NO_CONSOLE_FD)

extern unsigned int MSVCRT__commode;

/* errno.c's doserrno accessor.  errno.c is not a reference TU, so this is
 * ENVIRONMENT — supplied by refimpl.c — and takes the msvcrt_ref_* import
 * namespace for the same anti-circularity reason as _errno and GetLastError:
 * the reference must never be able to reach the code under test.  Declared
 * rather than left implicit because it returns a POINTER, which the implicit
 * `int()` rule would truncate to 32 bits before the first dereference. */
#define __doserrno msvcrt_ref___doserrno
__msvcrt_ulong * CDECL __doserrno(void);

#endif /* MSVCRT_DIFFTEST_FILE_PRELUDE_H */
