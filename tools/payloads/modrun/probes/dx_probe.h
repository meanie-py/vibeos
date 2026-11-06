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
static int dx_invalid;

static inline void dx_renderer(int argc, char **argv)
{
    int gl = argc > 1 && !strcmp(argv[1], "gl");
    dx_invalid = argc > 2 && !strcmp(argv[2], "invalid");
    _putenv(gl ? "WINE_D3D_CONFIG=renderer=gl" : "WINE_D3D_CONFIG=renderer=no3d");
    printf("renderer %s\n", gl ? "gl" : "no3d");
}

#define HR(tag, expr) printf("%s hr=%08lx\n", tag, (unsigned long)(HRESULT)(expr))
#define DONE() (printf("probe done\n"), fflush(stdout), 0)
#endif
