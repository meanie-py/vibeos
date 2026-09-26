/* Stand-in for wine/debug.h.
 *
 * The backend's TRACE lines are the only narration we get when a handshake goes
 * wrong, so they are kept and routed to stderr behind an env switch rather than
 * compiled out: SCHAN_HARNESS_TRACE=1 turns them on. */
#ifndef HARNESS_WINE_DEBUG_H
#define HARNESS_WINE_DEBUG_H

#include <stdio.h>
#include <stdlib.h>

int schan_harness_tracing(void);

#define WINE_DEFAULT_DEBUG_CHANNEL(x)
#define WINE_DECLARE_DEBUG_CHANNEL(x)

#define SCHAN_LOG(pfx, ...) \
    do { if (schan_harness_tracing()) { fprintf(stderr, pfx); fprintf(stderr, __VA_ARGS__); } } while (0)

#define TRACE(...) SCHAN_LOG("[trace] ", __VA_ARGS__)
#define WARN(...)  SCHAN_LOG("[warn ] ", __VA_ARGS__)
#define FIXME(...) SCHAN_LOG("[fixme] ", __VA_ARGS__)
#define ERR(...)   SCHAN_LOG("[err  ] ", __VA_ARGS__)

#define debugstr_a(s) (s)
#define debugstr_w(s) ((const char *)"<wide>")

#endif
