/* Host fixture for kmixer's converters: the types and macros pin.c's two converter
 * bodies use, nothing else. Pool allocation goes through the Rust test's allocator. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>
typedef int32_t NTSTATUS;
typedef uint32_t ULONG, *PULONG;
typedef uint16_t USHORT, *PUSHORT;
typedef uint8_t UCHAR, *PUCHAR;
typedef void *PVOID;
#define STATUS_SUCCESS ((NTSTATUS)0)
#define STATUS_NOT_IMPLEMENTED ((NTSTATUS)0xc0000002)
#define STATUS_INSUFFICIENT_RESOURCES ((NTSTATUS)0xc000009a)
#define NonPagedPool 0
void *fixture_allocate(uint32_t, size_t);
#define ExAllocatePool fixture_allocate
#define RtlMoveMemory memmove
#define ASSERT(x) ((void)0)
#define DPRINT1(...) ((void)0)
#define _byteswap_ushort __builtin_bswap16
#define _byteswap_ulong __builtin_bswap32
