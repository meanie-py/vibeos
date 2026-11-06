/* modrun probe: the guest's modern GL (Mesa softpipe, tools/mesa/build-mesa.sh), which the d3d
 * suites' device cases stand on. Not a Rust-vs-C grade: two ways of reaching the same GL must
 * agree. Both LoadLibrary opengl32 at runtime, as wined3d does -- so Mesa's implicit TLS
 * comes from the loader's dynamic-TLS path (ldr/tlsdyn.rs), not from process start -- and
 * staged as glsanity_thread.exe it makes every GL call on a thread created AFTER the load, the other half of that
 * path (LdrpInitializeThread allocating the new slot). On the Windows host both pass; on the
 * i386 guest wined3d's first immediate-mode call faulted reading NULL inside libgallium. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

#define GL_RENDERER 0x1F01
#define GL_VERSION 0x1F02
#define GL_TRIANGLE_STRIP 0x0005
#define GL_FOG_START 0x0B63
#define GL_COLOR_BUFFER_BIT 0x4000
#define GL_RGBA 0x1908
#define GL_UNSIGNED_BYTE 0x1401

typedef const char *(WINAPI *GETSTRING)(unsigned);
typedef void(WINAPI *FOGF)(unsigned, float);
typedef void(WINAPI *BEGIN)(unsigned);
typedef void(WINAPI *VOIDFN)(void);
typedef void(WINAPI *VERTEX3F)(float, float, float);
typedef void(WINAPI *COLOR3F)(float, float, float);
typedef void(WINAPI *CLEARCOLOR)(float, float, float, float);
typedef void(WINAPI *CLEAR)(unsigned);
typedef void(WINAPI *READPIXELS)(int, int, int, int, unsigned, unsigned, void *);
typedef unsigned(WINAPI *GETERROR)(void);
typedef HGLRC(WINAPI *CREATECTX)(HDC);
typedef BOOL(WINAPI *MAKECURRENT)(HDC, HGLRC);
typedef PROC(WINAPI *GETPROC)(const char *);

static HMODULE gl;
static HDC dc;
static HGLRC rc;

static FARPROC fn(const char *name)
{
    FARPROC f = GetProcAddress(gl, name);
    if (!f)
    {
        printf("missing %s\n", name);
        fflush(stdout);
        ExitProcess(2);
    }
    return f;
}

static DWORD WINAPI draw(void *unused)
{
    unsigned char px[4 * 16 * 16];
    unsigned long h = 2166136261u;
    int i;
    (void)unused;
    printf("make current %d\n", ((MAKECURRENT)fn("wglMakeCurrent"))(dc, rc));
    printf("renderer %s\n", ((GETSTRING)fn("glGetString"))(GL_RENDERER));
    fflush(stdout);
    ((FOGF)fn("glFogf"))(GL_FOG_START, 0.0f);
    printf("fog ok\n");
    fflush(stdout);
    ((CLEARCOLOR)fn("glClearColor"))(0.25f, 0.5f, 0.75f, 1.0f);
    ((CLEAR)fn("glClear"))(GL_COLOR_BUFFER_BIT);
    ((BEGIN)fn("glBegin"))(GL_TRIANGLE_STRIP);
    ((COLOR3F)fn("glColor3f"))(1.0f, 0.0f, 0.0f);
    ((VERTEX3F)fn("glVertex3f"))(-1.0f, -1.0f, 0.0f);
    ((VERTEX3F)fn("glVertex3f"))(1.0f, -1.0f, 0.0f);
    ((VERTEX3F)fn("glVertex3f"))(-1.0f, 1.0f, 0.0f);
    ((VOIDFN)fn("glEnd"))();
    printf("immediate ok err=%x\n", ((GETERROR)fn("glGetError"))());
    fflush(stdout);
    ((VOIDFN)fn("glFinish"))();
    ((READPIXELS)fn("glReadPixels"))(0, 0, 16, 16, GL_RGBA, GL_UNSIGNED_BYTE, px);
    for (i = 0; i < (int)sizeof(px); i++)
        h = (h ^ px[i]) * 16777619u;
    printf("pixels %08lx\n", h);
    printf("extension proc %d\n", ((GETPROC)fn("wglGetProcAddress"))("glProgramStringARB") != NULL);
    ((MAKECURRENT)fn("wglMakeCurrent"))(NULL, NULL);
    return 0;
}

/* One round: a fresh window, pixel format, context and draw, then all of it torn down.
 * wined3d does this per adapter init, and the d3d9 probe inits twice (Direct3DCreate9 with a
 * bad SDK version first): on the i386 guest the SECOND init's first draw faulted. Mesa's WGL
 * frees a window's framebuffer from a WH_CALLWNDPROC hook when the window goes away. */
static int round_(int n)
{
    PIXELFORMATDESCRIPTOR pfd;
    char self[MAX_PATH];
    HWND w;
    int pf;

    printf("round %d\n", n);
    w = CreateWindowExA(0, "STATIC", "gl", WS_OVERLAPPEDWINDOW, 0, 0, 64, 64, NULL, NULL, NULL, NULL);
    dc = GetDC(w);
    memset(&pfd, 0, sizeof(pfd));
    pfd.nSize = sizeof(pfd);
    pfd.nVersion = 1;
    pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    pfd.iPixelType = PFD_TYPE_RGBA;
    pfd.cColorBits = 32;
    pf = ChoosePixelFormat(dc, &pfd);
    printf("pixel format set %d\n", pf > 0 && SetPixelFormat(dc, pf, &pfd));
    rc = ((CREATECTX)fn("wglCreateContext"))(dc);
    printf("context %d\n", rc != NULL);
    if (!rc)
        return 1;
    GetModuleFileNameA(NULL, self, sizeof(self));
    if (strstr(self, "_thread"))
    {
        HANDLE t = CreateThread(NULL, 0, draw, NULL, 0, NULL);
        WaitForSingleObject(t, INFINITE);
        CloseHandle(t);
    }
    else
        draw(NULL);
    printf("delete context %d\n", ((BOOL(WINAPI *)(HGLRC))fn("wglDeleteContext"))(rc));
    ReleaseDC(w, dc);
    printf("destroy window %d\n", DestroyWindow(w));
    return 0;
}

/* wined3d's own context sequence (wined3d_caps_gl_ctx_create_attribs): a legacy context to
 * reach wglCreateContextAttribsARB, then 4.4 and 3.2 with no profile bits, each retried
 * FORWARD_COMPATIBLE on failure. On the Windows host 4.4 fails and 3.2 yields a 3.3 core
 * context in which immediate mode is GL_INVALID_OPERATION (0x502); the i386 guest's wined3d
 * instead reached a context whose immediate-mode vertex buffer was unmapped
 * (_mesa_Vertex3f wrote through NULL). Each step's result is printed. */
typedef HGLRC(WINAPI *CREATEATTRIBS)(HDC, HGLRC, const int *);
static int attribs_round(void)
{
    PIXELFORMATDESCRIPTOR pfd;
    static const int vers[][2] = {{4, 4}, {3, 2}};
    CREATEATTRIBS cca;
    HGLRC legacy, got = NULL;
    HWND w;
    int i;

    printf("attribs round\n");
    w = CreateWindowExA(0, "STATIC", "gl", WS_OVERLAPPEDWINDOW, 0, 0, 64, 64, NULL, NULL, NULL, NULL);
    dc = GetDC(w);
    memset(&pfd, 0, sizeof(pfd));
    pfd.nSize = sizeof(pfd);
    pfd.nVersion = 1;
    pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    pfd.iPixelType = PFD_TYPE_RGBA;
    pfd.cColorBits = 32;
    SetPixelFormat(dc, ChoosePixelFormat(dc, &pfd), &pfd);
    legacy = ((CREATECTX)fn("wglCreateContext"))(dc);
    ((MAKECURRENT)fn("wglMakeCurrent"))(dc, legacy);
    cca = (CREATEATTRIBS)((GETPROC)fn("wglGetProcAddress"))("wglCreateContextAttribsARB");
    printf("  CreateContextAttribsARB %d\n", cca != NULL);
    for (i = 0; cca && i < 2 && !got; i++)
    {
        int a[] = {0x2091, vers[i][0], 0x2092, vers[i][1], 0, 0, 0};
        got = cca(dc, NULL, a);
        printf("  ask %d.%d = %d\n", vers[i][0], vers[i][1], got != NULL);
        if (!got)
        {
            a[4] = 0x2094; /* WGL_CONTEXT_FLAGS_ARB */
            a[5] = 2;      /* FORWARD_COMPATIBLE */
            got = cca(dc, NULL, a);
            printf("  ask %d.%d forward-compatible = %d\n", vers[i][0], vers[i][1], got != NULL);
        }
    }
    if (got)
        ((MAKECURRENT)fn("wglMakeCurrent"))(dc, got);
    printf("  version %s\n", ((GETSTRING)fn("glGetString"))(GL_VERSION));
    ((BEGIN)fn("glBegin"))(GL_TRIANGLE_STRIP);
    ((VERTEX3F)fn("glVertex3f"))(-1.0f, -1.0f, 0.0f);
    ((VERTEX3F)fn("glVertex3f"))(1.0f, -1.0f, 0.0f);
    ((VERTEX3F)fn("glVertex3f"))(-1.0f, 1.0f, 0.0f);
    ((VOIDFN)fn("glEnd"))();
    printf("  immediate survived err=%x\n", ((GETERROR)fn("glGetError"))());
    ((MAKECURRENT)fn("wglMakeCurrent"))(NULL, NULL);
    if (got)
        ((BOOL(WINAPI *)(HGLRC))fn("wglDeleteContext"))(got);
    ((BOOL(WINAPI *)(HGLRC))fn("wglDeleteContext"))(legacy);
    ReleaseDC(w, dc);
    DestroyWindow(w);
    return 0;
}

int main(void)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    gl = LoadLibraryA("opengl32.dll");
    if (!gl)
    {
        printf("LoadLibrary opengl32 failed %lu\n", GetLastError());
        return 2;
    }
    if (round_(1) || round_(2) || attribs_round())
        return 1;
    printf("probe done\n");
    return 0;
}
