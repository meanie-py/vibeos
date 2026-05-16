/* Minimal <stdio.h> for isolating the genuine ReactOS printf machinery
 * (printf/streamout.c + _sxprintf.c) on the host. The CRT's `sprintf`
 * builds an in-memory `FILE` and `streamout_char` writes into it via
 * the `_LIBCNT_` branch (refstub precomp sets `_LIBCNT_`). The host
 * glibc <stdio.h> FILE is opaque, so we supply the genuine ReactOS
 * `struct _iobuf` (member names + types are all _sxprintf.c/streamout.c
 * touch: _base/_ptr/_charbuf/_cnt/_bufsiz/_flag/_tmpfname) and the two
 * stream flags. No host stdio is pulled. */
#ifndef CNT_REFSTUB_STDIO_H
#define CNT_REFSTUB_STDIO_H

#include <stddef.h> /* size_t, NULL */

typedef struct _iobuf {
    char *_ptr;
    int _cnt;
    char *_base;
    int _flag;
    int _file;
    int _charbuf;
    int _bufsiz;
    char *_tmpfname;
} FILE;

/* Genuine msvcrt values (only the bits streamout_char tests matter;
 * the `_LIBCNT_` path just checks `_cnt` and writes via `_ptr`). */
#define _IOWRT  0x0002
#define _IOSTRG 0x0040

#ifndef EOF
#define EOF (-1)
#endif

#endif
