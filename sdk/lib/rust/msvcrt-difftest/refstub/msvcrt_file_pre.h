/*
 * The file.c reference TU's FIRST header — it must precede every other line of
 * the prelude, including the union `ref_prelude.h`.
 *
 * Everything here works by claiming glibc's own include guards before anything
 * can pull <stdio.h>.  That only works if nothing has pulled it yet, which is
 * why this is a separate file from msvcrt_file_prelude.h (which lands AFTER
 * the union prelude, because it needs the Windows types the union's
 * refstub/msvcrt.h defines).  Same trick refstub/msvcrt.h already uses for
 * wint_t and mbstate_t.
 */
#ifndef MSVCRT_DIFFTEST_FILE_PRE_H
#define MSVCRT_DIFFTEST_FILE_PRE_H

/* ── msvcrt's FILE must win over glibc's ─────────────────────────────────
 * file.c reads and writes FILE's members directly, so msvcrt's `struct _iobuf`
 * LAYOUT is part of the contract under test — where glibc's <stdio.h> would
 * hand out `struct _IO_FILE`.  Claiming the guards makes glibc skip its own
 * definitions and declare its stdio entry points against ours.  That is a
 * compile-time convenience only: an msvcrt FILE must never be handed to a
 * glibc stdio function at runtime, and does not need to be — file.c's
 * reference implements the whole stream layer itself.
 *
 * Layout from dll/win32/msvcrt/include/corecrt_wstdio.h, which is where FILE
 * comes from at _MSVCR_VER=0 — NOT from dll/win32/msvcrt/msvcrt.h, whose
 * `_MSVCR_VER >= 140` block declares the same eight fields in a DIFFERENT
 * ORDER (_ptr, _base, _cnt, … there; _ptr, _cnt, _base, … here) and is not
 * compiled.  The first draft of this file took that block's order and was
 * wrong in a way no test would have named: every _cnt read would have returned
 * a pointer's low half.  msvcrt_crt_layout.h asserts all eight offsets, which
 * is what caught it. */
#define __FILE_defined 1
#define __struct_FILE_defined 1
#define _FILE_DEFINED 1
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

/* fpos_t needs a different lever.  msvcrt's is a plain 64-bit integer (file.c
 * does `*pos == -1` on it); glibc's is a struct wrapping __off64_t and a
 * mbstate_t.  But glibc's `typedef __fpos_t fpos_t;` has NO guard of its own,
 * so claiming one cannot suppress it — instead claim the guard on the
 * UNDERLYING type and define that, and glibc's own typedef then lands on ours.
 * `long long`, not `__int64`, because this runs before <msvcrt.h> has defined
 * the MSVC spelling. */
#define _____fpos_t_defined 1
#define _____fpos64_t_defined 1
typedef long long __fpos_t;
typedef long long __fpos64_t;

/* stdin/stdout/stderr are the other half of this problem, but they CANNOT be
 * fixed here — see the block at the end of msvcrt_file_prelude.h, which runs
 * after the host headers instead. */

#endif /* MSVCRT_DIFFTEST_FILE_PRE_H */
