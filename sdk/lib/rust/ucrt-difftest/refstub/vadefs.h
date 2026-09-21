#ifndef _INC_VADEFS
#define _INC_VADEFS
#include <stdarg.h>
#define _ADDRESSOF(v) (&(v))
#define _crt_va_start(ap, x) va_start(ap, x)
#define _crt_va_arg(ap, t)   va_arg(ap, t)
#define _crt_va_end(ap)      va_end(ap)
#define _crt_va_copy(d, s)   va_copy(d, s)
#endif
