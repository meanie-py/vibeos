/* Shared by the DirectX modrun probes (d3d8, d3d9, ddraw, dinput, dsound).
 *
 * Each probe is staged twice. Named <mod>_c.exe it loads <mod>_c.dll (the pre-flip C that
 * c_oracle.cmake builds) from its own directory; under any other name it loads the system
 * <mod>.dll (the Rust). Both runs print VALUES only -- HRESULTs, counts, sizes, caps bits,
 * pixel checksums -- never a pointer or handle, so the two outputs diff byte for byte. */
#ifndef DX_PROBE_H
#define DX_PROBE_H
#define COBJMACROS
#define INITGUID
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static HMODULE dx_dll;

/* `dll` is the module's file stem ("d3d9"). Exits 2 when it cannot load. */
static inline void dx_load(const wchar_t *dll)
{
    wchar_t self[MAX_PATH], name[64];
    /* Unbuffered: a crash then cuts the output at the call that crashed, not at a 4K boundary. */
    setvbuf(stdout, NULL, _IONBF, 0);
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        wchar_t *slash = wcsrchr(self, L'\\');
        lstrcpyW(name, dll);
        lstrcatW(name, L"_c.dll");
        lstrcpyW(slash + 1, name);
        dx_dll = LoadLibraryW(self);
    }
    else
    {
        lstrcpyW(name, dll);
        lstrcatW(name, L".dll");
        dx_dll = LoadLibraryW(name);
    }
    if (!dx_dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        fflush(stdout);
        ExitProcess(2);
    }
}

/* d3d8, d3d9 and ddraw sit on wined3d (d3dwine.dll), so each variant must run on ITS wined3d:
 * the C one staged as d3dwine.dll beside a _c.exe (wined3d is not a KnownDLL, so the exe's
 * directory wins), the system one -- the Rust, once wined3d flips -- under the other name.
 * Without this check a C variant that silently picked up the system wined3d would compare the
 * Rust with itself and pass. The output is the same line either way, so the diff holds. */
static inline void dx_wined3d(void)
{
    wchar_t self[MAX_PATH], lib[MAX_PATH];
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    HMODULE h = GetModuleHandleW(L"d3dwine.dll");
    wchar_t *a, *b;
    int c_variant = len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe");
    if (!h || !GetModuleFileNameW(h, lib, MAX_PATH))
    {
        printf("wined3d not loaded\n");
        ExitProcess(2);
    }
    a = wcsrchr(self, L'\\');
    b = wcsrchr(lib, L'\\');
    *a = *b = 0;
    if (c_variant != !lstrcmpiW(self, lib))
    {
        printf("wined3d from the wrong directory (%s variant)\n", c_variant ? "c" : "rust");
        ExitProcess(2);
    }
    printf("wined3d own\n");
}

static inline FARPROC dx_proc(const char *name)
{
    FARPROC f = GetProcAddress(dx_dll, name);
    if (!f)
    {
        printf("missing export %s\n", name);
        fflush(stdout);
        ExitProcess(2);
    }
    return f;
}

/* A hidden top-level window: device creation and cooperative levels need one. */
static inline HWND dx_window(void)
{
    return CreateWindowExA(0, "STATIC", "dxprobe", WS_OVERLAPPEDWINDOW, 0, 0, 640, 480, NULL,
                           NULL, GetModuleHandleA(NULL), NULL);
}

/* A struct as 32-bit words: floats print as their bits, so the diff is exact. */
static inline void dx_words(const char *tag, const void *p, size_t size)
{
    const DWORD *w = (const DWORD *)p;
    size_t i;
    printf("%s", tag);
    for (i = 0; i < size / 4; i++)
        printf("%s%lx", i % 8 ? " " : (i ? "\n  " : " "), (unsigned long)w[i]);
    printf("\n");
}

/* FNV-1a over bytes, for surface contents. */
static inline unsigned long dx_hash(const void *p, size_t size)
{
    const unsigned char *b = (const unsigned char *)p;
    unsigned long h = 2166136261u;
    while (size--)
        h = (h ^ *b++) * 16777619u;
    return h;
}

/* wined3d's renderer, from the case's argument. The default is no3d: it needs no GPU, so the
 * front end's logic is graded on any guest. `gl` selects the GL renderer, which only works
 * where the suite stages a modern GL beside the exe (tools/mesa/build-mesa.sh: the system
 * opengl32 is GL 1.1, and wined3d refuses it); that case's `expect` then demands a created
 * device, so a quiet fall back to no3d fails instead of grading nothing. wined3d reads this
 * from msvcrt's environment, shared with this exe, and is the same C under both variants. */
/* `gl invalid`: only the out-of-range calls (dx_invalid), which each suite grades in a case of
 * their own -- the C oracle reads past its state arrays on them, and d3d8's faults. */
static int dx_invalid, dx_gl, dx_trace;

/* `trace` only (never a graded case: the numbers differ by variant): this process's address
 * space, so a teardown that leaks reservations shows as a shrinking largest free region. */
static inline void dx_vm(const char *tag)
{
    MEMORY_BASIC_INFORMATION mbi;
    SIZE_T commit = 0, reserve = 0, free_max = 0, alloc_size = 0, big = 0;
    void *alloc = NULL, *big_base = NULL;
    HANDLE heaps[64];
    DWORD nheaps, i, regions = 0, in_heap = 0;
    char *p = NULL;

    if (!dx_trace)
        return;
    while (VirtualQuery(p, &mbi, sizeof(mbi)) == sizeof(mbi))
    {
        /* The largest single reservation (an allocation's regions summed), and its base. */
        if (mbi.State != MEM_FREE)
        {
            if (mbi.AllocationBase != alloc)
            {
                alloc = mbi.AllocationBase;
                alloc_size = 0;
                regions++;
            }
            alloc_size += mbi.RegionSize;
            if (alloc_size > big)
            {
                big = alloc_size;
                big_base = alloc;
            }
        }
        if (mbi.State == MEM_COMMIT)
            commit += mbi.RegionSize;
        else if (mbi.State == MEM_RESERVE)
            reserve += mbi.RegionSize;
        else if (mbi.State == MEM_FREE && mbi.RegionSize > free_max)
            free_max = mbi.RegionSize;
        if ((char *)mbi.BaseAddress + mbi.RegionSize <= p)
            break;
        p = (char *)mbi.BaseAddress + mbi.RegionSize;
    }
    nheaps = GetProcessHeaps(64, heaps);
    for (i = 0; i < nheaps && i < 64; i++)
        if ((void *)heaps[i] == big_base)
            in_heap = i + 1;
    printf("vm %s commit %luM reserve %luM largest free %luM allocations %lu heaps %lu"
           " biggest %luM at %p heap#%lu\n", tag, (unsigned long)(commit >> 20),
           (unsigned long)(reserve >> 20), (unsigned long)(free_max >> 20), regions, nheaps,
           (unsigned long)(big >> 20), big_base, in_heap);
}

static inline void dx_renderer(int argc, char **argv)
{
    int gl = dx_gl = argc > 1 && !strcmp(argv[1], "gl");
    dx_invalid = argc > 2 && !strcmp(argv[2], "invalid");
    /* `trace` (any position): wined3d's own TRACE/WARN on the serial line, through libwine's
     * DEBUGCHANNEL, which each module reads from this process's environment at its first
     * debug call. A diagnosis switch only; no graded case passes it. */
    for (int i = 1; i < argc; i++)
        if (!strcmp(argv[i], "trace") && (dx_trace = 1))
            SetEnvironmentVariableA("DEBUGCHANNEL", "+d3d,+d3d_caps,+wgl,+opengl");
    _putenv(gl ? "WINE_D3D_CONFIG=renderer=gl" : "WINE_D3D_CONFIG=renderer=no3d");
    printf("renderer %s\n", gl ? "gl" : "no3d");
}

#define HR(tag, expr) printf("%s hr=%08lx\n", tag, (unsigned long)(HRESULT)(expr))
#define DONE() (printf("probe done\n"), fflush(stdout), 0)
#endif
