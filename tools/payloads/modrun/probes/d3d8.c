/* modrun probe: d3d8 (dll/directx/wine/d3d8 -> d3d8-rust) against its pre-flip C.
 *
 * Same wined3d under both variants, so differences are the front end's: format tables, the
 * caps conversion, d3d8's texture-stage lookup (29 rows, where d3d8 still carries the
 * sampler states as stage states), and its own vertex-shader declaration parser
 * (D3DVSD_* tokens -> wined3d elements). Smoke never loads d3d8. */
#include "dx_probe.h"
#include <stdlib.h>
#include <d3d8.h>

typedef IDirect3D8 *(WINAPI *CREATE8)(UINT);

static const D3DFORMAT formats[] = {
    D3DFMT_R8G8B8, D3DFMT_A8R8G8B8, D3DFMT_X8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5,
    D3DFMT_A1R5G5B5, D3DFMT_A4R4G4B4, D3DFMT_R3G3B2, D3DFMT_A8, D3DFMT_A8R3G3B2,
    D3DFMT_X4R4G4B4, D3DFMT_A2B10G10R10, D3DFMT_G16R16, D3DFMT_A8P8, D3DFMT_P8, D3DFMT_L8,
    D3DFMT_A8L8, D3DFMT_A4L4, D3DFMT_V8U8, D3DFMT_L6V5U5, D3DFMT_X8L8V8U8, D3DFMT_Q8W8V8U8,
    D3DFMT_V16U16, D3DFMT_W11V11U10, D3DFMT_A2W10V10U10, D3DFMT_UYVY, D3DFMT_YUY2,
    D3DFMT_DXT1, D3DFMT_DXT2, D3DFMT_DXT3, D3DFMT_DXT4, D3DFMT_DXT5, D3DFMT_D16_LOCKABLE,
    D3DFMT_D32, D3DFMT_D15S1, D3DFMT_D24S8, D3DFMT_D16, D3DFMT_D24X8, D3DFMT_D24X4S4,
    D3DFMT_INDEX16, D3DFMT_INDEX32, (D3DFORMAT)MAKEFOURCC('R', 'E', 'S', 'Z'),
    (D3DFORMAT)MAKEFOURCC('I', 'N', 'T', 'Z'), (D3DFORMAT)0x12345678,
};
#define NFMT (sizeof(formats) / sizeof(formats[0]))
static const DWORD usages[] = {0, D3DUSAGE_RENDERTARGET, D3DUSAGE_DEPTHSTENCIL, D3DUSAGE_DYNAMIC};
static const D3DRESOURCETYPE rtypes[] = {
    D3DRTYPE_SURFACE, D3DRTYPE_VOLUME, D3DRTYPE_TEXTURE, D3DRTYPE_VOLUMETEXTURE,
    D3DRTYPE_CUBETEXTURE, D3DRTYPE_VERTEXBUFFER, D3DRTYPE_INDEXBUFFER,
};

static void adapter(IDirect3D8 *d3d)
{
    D3DADAPTER_IDENTIFIER8 id;
    D3DDISPLAYMODE mode;
    D3DFORMAT disp[] = {D3DFMT_X8R8G8B8, D3DFMT_R5G6B5, D3DFMT_X1R5G5B5};
    UINT i, n, f, u, r;

    printf("adapters %u\n", IDirect3D8_GetAdapterCount(d3d));
    memset(&id, 0, sizeof(id));
    HR("identifier", IDirect3D8_GetAdapterIdentifier(d3d, 0, 0, &id));
    printf("  driver=%s desc=%s vendor=%lx device=%lx whql=%lx\n", id.Driver, id.Description,
           id.VendorId, id.DeviceId, id.WHQLLevel);
    HR("identifier bad adapter", IDirect3D8_GetAdapterIdentifier(d3d, 7, 0, &id));
    HR("display mode", IDirect3D8_GetAdapterDisplayMode(d3d, 0, &mode));
    printf("  mode %ux%u fmt=%d\n", mode.Width, mode.Height, mode.Format);
    n = IDirect3D8_GetAdapterModeCount(d3d, 0);
    printf("modes %u\n", n);
    for (i = 0; i < n && i < 400; i++)
        if (SUCCEEDED(IDirect3D8_EnumAdapterModes(d3d, 0, i, &mode)) && (i < 4 || i == n - 1))
            printf("  mode %u %ux%u fmt=%d\n", i, mode.Width, mode.Height, mode.Format);
    HR("enum past end", IDirect3D8_EnumAdapterModes(d3d, 0, n, &mode));
    for (i = 0; i < 3; i++)
        printf("devtype %d hal=%08lx win=%08lx\n", disp[i],
               IDirect3D8_CheckDeviceType(d3d, 0, D3DDEVTYPE_HAL, disp[i], disp[i], FALSE),
               IDirect3D8_CheckDeviceType(d3d, 0, D3DDEVTYPE_HAL, disp[i], disp[i], TRUE));
    for (f = 0; f < NFMT; f++)
    {
        printf("fmt %08x ", (unsigned)formats[f]);
        for (r = 0; r < sizeof(rtypes) / sizeof(rtypes[0]); r++)
            for (u = 0; u < sizeof(usages) / sizeof(usages[0]); u++)
            {
                HRESULT hr = IDirect3D8_CheckDeviceFormat(d3d, 0, D3DDEVTYPE_HAL, D3DFMT_X8R8G8B8,
                                                          usages[u], rtypes[r], formats[f]);
                printf("%c", hr == D3D_OK ? 'y' : hr == D3DERR_NOTAVAILABLE ? 'n'
                           : hr == D3DERR_INVALIDCALL ? 'i' : '?');
            }
        printf(" ms=%08lx ds=%08lx\n",
               IDirect3D8_CheckDeviceMultiSampleType(d3d, 0, D3DDEVTYPE_HAL, formats[f], TRUE,
                                                     D3DMULTISAMPLE_2_SAMPLES),
               IDirect3D8_CheckDepthStencilMatch(d3d, 0, D3DDEVTYPE_HAL, D3DFMT_X8R8G8B8,
                                                 D3DFMT_X8R8G8B8, formats[f]));
    }
}

static void device(IDirect3D8 *d3d, HWND hwnd)
{
    D3DPRESENT_PARAMETERS pp;
    IDirect3DDevice8 *dev = NULL;
    IDirect3DVertexBuffer8 *vb;
    IDirect3DTexture8 *tex;
    IDirect3DSurface8 *surf;
    D3DVERTEXBUFFER_DESC vbd;
    D3DSURFACE_DESC sd;
    D3DLOCKED_RECT lr;
    D3DVIEWPORT8 vp;
    D3DMATRIX m;
    D3DCAPS8 caps;
    BYTE *data;
    DWORD v, i, s, vs, sb, size;
    DWORD decl[] = {D3DVSD_STREAM(0), D3DVSD_REG(D3DVSDE_POSITION, D3DVSDT_FLOAT3),
                    D3DVSD_REG(D3DVSDE_DIFFUSE, D3DVSDT_D3DCOLOR), D3DVSD_STREAM(1),
                    D3DVSD_REG(D3DVSDE_TEXCOORD0, D3DVSDT_FLOAT2), D3DVSD_SKIP(2),
                    D3DVSD_CONST(0, 1), 0x3f800000, 0, 0, 0x3f800000, D3DVSD_END()};
    DWORD back[32];
    UINT n;
    HRESULT hr;

    memset(&caps, 0, sizeof(caps));
    HR("caps hal", IDirect3D8_GetDeviceCaps(d3d, 0, D3DDEVTYPE_HAL, &caps));
    dx_words("caps", &caps, sizeof(caps));
    HR("caps ref", IDirect3D8_GetDeviceCaps(d3d, 0, D3DDEVTYPE_REF, &caps));

    memset(&pp, 0, sizeof(pp));
    pp.Windowed = TRUE;
    pp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    pp.BackBufferFormat = D3DFMT_X8R8G8B8;
    pp.BackBufferWidth = 64;
    pp.BackBufferHeight = 64;
    pp.EnableAutoDepthStencil = TRUE;
    pp.AutoDepthStencilFormat = D3DFMT_D24S8;
    pp.hDeviceWindow = hwnd;
    hr = IDirect3D8_CreateDevice(d3d, 0, D3DDEVTYPE_HAL, hwnd, D3DCREATE_SOFTWARE_VERTEXPROCESSING,
                                 &pp, &dev);
    HR("create device", hr);
    if (FAILED(hr))
    {
        /* no3d may offer no depth format: retry without the auto depth-stencil. */
        pp.EnableAutoDepthStencil = FALSE;
        pp.AutoDepthStencilFormat = D3DFMT_UNKNOWN;
        hr = IDirect3D8_CreateDevice(d3d, 0, D3DDEVTYPE_HAL, hwnd, D3DCREATE_SOFTWARE_VERTEXPROCESSING, &pp, &dev);
        HR("create device no depth", hr);
    }
    if (FAILED(hr))
        return;
    HR("test coop", IDirect3DDevice8_TestCooperativeLevel(dev));

    for (s = 0; s < 180; s++)
    {
        v = 0xdeadbeef;
        if (SUCCEEDED(IDirect3DDevice8_GetRenderState(dev, (D3DRENDERSTATETYPE)s, &v)))
            printf("rs %lu=%lx\n", s, v);
    }
    for (i = 0; i < 9; i++)
        for (s = 0; s < 32; s++)
        {
            v = 0xdeadbeef;
            hr = IDirect3DDevice8_GetTextureStageState(dev, i, (D3DTEXTURESTAGESTATETYPE)s, &v);
            printf("tss %lu.%lu hr=%08lx v=%lx\n", i, s, (unsigned long)hr, v);
        }
    for (s = 0; s < 32; s++)
    {
        hr = IDirect3DDevice8_SetTextureStageState(dev, 2, (D3DTEXTURESTAGESTATETYPE)s, 3);
        v = 0xdeadbeef;
        IDirect3DDevice8_GetTextureStageState(dev, 2, (D3DTEXTURESTAGESTATETYPE)s, &v);
        printf("tss set 2.%lu hr=%08lx back=%lx\n", s, (unsigned long)hr, v);
    }
    memset(&m, 0, sizeof(m));
    m._11 = m._22 = m._33 = m._44 = 3.0f;
    m._42 = -1.0f;
    HR("set view", IDirect3DDevice8_SetTransform(dev, D3DTS_VIEW, &m));
    HR("multiply", IDirect3DDevice8_MultiplyTransform(dev, D3DTS_VIEW, &m));
    IDirect3DDevice8_GetTransform(dev, D3DTS_VIEW, &m);
    dx_words("view^2", &m, sizeof(m));
    vp.X = 3; vp.Y = 4; vp.Width = 20; vp.Height = 10; vp.MinZ = 0.0f; vp.MaxZ = 0.5f;
    HR("set viewport", IDirect3DDevice8_SetViewport(dev, &vp));
    memset(&vp, 0, sizeof(vp));
    IDirect3DDevice8_GetViewport(dev, &vp);
    dx_words("viewport", &vp, sizeof(vp));

    hr = IDirect3DDevice8_CreateVertexBuffer(dev, 120, D3DUSAGE_WRITEONLY, D3DFVF_XYZ | D3DFVF_TEX1,
                                             D3DPOOL_MANAGED, &vb);
    HR("vb", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DVertexBuffer8_GetDesc(vb, &vbd);
        printf("  vb fmt=%d type=%d usage=%lx pool=%d size=%u fvf=%lx\n", vbd.Format, vbd.Type,
               vbd.Usage, vbd.Pool, vbd.Size, vbd.FVF);
        if (SUCCEEDED(IDirect3DVertexBuffer8_Lock(vb, 0, 0, &data, 0)))
        {
            memset(data, 0x11, 120);
            IDirect3DVertexBuffer8_Unlock(vb);
        }
        HR("  stream", IDirect3DDevice8_SetStreamSource(dev, 0, vb, 20));
        printf("  vb release %lu\n", IDirect3DVertexBuffer8_Release(vb));
    }
    hr = IDirect3DDevice8_CreateTexture(dev, 64, 8, 0, 0, D3DFMT_R5G6B5, D3DPOOL_MANAGED, &tex);
    HR("tex", hr);
    if (SUCCEEDED(hr))
    {
        n = IDirect3DTexture8_GetLevelCount(tex);
        printf("  levels %u\n", n);
        for (i = 0; i < n; i++)
        {
            IDirect3DTexture8_GetLevelDesc(tex, i, &sd);
            printf("  level %lu %ux%u fmt=%d size=%u pool=%d\n", i, sd.Width, sd.Height,
                   sd.Format, sd.Size, sd.Pool);
        }
        if (SUCCEEDED(IDirect3DTexture8_LockRect(tex, 0, &lr, NULL, 0)))
        {
            printf("  pitch %d\n", lr.Pitch);
            IDirect3DTexture8_UnlockRect(tex, 0);
        }
        printf("  tex release %lu\n", IDirect3DTexture8_Release(tex));
    }
    hr = IDirect3DDevice8_CreateImageSurface(dev, 16, 16, D3DFMT_A8R8G8B8, &surf);
    HR("image surface", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DSurface8_GetDesc(surf, &sd);
        printf("  img %ux%u fmt=%d pool=%d size=%u\n", sd.Width, sd.Height, sd.Format, sd.Pool, sd.Size);
        printf("  img release %lu\n", IDirect3DSurface8_Release(surf));
    }

    vs = 0;
    hr = IDirect3DDevice8_CreateVertexShader(dev, decl, NULL, &vs, 0);
    HR("vs decl only", hr);
    if (SUCCEEDED(hr))
    {
        size = 0;
        HR("  decl size", IDirect3DDevice8_GetVertexShaderDeclaration(dev, vs, NULL, &size));
        printf("  decl bytes %lu\n", size);
        size = sizeof(back);
        memset(back, 0xcc, sizeof(back));
        IDirect3DDevice8_GetVertexShaderDeclaration(dev, vs, back, &size);
        for (i = 0; i < size / 4; i++)
            printf("  tok %lu %08lx\n", i, back[i]);
        HR("  set vs", IDirect3DDevice8_SetVertexShader(dev, vs));
        IDirect3DDevice8_GetVertexShader(dev, &v);
        printf("  vs is handle %d\n", v == vs);
        HR("  delete vs", IDirect3DDevice8_DeleteVertexShader(dev, vs));
        HR("  delete vs twice", IDirect3DDevice8_DeleteVertexShader(dev, vs));
    }
    HR("set fvf", IDirect3DDevice8_SetVertexShader(dev, D3DFVF_XYZRHW | D3DFVF_DIFFUSE));
    IDirect3DDevice8_GetVertexShader(dev, &v);
    printf("fvf back %lx\n", v);

    hr = IDirect3DDevice8_CreateStateBlock(dev, D3DSBT_ALL, &sb);
    HR("stateblock", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DDevice8_SetRenderState(dev, D3DRS_LIGHTING, FALSE);
        HR("  apply", IDirect3DDevice8_ApplyStateBlock(dev, sb));
        IDirect3DDevice8_GetRenderState(dev, D3DRS_LIGHTING, &v);
        printf("  lighting after apply %lx\n", v);
        HR("  delete", IDirect3DDevice8_DeleteStateBlock(dev, sb));
    }
    HR("clear", IDirect3DDevice8_Clear(dev, 0, NULL, D3DCLEAR_TARGET, 0xff0000ff, 1.0f, 0));
    HR("begin scene", IDirect3DDevice8_BeginScene(dev));
    HR("end scene", IDirect3DDevice8_EndScene(dev));
    HR("end scene twice", IDirect3DDevice8_EndScene(dev));
    HR("present", IDirect3DDevice8_Present(dev, NULL, NULL, NULL, NULL));
    if (SUCCEEDED(IDirect3DDevice8_GetBackBuffer(dev, 0, D3DBACKBUFFER_TYPE_MONO, &surf)))
    {
        IDirect3DSurface8_GetDesc(surf, &sd);
        printf("bb %ux%u fmt=%d usage=%lx\n", sd.Width, sd.Height, sd.Format, sd.Usage);
        IDirect3DSurface8_Release(surf);
    }
    printf("device release %lu\n", IDirect3DDevice8_Release(dev));
}

int main(void)
{
    CREATE8 create;
    IDirect3D8 *d3d;
    HWND hwnd;

    /* The guest's GL is Mesa's software renderer, which wined3d's GL backend refuses (no
     * ARB_fragment_shader): Direct3DCreate then returns NULL under both variants and nothing
     * past it runs. wined3d's no3d renderer needs no GPU, reads this from msvcrt's
     * environment (shared with this exe), and is the same C under both. */
    _putenv("WINE_D3D_CONFIG=renderer=no3d");
    dx_load(L"d3d8");
    create = (CREATE8)dx_proc("Direct3DCreate8");
    printf("create bad sdk %d\n", create(1) != NULL);
    d3d = create(D3D_SDK_VERSION);
    printf("create %d\n", d3d != NULL);
    if (!d3d)
        return DONE();
    hwnd = dx_window();
    adapter(d3d);
    device(d3d, hwnd);
    printf("d3d release %lu\n", IDirect3D8_Release(d3d));
    DestroyWindow(hwnd);
    return DONE();
}
