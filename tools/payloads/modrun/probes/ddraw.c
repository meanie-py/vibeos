/* modrun probe: ddraw (dll/directx/wine/ddraw -> ddraw-rust) against its pre-flip C.
 *
 * System-memory surfaces only, so nothing depends on a video driver: surface creation and
 * description, pixel formats, locks, colour-fill/stretch/BltFast blits checked by a hash of
 * the pixels, colour keys, palettes, clippers, the old interface chain and refcounts, and
 * Direct3D7 vertex buffers -- whose sizes come from ddraw's own get_flexible_vertex_size,
 * the FVF arithmetic a translation needed hand fixes in. Smoke never loads ddraw. */
#include "dx_probe.h"
#include <ddraw.h>
#include <d3d.h>

/* DirectX 8 FVF bits the DX7 headers lack: a DX7 vertex buffer given them is an edge case
 * ddraw's own size arithmetic has to answer. */
#ifndef D3DFVF_XYZW
#define D3DFVF_XYZW 0x4002
#endif
#ifndef D3DFVF_PSIZE
#define D3DFVF_PSIZE 0x20
#endif
#ifndef D3DFVF_LASTBETA_UBYTE4
#define D3DFVF_LASTBETA_UBYTE4 0x1000
#endif

typedef HRESULT(WINAPI *CREATEEX)(GUID *, void **, REFIID, IUnknown *);
typedef HRESULT(WINAPI *CREATE)(GUID *, IDirectDraw **, IUnknown *);
typedef HRESULT(WINAPI *ENUMA)(LPDDENUMCALLBACKA, void *);

static int modes;
static HRESULT WINAPI mode_cb(DDSURFACEDESC2 *d, void *ctx)
{
    (void)ctx;
    if (modes++ < 4)
        printf("  mode %lux%lu bpp=%lu flags=%lx\n", d->dwWidth, d->dwHeight,
               d->ddpfPixelFormat.dwRGBBitCount, d->dwFlags);
    return DDENUMRET_OK;
}
static BOOL WINAPI drv_cb(GUID *g, char *desc, char *name, void *ctx)
{
    (void)ctx;
    printf("  driver guid=%d desc=%s name=%s\n", g != NULL, desc, name);
    return TRUE;
}
static HRESULT WINAPI dev_cb(char *desc, char *name, D3DDEVICEDESC7 *d, void *ctx)
{
    (void)ctx;
    printf("  d3d device %s (%s) caps=%lx tri=%lx\n", name, desc, d->dwDevCaps,
           d->dpcTriCaps.dwRasterCaps);
    return D3DENUMRET_OK;
}

static IDirectDrawSurface7 *surface(IDirectDraw7 *dd, const char *tag, DWORD w, DWORD h,
                                    DWORD bpp, DWORD r, DWORD g, DWORD b, DWORD flags)
{
    DDSURFACEDESC2 sd;
    IDirectDrawSurface7 *s = NULL;
    HRESULT hr;

    memset(&sd, 0, sizeof(sd));
    sd.dwSize = sizeof(sd);
    sd.dwFlags = DDSD_CAPS | DDSD_WIDTH | DDSD_HEIGHT | (bpp ? DDSD_PIXELFORMAT : 0);
    sd.ddsCaps.dwCaps = DDSCAPS_OFFSCREENPLAIN | DDSCAPS_SYSTEMMEMORY;
    sd.dwWidth = w;
    sd.dwHeight = h;
    if (bpp)
    {
        sd.ddpfPixelFormat.dwSize = sizeof(DDPIXELFORMAT);
        sd.ddpfPixelFormat.dwFlags = DDPF_RGB | flags;
        sd.ddpfPixelFormat.dwRGBBitCount = bpp;
        sd.ddpfPixelFormat.dwRBitMask = r;
        sd.ddpfPixelFormat.dwGBitMask = g;
        sd.ddpfPixelFormat.dwBBitMask = b;
    }
    hr = IDirectDraw7_CreateSurface(dd, &sd, &s, NULL);
    HR(tag, hr);
    if (FAILED(hr))
        return NULL;
    memset(&sd, 0, sizeof(sd));
    sd.dwSize = sizeof(sd);
    IDirectDrawSurface7_GetSurfaceDesc(s, &sd);
    printf("  desc flags=%lx %lux%lu pitch=%ld caps=%lx bpp=%lu pf=%lx masks=%lx/%lx/%lx\n",
           sd.dwFlags, sd.dwWidth, sd.dwHeight, sd.lPitch, sd.ddsCaps.dwCaps,
           sd.ddpfPixelFormat.dwRGBBitCount, sd.ddpfPixelFormat.dwFlags,
           sd.ddpfPixelFormat.dwRBitMask, sd.ddpfPixelFormat.dwGBitMask,
           sd.ddpfPixelFormat.dwBBitMask);
    return s;
}

static void hash(IDirectDrawSurface7 *s, const char *tag)
{
    DDSURFACEDESC2 sd;
    HRESULT hr;
    memset(&sd, 0, sizeof(sd));
    sd.dwSize = sizeof(sd);
    hr = IDirectDrawSurface7_Lock(s, NULL, &sd, DDLOCK_WAIT | DDLOCK_READONLY, NULL);
    if (FAILED(hr))
    {
        HR(tag, hr);
        return;
    }
    printf("%s hash=%08lx\n", tag, dx_hash(sd.lpSurface, sd.lPitch * sd.dwHeight));
    IDirectDrawSurface7_Unlock(s, NULL);
}

static void surfaces(IDirectDraw7 *dd)
{
    IDirectDrawSurface7 *a, *b, *p8;
    IDirectDrawPalette *pal;
    IDirectDrawClipper *clip;
    PALETTEENTRY pe[256], back[4];
    DDSURFACEDESC2 sd;
    DDPIXELFORMAT pf;
    DDCOLORKEY ck;
    DDBLTFX fx;
    RECT r1 = {2, 3, 20, 13}, r2 = {0, 0, 40, 30}, r3 = {5, 5, 9, 9};
    DWORD i, n;

    a = surface(dd, "surf 565", 32, 24, 16, 0xf800, 0x07e0, 0x001f, 0);
    b = surface(dd, "surf 8888", 48, 32, 32, 0xff0000, 0xff00, 0xff, DDPF_ALPHAPIXELS);
    if (b)
    {
        /* DDPF_ALPHAPIXELS without an alpha mask: what each side records is part of the diff. */
        memset(&pf, 0, sizeof(pf));
        pf.dwSize = sizeof(pf);
        HR("  pixel format", IDirectDrawSurface7_GetPixelFormat(b, &pf));
        printf("  pf flags=%lx bpp=%lu a=%lx\n", pf.dwFlags, pf.dwRGBBitCount, pf.dwRGBAlphaBitMask);
    }
    surface(dd, "surf bad bpp", 8, 8, 13, 0x1f00, 0xf0, 0xf, 0);
    surface(dd, "surf default fmt", 16, 16, 0, 0, 0, 0, 0);
    if (!a || !b)
        return;

    memset(&sd, 0, sizeof(sd));
    sd.dwSize = sizeof(sd);
    if (SUCCEEDED(IDirectDrawSurface7_Lock(a, &r1, &sd, DDLOCK_WAIT, NULL)))
    {
        for (i = 0; i < 10; i++)
            memset((BYTE *)sd.lpSurface + i * sd.lPitch, (int)(i * 7 + 1), 36);
        HR("lock rect unlock", IDirectDrawSurface7_Unlock(a, &r1));
    }
    hash(a, "a after lock");
    memset(&fx, 0, sizeof(fx));
    fx.dwSize = sizeof(fx);
    fx.dwFillColor = 0xf81f;
    HR("colorfill", IDirectDrawSurface7_Blt(a, &r3, NULL, NULL, DDBLT_COLORFILL | DDBLT_WAIT, &fx));
    hash(a, "a after fill");
    fx.dwFillColor = 0x80402010;
    HR("colorfill b", IDirectDrawSurface7_Blt(b, NULL, NULL, NULL, DDBLT_COLORFILL | DDBLT_WAIT, &fx));
    hash(b, "b after fill");
    HR("stretch 565->8888", IDirectDrawSurface7_Blt(b, &r2, a, NULL, DDBLT_WAIT, NULL));
    hash(b, "b after stretch");
    HR("bltfast", IDirectDrawSurface7_BltFast(a, 1, 1, a, &r3, DDBLTFAST_WAIT));
    HR("bltfast clip out", IDirectDrawSurface7_BltFast(a, 30, 20, a, &r1, DDBLTFAST_WAIT));
    hash(a, "a after bltfast");

    ck.dwColorSpaceLowValue = 0xf81f;
    ck.dwColorSpaceHighValue = 0xf81f;
    HR("set src ck", IDirectDrawSurface7_SetColorKey(a, DDCKEY_SRCBLT, &ck));
    memset(&ck, 0, sizeof(ck));
    HR("get src ck", IDirectDrawSurface7_GetColorKey(a, DDCKEY_SRCBLT, &ck));
    printf("  ck %lx-%lx\n", ck.dwColorSpaceLowValue, ck.dwColorSpaceHighValue);
    HR("get dst ck unset", IDirectDrawSurface7_GetColorKey(a, DDCKEY_DESTBLT, &ck));
    HR("keyed blt", IDirectDrawSurface7_Blt(b, &r2, a, NULL, DDBLT_KEYSRC | DDBLT_WAIT, NULL));
    hash(b, "b after keyed");

    for (i = 0; i < 256; i++)
    {
        pe[i].peRed = (BYTE)i;
        pe[i].peGreen = (BYTE)(255 - i);
        pe[i].peBlue = (BYTE)(i * 3);
        pe[i].peFlags = 0;
    }
    HR("palette", IDirectDraw7_CreatePalette(dd, DDPCAPS_8BIT | DDPCAPS_ALLOW256, pe, &pal, NULL));
    if (pal)
    {
        HR("  entries", IDirectDrawPalette_GetEntries(pal, 0, 100, 4, back));
        for (i = 0; i < 4; i++)
            printf("  pe %u %u %u %u\n", back[i].peRed, back[i].peGreen, back[i].peBlue, back[i].peFlags);
        HR("  caps", IDirectDrawPalette_GetCaps(pal, &n));
        printf("  palcaps %lx\n", n);
        p8 = surface(dd, "surf 8bpp", 16, 16, 8, 0, 0, 0, DDPF_PALETTEINDEXED8);
        if (p8)
        {
            HR("  set palette", IDirectDrawSurface7_SetPalette(p8, pal));
            HR("  set palette on 565", IDirectDrawSurface7_SetPalette(a, pal));
            printf("  p8 release %lu\n", IDirectDrawSurface7_Release(p8));
        }
        printf("  pal release %lu\n", IDirectDrawPalette_Release(pal));
    }
    HR("clipper", IDirectDraw7_CreateClipper(dd, 0, &clip, NULL));
    if (clip)
    {
        struct { RGNDATAHEADER h; RECT r[2]; } rgn;
        DWORD size = 0;
        memset(&rgn, 0, sizeof(rgn));
        rgn.h.dwSize = sizeof(rgn.h);
        rgn.h.iType = RDH_RECTANGLES;
        rgn.h.nCount = 2;
        rgn.h.nRgnSize = sizeof(rgn.r);
        SetRect(&rgn.r[0], 0, 0, 10, 10);
        SetRect(&rgn.r[1], 20, 0, 30, 5);
        SetRect(&rgn.h.rcBound, 0, 0, 30, 10);
        HR("  set cliplist", IDirectDrawClipper_SetClipList(clip, (RGNDATA *)&rgn, 0));
        HR("  get cliplist size", IDirectDrawClipper_GetClipList(clip, NULL, NULL, &size));
        printf("  clip size %lu\n", size);
        HR("  attach", IDirectDrawSurface7_SetClipper(b, clip));
        fx.dwFillColor = 0x00ffffff;
        HR("  clipped fill", IDirectDrawSurface7_Blt(b, NULL, NULL, NULL, DDBLT_COLORFILL | DDBLT_WAIT, &fx));
        hash(b, "b after clipped fill");
        HR("  detach", IDirectDrawSurface7_SetClipper(b, NULL));
        printf("  clip release %lu\n", IDirectDrawClipper_Release(clip));
    }
    printf("a release %lu b release %lu\n", IDirectDrawSurface7_Release(a),
           IDirectDrawSurface7_Release(b));
}

static void d3d7(IDirectDraw7 *dd)
{
    static const DWORD fvfs[] = {
        D3DFVF_XYZ, D3DFVF_XYZRHW | D3DFVF_DIFFUSE | D3DFVF_SPECULAR | D3DFVF_TEX1,
        D3DFVF_XYZB3 | D3DFVF_NORMAL, D3DFVF_XYZB5 | D3DFVF_LASTBETA_UBYTE4 | D3DFVF_TEX2,
        D3DFVF_XYZ | D3DFVF_TEX3 | D3DFVF_TEXCOORDSIZE3(0) | D3DFVF_TEXCOORDSIZE1(1) | D3DFVF_TEXCOORDSIZE4(2),
        D3DFVF_XYZ | D3DFVF_TEX8, D3DFVF_XYZB1 | D3DFVF_TEX4 | D3DFVF_TEXCOORDSIZE2(3),
        D3DFVF_XYZW | D3DFVF_NORMAL | D3DFVF_PSIZE, 0,
    };
    IDirect3D7 *d3d;
    IDirect3DVertexBuffer7 *vb;
    D3DVERTEXBUFFERDESC vd;
    void *data;
    DWORD size, i;
    HRESULT hr;

    hr = IDirectDraw7_QueryInterface(dd, &IID_IDirect3D7, (void **)&d3d);
    HR("qi d3d7", hr);
    if (FAILED(hr))
        return;
    HR("enum devices", IDirect3D7_EnumDevices(d3d, dev_cb, NULL));
    for (i = 0; i < sizeof(fvfs) / sizeof(fvfs[0]); i++)
    {
        memset(&vd, 0, sizeof(vd));
        vd.dwSize = sizeof(vd);
        vd.dwCaps = D3DVBCAPS_SYSTEMMEMORY;
        vd.dwFVF = fvfs[i];
        vd.dwNumVertices = 7;
        hr = IDirect3D7_CreateVertexBuffer(d3d, &vd, &vb, 0);
        printf("vb fvf=%lx hr=%08lx", fvfs[i], (unsigned long)hr);
        if (SUCCEEDED(hr))
        {
            size = 0;
            hr = IDirect3DVertexBuffer7_Lock(vb, DDLOCK_WAIT, &data, &size);
            printf(" lock=%08lx bytes=%lu", (unsigned long)hr, size);
            if (SUCCEEDED(hr))
                IDirect3DVertexBuffer7_Unlock(vb);
            memset(&vd, 0, sizeof(vd));
            vd.dwSize = sizeof(vd);
            IDirect3DVertexBuffer7_GetVertexBufferDesc(vb, &vd);
            printf(" desc caps=%lx fvf=%lx n=%lu release=%lu", vd.dwCaps, vd.dwFVF,
                   vd.dwNumVertices, IDirect3DVertexBuffer7_Release(vb));
        }
        printf("\n");
    }
    printf("d3d7 release %lu\n", IDirect3D7_Release(d3d));
}

int main(void)
{
    CREATEEX createex;
    CREATE create;
    IDirectDraw7 *dd = NULL;
    IDirectDraw *dd1 = NULL;
    IDirectDraw2 *dd2;
    IDirectDraw4 *dd4;
    DDCAPS hal, hel;
    DDSURFACEDESC2 sd;
    DWORD total, freemem, codes[16], n;
    DDSCAPS2 caps;
    HWND hwnd;
    HRESULT hr;

    dx_load(L"ddraw");
    createex = (CREATEEX)dx_proc("DirectDrawCreateEx");
    create = (CREATE)dx_proc("DirectDrawCreate");
    HR("enumerate", ((ENUMA)dx_proc("DirectDrawEnumerateA"))(drv_cb, NULL));
    HR("createex wrong iid", createex(NULL, (void **)&dd, &IID_IDirectDraw, NULL));
    hr = createex(NULL, (void **)&dd, &IID_IDirectDraw7, NULL);
    HR("createex", hr);
    if (FAILED(hr))
        return DONE();
    hwnd = dx_window();
    HR("coop normal", IDirectDraw7_SetCooperativeLevel(dd, hwnd, DDSCL_NORMAL));
    HR("coop bad flags", IDirectDraw7_SetCooperativeLevel(dd, hwnd, DDSCL_FULLSCREEN));

    memset(&hal, 0, sizeof(hal));
    memset(&hel, 0, sizeof(hel));
    hal.dwSize = hel.dwSize = sizeof(DDCAPS);
    HR("caps", IDirectDraw7_GetCaps(dd, &hal, &hel));
    dx_words("hal", &hal, sizeof(hal));
    dx_words("hel", &hel, sizeof(hel));
    memset(&sd, 0, sizeof(sd));
    sd.dwSize = sizeof(sd);
    HR("display mode", IDirectDraw7_GetDisplayMode(dd, &sd));
    printf("  %lux%lu bpp=%lu flags=%lx\n", sd.dwWidth, sd.dwHeight,
           sd.ddpfPixelFormat.dwRGBBitCount, sd.dwFlags);
    HR("enum modes", IDirectDraw7_EnumDisplayModes(dd, 0, NULL, NULL, mode_cb));
    printf("modes %d\n", modes);
    n = 16;
    HR("fourcc", IDirectDraw7_GetFourCCCodes(dd, &n, codes));
    printf("  fourcc count %lu\n", n);
    memset(&caps, 0, sizeof(caps));
    caps.dwCaps = DDSCAPS_TEXTURE;
    hr = IDirectDraw7_GetAvailableVidMem(dd, &caps, &total, &freemem);
    printf("vidmem hr=%08lx nonzero=%d\n", (unsigned long)hr, SUCCEEDED(hr) && total != 0);
    HR("verify", IDirectDraw7_TestCooperativeLevel(dd));

    surfaces(dd);
    d3d7(dd);
    printf("dd7 release %lu\n", IDirectDraw7_Release(dd));

    hr = create(NULL, &dd1, NULL);
    HR("create v1", hr);
    if (SUCCEEDED(hr))
    {
        HR("  qi 2", IDirectDraw_QueryInterface(dd1, &IID_IDirectDraw2, (void **)&dd2));
        HR("  qi 4", IDirectDraw_QueryInterface(dd1, &IID_IDirectDraw4, (void **)&dd4));
        printf("  dd4 release %lu\n", IDirectDraw4_Release(dd4));
        printf("  dd2 release %lu\n", IDirectDraw2_Release(dd2));
        printf("  dd1 release %lu\n", IDirectDraw_Release(dd1));
    }
    DestroyWindow(hwnd);
    return DONE();
}
