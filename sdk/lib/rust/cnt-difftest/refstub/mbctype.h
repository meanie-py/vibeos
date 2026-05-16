/* Minimal <mbctype.h>: the only user (_tsplitpath_x.h's _ismbblead)
 * sits inside the _LIBCNT_-dead MBCS block, so this header just needs
 * to resolve the #include. _ismbblead itself is additionally stubbed
 * to 0 in the splitpath ref-unit prelude (defensive — never reached). */
#ifndef CNT_REFSTUB_MBCTYPE_H
#define CNT_REFSTUB_MBCTYPE_H
#endif
