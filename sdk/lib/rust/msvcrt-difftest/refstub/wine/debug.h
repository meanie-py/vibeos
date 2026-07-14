/*
 * Minimal <wine/debug.h>.  string.c only TRACE()s / FIXME()s; on ReactOS
 * those route through ros_dbg_log()/wine_dbgstr_an().  No-op them: the
 * differential harness has no serial port, and debug output is not part
 * of any value contract under test.  (This is also why ros_dbg_log /
 * wine_dbgstr_an — two of the 19 externals of the real string.c.obj —
 * end up unreferenced here; refimpl.c still defines them, harmlessly.)
 */
#ifndef __WINE_DEBUG_H
#define __WINE_DEBUG_H

#define WINE_DEFAULT_DEBUG_CHANNEL(ch)
#define WINE_DECLARE_DEBUG_CHANNEL(ch)
#define TRACE(...)      ((void)0)
#define TRACE_(ch)      ((void)0)
#define WARN(...)       ((void)0)
#define FIXME(...)      ((void)0)
#define ERR(...)        ((void)0)
#define TRACE_ON(ch)    0
#define WARN_ON(ch)     0
#define ERR_ON(ch)      0
#define debugstr_a(s)   (s)
#define debugstr_w(s)   (s)
#define debugstr_an(s,n) (s)
#define wine_dbgstr_a(s) (s)

#endif /* __WINE_DEBUG_H */
