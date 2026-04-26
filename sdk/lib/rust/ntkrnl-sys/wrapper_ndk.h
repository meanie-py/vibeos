/* Wrapper header for bindgen — public NDK kernel API */
#define _NTOSKRNL_
#define _NTSYSTEM_
#define NO_RTL_INLINES
/* Prevent FORCEINLINE expansion (bindgen can't handle inline bodies) */
#undef FORCEINLINE
#define FORCEINLINE

#include <ndk/ntndk.h>
