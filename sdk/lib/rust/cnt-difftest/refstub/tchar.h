/* Minimal <tchar.h> for isolating libcntpr tcs*.h logic on the host.
 * The wrapper TU pre-typedefs `_TCHAR`; here we only neutralize the
 * calling-convention macros the templates spell out. */
#ifndef CNT_REFSTUB_TCHAR_H
#define CNT_REFSTUB_TCHAR_H
#define __cdecl
#define CDECL
#endif
