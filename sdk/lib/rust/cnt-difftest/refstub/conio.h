/* Minimal <conio.h>: winesup.h #includes it but the _LIBCNT_
 * (string-buffer) scanf instantiations never use _getch/_ungetch
 * (those are the CONSOLE path, gated out by _LIBCNT_). Empty. */
#ifndef CNT_REFSTUB_CONIO_H
#define CNT_REFSTUB_CONIO_H
#endif
