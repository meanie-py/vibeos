
#include <stdint.h>
#include <intrin.h>
#include <malloc.h>

// atexit is needed by libsupc++
extern int __cdecl _crt_atexit(void (__cdecl*)(void));
int __cdecl atexit(void (__cdecl* function)(void))
{
    return _crt_atexit(function);
}

void* __cdecl operator_new(size_t size)
{
    return malloc(size);
}

void _cdecl operator_delete(void *mem)
{
    free(mem);
}

#ifdef _M_IX86
void _chkesp_failed(void)
{
    __debugbreak();
}
#endif

int __cdecl __acrt_initialize_sse2(void)
{
    return 0;
}

// fma/fmaf, log2/log2f and lrint/lrintf lived here as "simplistic" bodies (an unfused
// multiply-add, log(x) * M_LOG2E, and __debugbreak()). They are ucrt-rust's c99math.rs
// now, over the libm crate.
