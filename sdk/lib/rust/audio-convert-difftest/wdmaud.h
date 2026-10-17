/* Host fixture for wdmaud.drv's converters: the same bodies as kmixer's, on the
 * process heap with Win32 error codes. Allocation goes through the Rust test's allocator. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>
typedef uint32_t DWORD;
typedef uint32_t ULONG, *PULONG;
typedef uint16_t USHORT, *PUSHORT;
typedef uint8_t UCHAR, *PUCHAR;
typedef void *PVOID;
#define ERROR_SUCCESS 0u
#define ERROR_NOT_ENOUGH_MEMORY 8u
#define ERROR_NOT_SUPPORTED 50u
void *fixture_allocate(uint32_t, size_t);
#define GetProcessHeap() ((PVOID)0)
#define HeapAlloc(heap, flags, size) fixture_allocate(0, (size))
#define RtlMoveMemory memmove
#define ASSERT(x) ((void)0)
#define DPRINT1(...) ((void)0)
#define SND_TRACE(...) ((void)0)
#define _byteswap_ushort __builtin_bswap16
#define _byteswap_ulong __builtin_bswap32
