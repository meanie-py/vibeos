/* Shared host environment for grading dll/nls/normaliz (IdnToAscii.c, IdnToUnicode.c).
 * Linked into BOTH the genuine C and the Rust candidate, so these definitions are the
 * world both sides see; they are not under grade. A last-error cell, a malloc-backed
 * process heap that counts live blocks, 16-bit wcslen/towlower (host glibc's are 4-byte),
 * and a FoldStringW model. The probe reports (live blocks << 32) | last error, so a
 * different error code or a leaked block on any path is a mismatch. */
#include <stdint.h>
#include <stdlib.h>
#include "normaliz_env.h"

typedef unsigned short WCHAR;

static uint32_t env_last_error;
static uint32_t env_live_blocks;

void SetLastError(uint32_t code) { env_last_error = code; }
uint32_t GetLastError(void) { return env_last_error; }

void *GetProcessHeap(void) { return (void *)0x1000; }
void *HeapAlloc(void *heap, uint32_t flags, size_t size)
{
    void *p = (flags & 0x8 /* HEAP_ZERO_MEMORY */) ? calloc(1, size ? size : 1) : malloc(size ? size : 1);
    (void)heap;
    if (p) env_live_blocks++;
    return p;
}
int HeapFree(void *heap, uint32_t flags, void *mem)
{
    (void)heap; (void)flags;
    if (mem) env_live_blocks--;
    free(mem);
    return 1;
}

size_t wcslen(const WCHAR *s) { const WCHAR *p = s; while (*p) p++; return p - s; }
/* The IDN inputs reaching towlower are ASCII (IdnToUnicode rejects > 0x7f first). */
unsigned short towlower(unsigned short c) { return (c >= 'A' && c <= 'Z') ? c + 32 : c; }

/* FoldStringW(MAP_FOLDCZONE, ...): the fullwidth ASCII block folds to ASCII, everything
 * else is itself. A zero srclen is ERROR_INVALID_PARAMETER and a short destination
 * ERROR_INSUFFICIENT_BUFFER, as in kernel32. */
int FoldStringW(uint32_t flags, const WCHAR *src, int srclen, WCHAR *dst, int dstlen)
{
    int i;
    (void)flags;
    if (srclen == 0 || dstlen < 0) { env_last_error = 87; return 0; }
    if (srclen < 0) srclen = (int)wcslen(src) + 1;
    if (!dstlen) return srclen;
    if (dstlen < srclen) { env_last_error = 122; return 0; }
    for (i = 0; i < srclen; i++)
        dst[i] = (src[i] >= 0xff01 && src[i] <= 0xff5e) ? src[i] - 0xfee0 : src[i];
    return srclen;
}

/* c_env_probe */
void normaliz_env_reset(void) { env_last_error = 0xfeedf00d; env_live_blocks = 0; }
uint64_t normaliz_env_read(void) { return ((uint64_t)env_live_blocks << 32) | env_last_error; }
