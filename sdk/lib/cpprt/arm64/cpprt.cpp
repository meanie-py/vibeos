/*
 * PROJECT:         ReactOS CRT library
 * LICENSE:         GPL-2.0-or-later
 * PURPOSE:         C++ runtime stubs for ARM64
 * COPYRIGHT:       Copyright 2024 ReactOS ARM64 Port
 */

/* Use malloc/free directly to avoid circular dependencies */
extern "C" {
    void* malloc(unsigned long long size);
    void free(void* ptr);
}

/*
 * C++ operator overloads
 * When building with Clang/libc++, these may conflict with the libc++ versions.
 * Use weak linkage so libc++ can override them when linked.
 */
#ifdef __clang__
#define WEAK_SYMBOL __attribute__((weak))
#else
#define WEAK_SYMBOL
#endif

WEAK_SYMBOL void* operator new(unsigned long long size)
{
    return malloc(size);
}

WEAK_SYMBOL void* operator new[](unsigned long long size)
{
    return malloc(size);
}

WEAK_SYMBOL void operator delete(void* ptr)
{
    free(ptr);
}

WEAK_SYMBOL void operator delete[](void* ptr)
{
    free(ptr);
}

/* Sized deallocation (C++14) */
WEAK_SYMBOL void operator delete(void* ptr, unsigned long long)
{
    free(ptr);
}

WEAK_SYMBOL void operator delete[](void* ptr, unsigned long long)
{
    free(ptr);
}

/*
 * __cxa_guard_* functions for thread-safe static initialization
 * These are required by the Itanium C++ ABI used by GCC/Clang
 * When linking libc++abi, its versions will override these.
 */
extern "C" {

/* Guard variable structure (8 bytes on ARM64) */
struct __cxa_guard_type {
    char initialized;
    char reserved[7];
};

/* Acquire the guard - returns 1 if initialization should proceed, 0 if already done */
WEAK_SYMBOL int __cxa_guard_acquire(__cxa_guard_type* guard)
{
    /* Simple implementation without threading support for now */
    if (guard->initialized)
        return 0;
    return 1;
}

/* Release the guard after successful initialization */
WEAK_SYMBOL void __cxa_guard_release(__cxa_guard_type* guard)
{
    guard->initialized = 1;
}

/* Abort initialization (called if constructor throws) */
WEAK_SYMBOL void __cxa_guard_abort(__cxa_guard_type* guard)
{
    guard->initialized = 0;
}

} /* extern "C" */
