/* The names sdk/lib/rtl/compress.c needs beyond rtl-difftest's refstub <rtl.h>, with the
   genuine values: psdk/ntstatus.h (the statuses), xdk/rtltypes.h (formats/engines). */
#include <rtl.h>
#define STATUS_BAD_COMPRESSION_BUFFER ((NTSTATUS)0xC0000242)
#define STATUS_UNSUPPORTED_COMPRESSION ((NTSTATUS)0xC000025F)
#define STATUS_NOT_SUPPORTED ((NTSTATUS)0xC00000BB)
#define COMPRESSION_FORMAT_NONE (0x0000)
#define COMPRESSION_FORMAT_DEFAULT (0x0001)
#define COMPRESSION_FORMAT_LZNT1 (0x0002)
#define COMPRESSION_ENGINE_STANDARD (0x0000)
#define COMPRESSION_ENGINE_MAXIMUM (0x0100)
typedef struct _COMPRESSED_DATA_INFO *PCOMPRESSED_DATA_INFO;
