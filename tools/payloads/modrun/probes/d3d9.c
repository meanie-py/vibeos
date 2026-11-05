/* modrun probe: d3d9 (dll/directx/wine/d3d9 -> d3d9-rust) against its pre-flip C.
 *
 * Both variants run on the same wined3d (d3dwine.dll, still C), so every difference is the
 * front end's: the D3DFORMAT <-> wined3d format tables, the caps conversion, the usage and
 * pool checks, state get/set (the texture-stage lookup a translation once got wrong -- it
 * re-declared WINED3D_TSS_INVALID as 18), vertex declarations and resource descriptions.
 * Smoke never loads d3d9 at all. */
#include "dx_probe.h"
#include <stdlib.h>
#include <d3d9.h>

typedef IDirect3D9 *(WINAPI *CREATE9)(UINT);

static const D3DFORMAT formats[] = {
    D3DFMT_R8G8B8, D3DFMT_A8R8G8B8, D3DFMT_X8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5,
    D3DFMT_A1R5G5B5, D3DFMT_A4R4G4B4, D3DFMT_R3G3B2, D3DFMT_A8, D3DFMT_A8R3G3B2,
    D3DFMT_X4R4G4B4, D3DFMT_A2B10G10R10, D3DFMT_A8B8G8R8, D3DFMT_X8B8G8R8, D3DFMT_G16R16,
    D3DFMT_A2R10G10B10, D3DFMT_A16B16G16R16, D3DFMT_A8P8, D3DFMT_P8, D3DFMT_L8, D3DFMT_A8L8,
    D3DFMT_A4L4, D3DFMT_V8U8, D3DFMT_L6V5U5, D3DFMT_X8L8V8U8, D3DFMT_Q8W8V8U8, D3DFMT_V16U16,
    D3DFMT_A2W10V10U10, D3DFMT_UYVY, D3DFMT_R8G8_B8G8, D3DFMT_YUY2, D3DFMT_G8R8_G8B8,
    D3DFMT_DXT1, D3DFMT_DXT2, D3DFMT_DXT3, D3DFMT_DXT4, D3DFMT_DXT5, D3DFMT_D16_LOCKABLE,
    D3DFMT_D32, D3DFMT_D15S1, D3DFMT_D24S8, D3DFMT_D24X8, D3DFMT_D24X4S4, D3DFMT_D16,
    D3DFMT_D32F_LOCKABLE, D3DFMT_D24FS8, D3DFMT_L16, D3DFMT_INDEX16, D3DFMT_INDEX32,
    D3DFMT_Q16W16V16U16, D3DFMT_R16F, D3DFMT_G16R16F, D3DFMT_A16B16G16R16F, D3DFMT_R32F,
    D3DFMT_G32R32F, D3DFMT_A32B32G32R32F, D3DFMT_CxV8U8,
    (D3DFORMAT)MAKEFOURCC('A', 'T', 'I', '2'), (D3DFORMAT)MAKEFOURCC('N', 'V', 'D', 'B'),
    (D3DFORMAT)MAKEFOURCC('I', 'N', 'T', 'Z'), (D3DFORMAT)MAKEFOURCC('R', 'E', 'S', 'Z'),
    (D3DFORMAT)MAKEFOURCC('N', 'U', 'L', 'L'), (D3DFORMAT)0x12345678,
};
#define NFMT (sizeof(formats) / sizeof(formats[0]))

static const DWORD usages[] = {
    0, D3DUSAGE_RENDERTARGET, D3DUSAGE_DEPTHSTENCIL, D3DUSAGE_DYNAMIC, D3DUSAGE_AUTOGENMIPMAP,
    D3DUSAGE_QUERY_FILTER, D3DUSAGE_QUERY_SRGBREAD, D3DUSAGE_QUERY_SRGBWRITE,
    D3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING, D3DUSAGE_QUERY_VERTEXTEXTURE,
    D3DUSAGE_QUERY_WRAPANDMIP, D3DUSAGE_QUERY_LEGACYBUMPMAP, D3DUSAGE_RENDERTARGET | D3DUSAGE_DYNAMIC,
};
static const D3DRESOURCETYPE rtypes[] = {
    D3DRTYPE_SURFACE, D3DRTYPE_VOLUME, D3DRTYPE_TEXTURE, D3DRTYPE_VOLUMETEXTURE,
    D3DRTYPE_CUBETEXTURE, D3DRTYPE_VERTEXBUFFER, D3DRTYPE_INDEXBUFFER,
};

static void adapter(IDirect3D9 *d3d)
{
    D3DADAPTER_IDENTIFIER9 id;
    D3DDISPLAYMODE mode;
    D3DFORMAT adapters[] = {D3DFMT_X8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5, D3DFMT_A8R8G8B8};
    UINT i, n;

    printf("adapters %u\n", IDirect3D9_GetAdapterCount(d3d));
    memset(&id, 0, sizeof(id));
    HR("identifier", IDirect3D9_GetAdapterIdentifier(d3d, 0, 0, &id));
    printf("  driver=%s desc=%s dev=%s\n", id.Driver, id.Description, id.DeviceName);
    printf("  vendor=%lx device=%lx subsys=%lx rev=%lx whql=%lx\n", id.VendorId, id.DeviceId,
           id.SubSysId, id.Revision, id.WHQLLevel);
    HR("identifier bad adapter", IDirect3D9_GetAdapterIdentifier(d3d, 7, 0, &id));
    HR("display mode", IDirect3D9_GetAdapterDisplayMode(d3d, 0, &mode));
    printf("  mode %ux%u fmt=%d\n", mode.Width, mode.Height, mode.Format);
    for (i = 0; i < 4; i++)
    {
        n = IDirect3D9_GetAdapterModeCount(d3d, 0, adapters[i]);
        printf("modes fmt=%d count=%u\n", adapters[i], n);
        if (n)
        {
            HR("  enum 0", IDirect3D9_EnumAdapterModes(d3d, 0, adapters[i], 0, &mode));
            printf("  %ux%u fmt=%d\n", mode.Width, mode.Height, mode.Format);
        }
        HR("  enum past end", IDirect3D9_EnumAdapterModes(d3d, 0, adapters[i], n, &mode));
    }
    for (i = 0; i < 4; i++)
    {
        printf("devtype adapter=%d ", adapters[i]);
        printf("hal=%08lx ", IDirect3D9_CheckDeviceType(d3d, 0, D3DDEVTYPE_HAL, adapters[i],
                                                           adapters[i], TRUE));
        printf("ref=%08lx ", IDirect3D9_CheckDeviceType(d3d, 0, D3DDEVTYPE_REF, adapters[i],
                                                           adapters[i], TRUE));
        printf("full=%08lx\n", IDirect3D9_CheckDeviceType(d3d, 0, D3DDEVTYPE_HAL, adapters[i],
                                                              adapters[i], FALSE));
    }
}

static void format_matrix(IDirect3D9 *d3d)
{
    UINT f, u, r;
    for (f = 0; f < NFMT; f++)
    {
        printf("fmt %08x", (unsigned)formats[f]);
        for (r = 0; r < sizeof(rtypes) / sizeof(rtypes[0]); r++)
            for (u = 0; u < sizeof(usages) / sizeof(usages[0]); u++)
            {
                HRESULT hr = IDirect3D9_CheckDeviceFormat(d3d, 0, D3DDEVTYPE_HAL, D3DFMT_X8R8G8B8,
                                                          usages[u], rtypes[r], formats[f]);
                /* One letter per cell keeps 6 KB of matrix readable in the serial log. */
                printf("%c", hr == D3D_OK ? 'y' : hr == D3DOK_NOAUTOGEN ? 'a'
                           : hr == D3DERR_NOTAVAILABLE ? 'n' : hr == D3DERR_INVALIDCALL ? 'i' : '?');
            }
        printf(" ms=%08lx", IDirect3D9_CheckDeviceMultiSampleType(
                               d3d, 0, D3DDEVTYPE_HAL, formats[f], TRUE, D3DMULTISAMPLE_4_SAMPLES, NULL));
        printf(" ds=%08lx", IDirect3D9_CheckDepthStencilMatch(d3d, 0, D3DDEVTYPE_HAL, D3DFMT_X8R8G8B8,
                                                               D3DFMT_X8R8G8B8, formats[f]));
        printf(" conv=%08lx\n", IDirect3D9_CheckDeviceFormatConversion(d3d, 0, D3DDEVTYPE_HAL,
                                                                       formats[f], D3DFMT_X8R8G8B8));
    }
}

static void device(IDirect3D9 *d3d, HWND hwnd)
{
    D3DPRESENT_PARAMETERS pp;
    IDirect3DDevice9 *dev = NULL;
    IDirect3DVertexBuffer9 *vb;
    IDirect3DIndexBuffer9 *ib;
    IDirect3DTexture9 *tex;
    IDirect3DCubeTexture9 *cube;
    IDirect3DVolumeTexture9 *vol;
    IDirect3DSurface9 *surf;
    IDirect3DVertexDeclaration9 *decl;
    IDirect3DStateBlock9 *sb;
    D3DVERTEXBUFFER_DESC vbd;
    D3DSURFACE_DESC sd;
    D3DVOLUME_DESC vd;
    D3DLOCKED_RECT lr;
    D3DVIEWPORT9 vp;
    D3DMATRIX m;
    D3DVERTEXELEMENT9 elems[] = {
        {0, 0, D3DDECLTYPE_FLOAT3, D3DDECLMETHOD_DEFAULT, D3DDECLUSAGE_POSITION, 0},
        {0, 12, D3DDECLTYPE_D3DCOLOR, D3DDECLMETHOD_DEFAULT, D3DDECLUSAGE_COLOR, 0},
        {1, 0, D3DDECLTYPE_FLOAT2, D3DDECLMETHOD_DEFAULT, D3DDECLUSAGE_TEXCOORD, 1},
        D3DDECL_END()};
    D3DVERTEXELEMENT9 back[8];
    D3DCAPS9 caps;
    void *data;
    DWORD v, i, s;
    UINT n;
    HRESULT hr;

    memset(&caps, 0, sizeof(caps));
    HR("caps hal", IDirect3D9_GetDeviceCaps(d3d, 0, D3DDEVTYPE_HAL, &caps));
    dx_words("caps", &caps, sizeof(caps));
    HR("caps ref", IDirect3D9_GetDeviceCaps(d3d, 0, D3DDEVTYPE_REF, &caps));
    HR("caps sw", IDirect3D9_GetDeviceCaps(d3d, 0, D3DDEVTYPE_SW, &caps));

    memset(&pp, 0, sizeof(pp));
    pp.Windowed = TRUE;
    pp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    pp.BackBufferFormat = D3DFMT_X8R8G8B8;
    pp.BackBufferWidth = 64;
    pp.BackBufferHeight = 64;
    pp.EnableAutoDepthStencil = TRUE;
    pp.AutoDepthStencilFormat = D3DFMT_D24S8;
    pp.hDeviceWindow = hwnd;
    hr = IDirect3D9_CreateDevice(d3d, 0, D3DDEVTYPE_HAL, hwnd,
                                 D3DCREATE_SOFTWARE_VERTEXPROCESSING, &pp, &dev);
    HR("create device", hr);
    if (FAILED(hr))
    {
        /* no3d may offer no depth format: retry without the auto depth-stencil. */
        pp.EnableAutoDepthStencil = FALSE;
        pp.AutoDepthStencilFormat = D3DFMT_UNKNOWN;
        hr = IDirect3D9_CreateDevice(d3d, 0, D3DDEVTYPE_HAL, hwnd, D3DCREATE_SOFTWARE_VERTEXPROCESSING, &pp, &dev);
        HR("create device no depth", hr);
    }
    if (FAILED(hr))
        return;

    printf("available mem nonzero %d\n", IDirect3DDevice9_GetAvailableTextureMem(dev) != 0);
    HR("test coop", IDirect3DDevice9_TestCooperativeLevel(dev));
    printf("swapchains %u\n", IDirect3DDevice9_GetNumberOfSwapChains(dev));

    /* Render and texture-stage state: set, read back, and an out-of-range stage/state. */
    for (s = 0; s < 210; s++)
    {
        v = 0xdeadbeef;
        hr = IDirect3DDevice9_GetRenderState(dev, (D3DRENDERSTATETYPE)s, &v);
        if (SUCCEEDED(hr))
            printf("rs %lu=%lx\n", s, v);
    }
    HR("rs set zenable", IDirect3DDevice9_SetRenderState(dev, D3DRS_ZENABLE, D3DZB_FALSE));
    HR("rs set cull", IDirect3DDevice9_SetRenderState(dev, D3DRS_CULLMODE, D3DCULL_CW));
    IDirect3DDevice9_GetRenderState(dev, D3DRS_CULLMODE, &v);
    printf("rs cull back %lx\n", v);
    for (i = 0; i < 9; i++)
        for (s = 0; s < 34; s++)
        {
            v = 0xdeadbeef;
            hr = IDirect3DDevice9_GetTextureStageState(dev, i, (D3DTEXTURESTAGESTATETYPE)s, &v);
            printf("tss %lu.%lu hr=%08lx v=%lx\n", i, s, (unsigned long)hr, v);
        }
    for (s = 0; s < 34; s++)
    {
        hr = IDirect3DDevice9_SetTextureStageState(dev, 1, (D3DTEXTURESTAGESTATETYPE)s, 5);
        v = 0xdeadbeef;
        IDirect3DDevice9_GetTextureStageState(dev, 1, (D3DTEXTURESTAGESTATETYPE)s, &v);
        printf("tss set 1.%lu hr=%08lx back=%lx\n", s, (unsigned long)hr, v);
    }
    for (s = 0; s < 14; s++)
    {
        v = 0xdeadbeef;
        hr = IDirect3DDevice9_GetSamplerState(dev, 0, (D3DSAMPLERSTATETYPE)s, &v);
        printf("samp 0.%lu hr=%08lx v=%lx\n", s, (unsigned long)hr, v);
    }
    memset(&m, 0, sizeof(m));
    m._11 = m._22 = m._33 = m._44 = 2.0f;
    m._41 = 0.5f;
    HR("set world", IDirect3DDevice9_SetTransform(dev, D3DTS_WORLDMATRIX(3), &m));
    memset(&m, 0, sizeof(m));
    HR("get world", IDirect3DDevice9_GetTransform(dev, D3DTS_WORLDMATRIX(3), &m));
    dx_words("world", &m, sizeof(m));
    HR("multiply", IDirect3DDevice9_MultiplyTransform(dev, D3DTS_WORLDMATRIX(3), &m));
    IDirect3DDevice9_GetTransform(dev, D3DTS_WORLDMATRIX(3), &m);
    dx_words("world^2", &m, sizeof(m));
    vp.X = 1; vp.Y = 2; vp.Width = 30; vp.Height = 40; vp.MinZ = 0.25f; vp.MaxZ = 0.75f;
    HR("set viewport", IDirect3DDevice9_SetViewport(dev, &vp));
    memset(&vp, 0, sizeof(vp));
    HR("get viewport", IDirect3DDevice9_GetViewport(dev, &vp));
    dx_words("viewport", &vp, sizeof(vp));

    /* Resources: descriptions, locks, and the pool/usage validation. */
    hr = IDirect3DDevice9_CreateVertexBuffer(dev, 96, D3DUSAGE_WRITEONLY, D3DFVF_XYZ | D3DFVF_DIFFUSE,
                                             D3DPOOL_MANAGED, &vb, NULL);
    HR("vb", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DVertexBuffer9_GetDesc(vb, &vbd);
        printf("  vb fmt=%d type=%d usage=%lx pool=%d size=%u fvf=%lx\n", vbd.Format, vbd.Type,
               vbd.Usage, vbd.Pool, vbd.Size, vbd.FVF);
        hr = IDirect3DVertexBuffer9_Lock(vb, 16, 32, &data, 0);
        HR("  vb lock", hr);
        if (SUCCEEDED(hr))
        {
            memset(data, 0x5a, 32);
            HR("  vb unlock", IDirect3DVertexBuffer9_Unlock(vb));
        }
        HR("  stream", IDirect3DDevice9_SetStreamSource(dev, 0, vb, 0, 16));
        printf("  vb release %lu\n", IDirect3DVertexBuffer9_Release(vb));
    }
    HR("vb default+dynamic", IDirect3DDevice9_CreateVertexBuffer(dev, 64, D3DUSAGE_DYNAMIC, 0,
                                                                  D3DPOOL_MANAGED, &vb, NULL));
    hr = IDirect3DDevice9_CreateIndexBuffer(dev, 64, 0, D3DFMT_INDEX16, D3DPOOL_SYSTEMMEM, &ib, NULL);
    HR("ib", hr);
    if (SUCCEEDED(hr))
        printf("  ib release %lu\n", IDirect3DIndexBuffer9_Release(ib));
    HR("ib bad fmt", IDirect3DDevice9_CreateIndexBuffer(dev, 64, 0, D3DFMT_X8R8G8B8,
                                                        D3DPOOL_SYSTEMMEM, &ib, NULL));
    hr = IDirect3DDevice9_CreateTexture(dev, 32, 16, 0, 0, D3DFMT_A8R8G8B8, D3DPOOL_MANAGED, &tex, NULL);
    HR("tex", hr);
    if (SUCCEEDED(hr))
    {
        n = IDirect3DTexture9_GetLevelCount(tex);
        printf("  levels %u lod %lu\n", n, IDirect3DTexture9_GetLOD(tex));
        for (i = 0; i < n; i++)
        {
            IDirect3DTexture9_GetLevelDesc(tex, i, &sd);
            printf("  level %lu %ux%u fmt=%d usage=%lx pool=%d ms=%d\n", i, sd.Width, sd.Height,
                   sd.Format, sd.Usage, sd.Pool, sd.MultiSampleType);
        }
        HR("  desc past end", IDirect3DTexture9_GetLevelDesc(tex, n, &sd));
        hr = IDirect3DTexture9_LockRect(tex, 1, &lr, NULL, 0);
        HR("  lock 1", hr);
        if (SUCCEEDED(hr))
        {
            printf("  pitch %d\n", lr.Pitch);
            memset(lr.pBits, 0x33, lr.Pitch * 8);
            IDirect3DTexture9_UnlockRect(tex, 1);
        }
        HR("  surface level 2", IDirect3DTexture9_GetSurfaceLevel(tex, 2, &surf));
        if (SUCCEEDED(IDirect3DTexture9_GetSurfaceLevel(tex, 2, &surf)))
        {
            IDirect3DSurface9_GetDesc(surf, &sd);
            printf("  surf %ux%u type=%d\n", sd.Width, sd.Height, sd.Type);
            IDirect3DSurface9_Release(surf);
            IDirect3DSurface9_Release(surf);
        }
        HR("  set texture", IDirect3DDevice9_SetTexture(dev, 0, (IDirect3DBaseTexture9 *)tex));
        HR("  unset texture", IDirect3DDevice9_SetTexture(dev, 0, NULL));
        printf("  tex release %lu\n", IDirect3DTexture9_Release(tex));
    }
    HR("tex dxt odd", IDirect3DDevice9_CreateTexture(dev, 3, 3, 1, 0, D3DFMT_DXT1, D3DPOOL_MANAGED, &tex, NULL));
    HR("tex rt managed", IDirect3DDevice9_CreateTexture(dev, 16, 16, 1, D3DUSAGE_RENDERTARGET,
                                                        D3DFMT_X8R8G8B8, D3DPOOL_MANAGED, &tex, NULL));
    hr = IDirect3DDevice9_CreateCubeTexture(dev, 16, 0, 0, D3DFMT_X8R8G8B8, D3DPOOL_MANAGED, &cube, NULL);
    HR("cube", hr);
    if (SUCCEEDED(hr))
    {
        printf("  cube levels %lu\n", (unsigned long)IDirect3DCubeTexture9_GetLevelCount(cube));
        HR("  cube face 5", IDirect3DCubeTexture9_GetCubeMapSurface(cube, D3DCUBEMAP_FACE_NEGATIVE_Z, 1, &surf));
        if (SUCCEEDED(IDirect3DCubeTexture9_GetCubeMapSurface(cube, D3DCUBEMAP_FACE_NEGATIVE_Z, 1, &surf)))
        {
            IDirect3DSurface9_Release(surf);
            IDirect3DSurface9_Release(surf);
        }
        printf("  cube release %lu\n", IDirect3DCubeTexture9_Release(cube));
    }
    hr = IDirect3DDevice9_CreateVolumeTexture(dev, 8, 4, 2, 0, 0, D3DFMT_A8R8G8B8, D3DPOOL_MANAGED, &vol, NULL);
    HR("volume", hr);
    if (SUCCEEDED(hr))
    {
        n = IDirect3DVolumeTexture9_GetLevelCount(vol);
        for (i = 0; i < n; i++)
        {
            IDirect3DVolumeTexture9_GetLevelDesc(vol, i, &vd);
            printf("  vlevel %lu %ux%ux%u fmt=%d\n", i, vd.Width, vd.Height, vd.Depth, vd.Format);
        }
        printf("  volume release %lu\n", IDirect3DVolumeTexture9_Release(vol));
    }
    hr = IDirect3DDevice9_CreateOffscreenPlainSurface(dev, 16, 16, D3DFMT_A8R8G8B8, D3DPOOL_SYSTEMMEM, &surf, NULL);
    HR("offscreen", hr);
    if (SUCCEEDED(hr))
    {
        RECT r = {2, 2, 10, 6};
        HR("  colorfill sysmem", IDirect3DDevice9_ColorFill(dev, surf, &r, 0xff102030));
        if (SUCCEEDED(IDirect3DSurface9_LockRect(surf, &lr, NULL, D3DLOCK_READONLY)))
        {
            printf("  hash %08lx pitch %d\n", dx_hash(lr.pBits, lr.Pitch * 16), lr.Pitch);
            IDirect3DSurface9_UnlockRect(surf);
        }
        printf("  offscreen release %lu\n", IDirect3DSurface9_Release(surf));
    }

    hr = IDirect3DDevice9_CreateVertexDeclaration(dev, elems, &decl);
    HR("decl", hr);
    if (SUCCEEDED(hr))
    {
        n = 0;
        HR("  decl count", IDirect3DVertexDeclaration9_GetDeclaration(decl, NULL, &n));
        printf("  decl elements %u\n", n);
        memset(back, 0xcc, sizeof(back));
        IDirect3DVertexDeclaration9_GetDeclaration(decl, back, &n);
        for (i = 0; i < n; i++)
            printf("  el %u %u %u %u %u %u\n", back[i].Stream, back[i].Offset, back[i].Type,
                   back[i].Method, back[i].Usage, back[i].UsageIndex);
        HR("  set decl", IDirect3DDevice9_SetVertexDeclaration(dev, decl));
        HR("  fvf after decl", IDirect3DDevice9_GetFVF(dev, &v));
        printf("  fvf %lx\n", v);
        printf("  decl release %lu\n", IDirect3DVertexDeclaration9_Release(decl));
    }
    HR("set fvf", IDirect3DDevice9_SetFVF(dev, D3DFVF_XYZRHW | D3DFVF_TEX2 | D3DFVF_TEXCOORDSIZE3(1)));
    IDirect3DDevice9_GetFVF(dev, &v);
    printf("fvf back %lx\n", v);

    hr = IDirect3DDevice9_CreateStateBlock(dev, D3DSBT_ALL, &sb);
    HR("stateblock", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DDevice9_SetRenderState(dev, D3DRS_CULLMODE, D3DCULL_NONE);
        HR("  apply", IDirect3DStateBlock9_Apply(sb));
        IDirect3DDevice9_GetRenderState(dev, D3DRS_CULLMODE, &v);
        printf("  cull after apply %lx\n", v);
        printf("  sb release %lu\n", IDirect3DStateBlock9_Release(sb));
    }
    HR("begin sb", IDirect3DDevice9_BeginStateBlock(dev));
    IDirect3DDevice9_SetRenderState(dev, D3DRS_FOGENABLE, TRUE);
    hr = IDirect3DDevice9_EndStateBlock(dev, &sb);
    HR("end sb", hr);
    if (SUCCEEDED(hr))
        IDirect3DStateBlock9_Release(sb);

    HR("clear", IDirect3DDevice9_Clear(dev, 0, NULL, D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER, 0xff00ff00, 1.0f, 0));
    HR("begin scene", IDirect3DDevice9_BeginScene(dev));
    HR("begin scene twice", IDirect3DDevice9_BeginScene(dev));
    HR("end scene", IDirect3DDevice9_EndScene(dev));
    HR("present", IDirect3DDevice9_Present(dev, NULL, NULL, NULL, NULL));
    HR("backbuffer", IDirect3DDevice9_GetBackBuffer(dev, 0, 0, D3DBACKBUFFER_TYPE_MONO, &surf));
    if (SUCCEEDED(IDirect3DDevice9_GetBackBuffer(dev, 0, 0, D3DBACKBUFFER_TYPE_MONO, &surf)))
    {
        IDirect3DSurface9_GetDesc(surf, &sd);
        printf("  bb %ux%u fmt=%d usage=%lx\n", sd.Width, sd.Height, sd.Format, sd.Usage);
        IDirect3DSurface9_Release(surf);
        IDirect3DSurface9_Release(surf);
    }
    printf("device release %lu\n", IDirect3DDevice9_Release(dev));
}

int main(void)
{
    CREATE9 create;
    IDirect3D9 *d3d;
    HWND hwnd;

    /* The guest's GL is Mesa's software renderer, which wined3d's GL backend refuses (no
     * ARB_fragment_shader): Direct3DCreate then returns NULL under both variants and nothing
     * past it runs. wined3d's no3d renderer needs no GPU, reads this from msvcrt's
     * environment (shared with this exe), and is the same C under both. */
    _putenv("WINE_D3D_CONFIG=renderer=no3d");
    dx_load(L"d3d9");
    create = (CREATE9)dx_proc("Direct3DCreate9");
    printf("create bad sdk %d\n", create(1) != NULL);
    d3d = create(D3D_SDK_VERSION);
    printf("create %d\n", d3d != NULL);
    if (!d3d)
        return DONE();
    hwnd = dx_window();
    adapter(d3d);
    format_matrix(d3d);
    device(d3d, hwnd);
    printf("d3d release %lu\n", IDirect3D9_Release(d3d));
    DestroyWindow(hwnd);
    return DONE();
}
