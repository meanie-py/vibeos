/*
 * Part 2 of 2 of the SUBJECT-side prelude for the varargs shim.  Part 1 is
 * msvcrt_subject_shim_pre.h; read its banner first.
 *
 * This half lands AFTER the host headers, because everything in it has to
 * displace something glibc declared.
 */
#ifndef MSVCRT_DIFFTEST_SUBJECT_SHIM_H
#define MSVCRT_DIFFTEST_SUBJECT_SHIM_H

/* ── stdout/stderr, on the SUBJECT's array ───────────────────────────────
 * The printf shells spell their destination `stdout`.  msvcrt has no such
 * variable: <corecrt_wstdio.h> spells the three over `__acrt_iob_func(N)`, and
 * dll/win32/msvcrt/msvcrt.h — inside the `_MSVCR_VER < 140` arm, the one this
 * harness compiles — turns that accessor into plain pointer arithmetic on the
 * `MSVCRT__iob` array.  file.rs exports that array, so the subject's stdout is
 * the subject's, and cannot reach across to `ref_MSVCRT__iob`.
 *
 * Note the un-prefixed name, which is the whole point of this file being
 * separate from the reference's msvcrt_file_prelude.h: that one renames the
 * array into ref_* so the two implementations cannot share one FILE table.
 *
 * The #undefs are load-bearing for the same reason they are over there: glibc
 * writes `extern FILE *stdout;` and only then `#define stdout stdout`, so with
 * our macro already in scope its own declaration would macro-expand at the
 * point of declaration and fail to parse.  Let glibc declare its own; displace
 * it here. */
#undef stdin
#undef stdout
#undef stderr
extern FILE MSVCRT__iob[];
#define stdin  (MSVCRT__iob + 0)
#define stdout (MSVCRT__iob + 1)
#define stderr (MSVCRT__iob + 2)

#endif /* MSVCRT_DIFFTEST_SUBJECT_SHIM_H */
