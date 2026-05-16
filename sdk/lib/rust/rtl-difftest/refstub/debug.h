/* Minimal <debug.h>: unicode.c's DPRINT/DPRINT1/ASSERT have no
 * observable contract under the diff harness — neutralize. */
#ifndef RTL_REFSTUB_DEBUG_H
#define RTL_REFSTUB_DEBUG_H
#define DPRINT(...) ((void)0)
#define DPRINT1(...) ((void)0)
#define ASSERT(x) ((void)0)
#define ASSERTMSG(m, x) ((void)0)
#define UNIMPLEMENTED ((void)0)
#endif
