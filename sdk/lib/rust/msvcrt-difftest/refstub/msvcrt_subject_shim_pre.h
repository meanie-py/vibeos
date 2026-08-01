/*
 * Part 1 of 2 of the SUBJECT-side prelude for
 * dll/win32/msvcrt/rust_file_varargs_shim.c — the twenty C-variadic entry
 * points file.rs cannot express on pinned stable Rust.  This half must land
 * before ANY host header; part 2 (msvcrt_subject_shim.h) lands after them.
 * build.rs generates the wall between the two and force-includes the lot.
 *
 * WHY THE SHIM IS COMPILED INTO THE HARNESS AT ALL.  Three of its bodies are on
 * the path under test, not beside it: file.rs's `_wfsopen` calls `_wsopen` (so
 * every `fopen`/`_wfopen` row runs through it), and `_creat`/`_wcreat` call
 * `_open`/`_wopen`.  Rust DECLARES those three and defines none of them, so
 * without this TU the subject does not link at all.  A hand-written host copy of
 * the four bodies would be a THIRD transcription of file.c's originals with
 * nothing diffing it — and tools/scan_varargs_shim_parity.py already diffs THIS
 * file against file.c, so compiling the real one is what keeps the harness
 * graded against exactly what ships.
 *
 * SUBJECT SIDE, so nothing here may reach refstub/msvcrt.h: that header carries
 * the anti-circularity `msvcrt_ref_*` namespace, and pulling it would bind the
 * code under test to the reference's state.  subjectimpl.c follows the same rule
 * and says so at more length.
 *
 * Only SPELLINGS have to agree here, not layouts: the shim dereferences no FILE
 * and does no wchar_t arithmetic — every aggregate is forwarded whole.  msvcrt's
 * FILE arrives anyway, because glibc would otherwise hand out `struct _IO_FILE`
 * and the shim's prototypes would then disagree with the ones file.rs exports.
 */
#ifndef MSVCRT_DIFFTEST_SUBJECT_SHIM_PRE_H
#define MSVCRT_DIFFTEST_SUBJECT_SHIM_PRE_H

/* Must precede every host header — it works by claiming glibc's include guards. */
#include <msvcrt_file_pre.h>

/* The shim's `#include "msvcrt.h"` is a QUOTED include, so it resolves next to
 * the shim itself: dll/win32/msvcrt/msvcrt.h, which drags in the whole PSDK
 * chain and cannot share a translation unit with glibc.  Claim its guard and it
 * expands to nothing; the handful of decorations it would have supplied are
 * below.  Same lever refstub/corecrt.h pulls on __WINE_CORECRT_H, and
 * msvcrt_file_pre.h on glibc's. */
#define __WINE_MSVCRT_H

/* Calling-convention decorations, all no-ops on a SysV host. */
#define CDECL
#define WINAPIV
#ifndef __cdecl
#define __cdecl
#endif

/* Opaque: the shim only forwards it to the `_l` bodies in file.rs. */
typedef void *_locale_t;

/* _O_CREAT and _SH_DENYNO — the two constants the open-family bodies branch on.
 * Measured by the pinned cross-compiler against msvcrt's real include set, not
 * transcribed here, for the reason that header's banner gives. */
#include <msvcrt_crt_consts.h>

#endif /* MSVCRT_DIFFTEST_SUBJECT_SHIM_PRE_H */
