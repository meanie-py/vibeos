/* Wrapper header for bindgen — internal kernel MM types */
#define _NTOSKRNL_
#define _NTSYSTEM_
#undef FORCEINLINE
#define FORCEINLINE

/* Need the full ntoskrnl internal headers */
#include <ntoskrnl.h>
#include "mm/ARM3/miarm.h"
