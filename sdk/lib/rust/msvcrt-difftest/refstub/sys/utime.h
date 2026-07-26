/*
 * Stub for file.c's `#include <utime.h>`.  msvcrt's own copy of this
 * header is MSVC-specific and cannot share a translation unit with glibc's;
 * everything the reference TU actually needs from it — the constants and the
 * struct layouts — arrives ahead of this via the force-included
 * msvcrt_file_prelude.h, derived and layout-asserted rather than transcribed.
 *
 * Deliberately EMPTY of declarations.  If file.c calls something this header
 * used to declare, the -Werror=implicit-function-declaration tripwire says so
 * by name; a plausible-looking guess here would not.
 */
#ifndef MSVCRT_DIFFTEST_SYS_UTIME_H
#define MSVCRT_DIFFTEST_SYS_UTIME_H
#endif
