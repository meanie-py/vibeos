/*
 * <pshpack8.h> — 8-byte struct packing, pushed.
 *
 * The PSDK's own copy is unreachable from a reference TU: putting the PSDK on
 * the include path would also capture refstub/msvcrt.h's `#include <assert.h>`
 * / `<ctype.h>`, which must resolve to the HOST's headers (that is the whole
 * point of refstub/msvcrt.h).  So this is the one-line pragma, spelled out.
 *
 * It is NOT a no-op, deliberately.  <mbstring.h> declares structs between
 * pshpack8 and poppack; dropping the pragma would silently give them host
 * default alignment, and a layout the reference disagrees with is exactly the
 * kind of difference a shared fixture hides.
 */
#if !defined(RC_INVOKED)
#pragma pack(push, 8)
#endif
