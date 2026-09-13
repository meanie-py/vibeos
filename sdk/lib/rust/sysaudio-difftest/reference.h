#include <stdint.h>
#include <stddef.h>
#include <string.h>
#define IN
#define OUT
typedef int32_t NTSTATUS;
typedef uint32_t ULONG;
typedef uint16_t WCHAR;
typedef void *PVOID;
typedef struct { uint8_t bytes[64]; } KSPIN_CONNECT, *PKSPIN_CONNECT;
typedef struct { uint16_t Length, MaximumLength; WCHAR *Buffer; } UNICODE_STRING;
typedef struct { UNICODE_STRING FileName; } FILE_OBJECT;
typedef struct { FILE_OBJECT *FileObject; } IO_STACK_LOCATION, *PIO_STACK_LOCATION;
typedef struct { PIO_STACK_LOCATION Stack; } IRP, *PIRP;
#define STATUS_SUCCESS ((NTSTATUS)0)
#define STATUS_UNSUCCESSFUL ((NTSTATUS)0xc0000001)
#define STATUS_INSUFFICIENT_RESOURCES ((NTSTATUS)0xc000009a)
#define NonPagedPool 0
#define IoGetCurrentIrpStackLocation(irp) ((irp)->Stack)
#define RtlMoveMemory memmove
void *fixture_allocate(uint32_t, size_t);
#define AllocateItem fixture_allocate
static size_t fixture_wcslen(const WCHAR *s) { size_t n = 0; while (s[n]) ++n; return n; }
#define wcslen fixture_wcslen
