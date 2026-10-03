/* Host stand-in for the <ntdef.h> names cryptlib's headers use. ULONG is the
   Windows LLP64 32-bit type, not host long; NTAPI is the default C ABI on x86-64. */
#pragma once
#include <stdint.h>
#include <string.h> /* the real ntdef.h chain declares memset/memcpy */
#define VOID void
#define NTAPI
typedef int32_t INT;
typedef uint8_t UCHAR;
typedef uint32_t ULONG, *PULONG;
