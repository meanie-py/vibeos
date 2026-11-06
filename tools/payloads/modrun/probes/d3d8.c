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

/* The lockable back buffer's colour bytes as one checksum. */
static void bb_hash(IDirect3DDevice8 *dev, const char *tag)
{
    IDirect3DSurface8 *bb;
    D3DLOCKED_RECT lr;
    HRESULT hr;
    int x, y;

    if (FAILED(IDirect3DDevice8_GetBackBuffer(dev, 0, D3DBACKBUFFER_TYPE_MONO, &bb)))
        return;
    hr = IDirect3DSurface8_LockRect(bb, &lr, NULL, D3DLOCK_READONLY);
    printf("%s lock hr=%08lx", tag, (unsigned long)hr);
    if (SUCCEEDED(hr))
    {
        unsigned long h = 2166136261u;
        for (y = 0; y < 64; y++)
            for (x = 0; x < 64; x++)
                h = (h ^ (((DWORD *)((BYTE *)lr.pBits + y * lr.Pitch))[x] & 0xffffff)) * 16777619u;
        printf(" hash=%08lx centre=%06lx", h, ((DWORD *)((BYTE *)lr.pBits + 32 * lr.Pitch))[32] & 0xffffff);
        IDirect3DSurface8_UnlockRect(bb);
    }
    printf("\n");
    IDirect3DSurface8_Release(bb);
}

/* Hand-assembled shader bytecode, as d3d8 takes it: no dcl instructions (the declaration
 * tokens bind the registers), so d3d8's own function-token walk and wined3d's sm1 parser and
 * GLSL generator all sit between these tokens and the read-back pixels. */
static const DWORD vs11[] = {
    0xfffe0101,                                     /* vs_1_1 */
    0x00000001, 0xc00f0000, 0x90e40000,             /* mov oPos, v0 */
    0x00000005, 0xd00f0000, 0x90e40005, 0xa0e40000, /* mul oD0, v5, c0 */
    0x0000ffff};
static const DWORD ps11[] = {
    0xffff0101,                                     /* ps_1_1 */
    0x00000005, 0x800f0000, 0x90e40000, 0xa0e40001, /* mul r0, v0, c1 */
    0x0000ffff};

static void shaders(IDirect3DDevice8 *dev, DWORD *decl, const void *verts, UINT stride)
{
    static const float c0[4] = {1.0f, 0.5f, 2.0f, 1.0f}, c1[4] = {0.5f, 1.0f, 1.0f, 1.0f};
    DWORD vs = 0, ps = 0, buf[32];
    float back[8];
    DWORD size;
    HRESULT hr;

    hr = IDirect3DDevice8_CreateVertexShader(dev, decl, vs11, &vs, 0);
    HR("s vs11", hr);
    if (FAILED(hr))
        return;
    size = 0;
    HR("  vs func size", IDirect3DDevice8_GetVertexShaderFunction(dev, vs, NULL, &size));
    memset(buf, 0, sizeof(buf));
    if (size <= sizeof(buf))
        IDirect3DDevice8_GetVertexShaderFunction(dev, vs, buf, &size);
    printf("  vs func %lu hash %08lx\n", size, dx_hash(buf, size <= sizeof(buf) ? size : 0));
    hr = IDirect3DDevice8_CreatePixelShader(dev, ps11, &ps);
    HR("s ps11", hr);
    if (SUCCEEDED(hr))
    {
        size = 0;
        HR("  ps func size", IDirect3DDevice8_GetPixelShaderFunction(dev, ps, NULL, &size));
        printf("  ps func %lu\n", size);
    }
    HR("s vsconst", IDirect3DDevice8_SetVertexShaderConstant(dev, 0, c0, 1));
    HR("s psconst", IDirect3DDevice8_SetPixelShaderConstant(dev, 1, c1, 1));
    HR("s vsconst far", IDirect3DDevice8_SetVertexShaderConstant(dev, 96, c0, 1));
    memset(back, 0xcc, sizeof(back));
    HR("s get vsconst", IDirect3DDevice8_GetVertexShaderConstant(dev, 0, back, 2));
    dx_words("  c", back, sizeof(back));
    memset(back, 0xcc, sizeof(back));
    HR("s get psconst", IDirect3DDevice8_GetPixelShaderConstant(dev, 1, back, 1));
    dx_words("  p", back, 16);

    IDirect3DDevice8_Clear(dev, 0, NULL, D3DCLEAR_TARGET, 0xff000000, 1.0f, 0);
    IDirect3DDevice8_BeginScene(dev);
    HR("s set vs", IDirect3DDevice8_SetVertexShader(dev, vs));
    HR("s vs draw", IDirect3DDevice8_DrawPrimitiveUP(dev, D3DPT_TRIANGLELIST, 1, verts, stride));
    IDirect3DDevice8_EndScene(dev);
    bb_hash(dev, "s vs11");
    if (ps)
    {
        IDirect3DDevice8_Clear(dev, 0, NULL, D3DCLEAR_TARGET, 0xff000000, 1.0f, 0);
        IDirect3DDevice8_BeginScene(dev);
        HR("s set ps", IDirect3DDevice8_SetPixelShader(dev, ps));
        HR("s ps draw", IDirect3DDevice8_DrawPrimitiveUP(dev, D3DPT_TRIANGLELIST, 1, verts, stride));
        IDirect3DDevice8_EndScene(dev);
        bb_hash(dev, "s vs11 ps11");
        IDirect3DDevice8_SetPixelShader(dev, 0);
        HR("s delete ps", IDirect3DDevice8_DeletePixelShader(dev, ps));
    }
    IDirect3DDevice8_SetVertexShader(dev, D3DFVF_XYZRHW | D3DFVF_DIFFUSE);
    HR("s delete vs", IDirect3DDevice8_DeleteVertexShader(dev, vs));
}

/* Only reachable with a real 3D device (the gl case): d3d8's draw paths, through an FVF and
 * through a declaration-only vertex shader -- d3d8's own token parser feeding the draw. */
static void render(IDirect3DDevice8 *dev)
{
    struct { float x, y, z, rhw; DWORD c; } tri[3] = {
        {4.0f, 60.0f, 0.5f, 1.0f, 0xffffff00},
        {32.0f, 2.0f, 0.5f, 1.0f, 0xff00ffff},
        {60.0f, 60.0f, 0.5f, 1.0f, 0xffff00ff}};
    struct { float x, y, z; DWORD c; } tri3[3] = {
        {-0.5f, -0.5f, 0.5f, 0xff102030}, {0.0f, 0.5f, 0.5f, 0xff405060}, {0.5f, -0.5f, 0.5f, 0xff708090}};
    DWORD decl[] = {D3DVSD_STREAM(0), D3DVSD_REG(D3DVSDE_POSITION, D3DVSDT_FLOAT3),
                    D3DVSD_REG(D3DVSDE_DIFFUSE, D3DVSDT_D3DCOLOR), D3DVSD_END()};
    D3DVIEWPORT8 vp = {0, 0, 64, 64, 0.0f, 1.0f};
    D3DMATRIX id;
    DWORD vs = 0;
    HRESULT hr;

    memset(&id, 0, sizeof(id));
    id._11 = id._22 = id._33 = id._44 = 1.0f;
    IDirect3DDevice8_SetViewport(dev, &vp);
    IDirect3DDevice8_SetTransform(dev, D3DTS_WORLD, &id);
    IDirect3DDevice8_SetTransform(dev, D3DTS_VIEW, &id);
    IDirect3DDevice8_SetTransform(dev, D3DTS_PROJECTION, &id);
    IDirect3DDevice8_SetRenderState(dev, D3DRS_ZENABLE, D3DZB_FALSE);
    IDirect3DDevice8_SetRenderState(dev, D3DRS_LIGHTING, FALSE);
    IDirect3DDevice8_SetRenderState(dev, D3DRS_CULLMODE, D3DCULL_NONE);
    HR("r clear", IDirect3DDevice8_Clear(dev, 0, NULL, D3DCLEAR_TARGET, 0xff336699, 1.0f, 0));
    bb_hash(dev, "r cleared");
    IDirect3DDevice8_BeginScene(dev);
    HR("r fvf", IDirect3DDevice8_SetVertexShader(dev, D3DFVF_XYZRHW | D3DFVF_DIFFUSE));
    HR("r tri", IDirect3DDevice8_DrawPrimitiveUP(dev, D3DPT_TRIANGLELIST, 1, tri, sizeof(tri[0])));
    IDirect3DDevice8_EndScene(dev);
    bb_hash(dev, "r triangle");
    hr = IDirect3DDevice8_CreateVertexShader(dev, decl, NULL, &vs, 0);
    HR("r decl vs", hr);
    if (SUCCEEDED(hr))
    {
        IDirect3DDevice8_Clear(dev, 0, NULL, D3DCLEAR_TARGET, 0xff000000, 1.0f, 0);
        IDirect3DDevice8_BeginScene(dev);
        HR("r set decl vs", IDirect3DDevice8_SetVertexShader(dev, vs));
        HR("r tri decl", IDirect3DDevice8_DrawPrimitiveUP(dev, D3DPT_TRIANGLELIST, 1, tri3, sizeof(tri3[0])));
        IDirect3DDevice8_EndScene(dev);
        bb_hash(dev, "r transformed");
        IDirect3DDevice8_SetVertexShader(dev, D3DFVF_XYZRHW | D3DFVF_DIFFUSE);
        HR("r delete decl vs", IDirect3DDevice8_DeleteVertexShader(dev, vs));
    }
    shaders(dev, decl, tri3, sizeof(tri3[0]));
    HR("r present", IDirect3DDevice8_Present(dev, NULL, NULL, NULL, NULL));
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
    /* render() reads the back buffer: d3d8 has no GetRenderTargetData. */
    pp.Flags = D3DPRESENTFLAG_LOCKABLE_BACKBUFFER;
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
    if (dx_invalid)
    {
        /* Stage 8, and the unused slots 0 and 12 (WINED3D_TSS_INVALID, ~0u, in the C's lookup:
         * GetTextureStageState(0, 0) reads texture_states[0][0xffffffff] and faults). */
        static const DWORD bad[][2] = {{8, 1}, {8, 13}, {0, 0}, {0, 12}};
        for (i = 0; i < sizeof(bad) / sizeof(bad[0]); i++)
        {
            v = 0xdeadbeef;
            hr = IDirect3DDevice8_GetTextureStageState(dev, bad[i][0], (D3DTEXTURESTAGESTATETYPE)bad[i][1], &v);
            printf("tss %lu.%lu hr=%08lx v=%lx\n", bad[i][0], bad[i][1], (unsigned long)hr, v);
        }
        printf("device release %lu\n", IDirect3DDevice8_Release(dev));
        return;
    }
    HR("test coop", IDirect3DDevice8_TestCooperativeLevel(dev));

    for (s = 0; s < 180; s++)
    {
        v = 0xdeadbeef;
        if (SUCCEEDED(IDirect3DDevice8_GetRenderState(dev, (D3DRENDERSTATETYPE)s, &v)))
            printf("rs %lu=%lx\n", s, v);
    }
    for (i = 0; i < 8; i++) /* stage 8, and states 0 and 12, are dx_invalid's */
        for (s = 1; s < 32; s++)
        {
            if (s == 12)
                continue;
            v = 0xdeadbeef;
            hr = IDirect3DDevice8_GetTextureStageState(dev, i, (D3DTEXTURESTAGESTATETYPE)s, &v);
            printf("tss %lu.%lu hr=%08lx v=%lx\n", i, s, (unsigned long)hr, v);
        }
    for (s = 1; s < 32; s++)
    {
        if (s == 12)
            continue;
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
    render(dev);
    printf("device release %lu\n", IDirect3DDevice8_Release(dev));
}

int main(int argc, char **argv)
{
    CREATE8 create;
    IDirect3D8 *d3d;
    HWND hwnd;

    dx_renderer(argc, argv);
    dx_load(L"d3d8");
    dx_wined3d();
    create = (CREATE8)dx_proc("Direct3DCreate8");
    /* wined3d ignores the SDK version, so this is a whole second d3d8 with its own wined3d
     * adapter; leaked, it left the i386 guest's next adapter init unable to create any GL
     * context (ERROR_NOT_ENOUGH_MEMORY) -- roadmap "Known bugs". */
    dx_vm("start");
    d3d = create(1);
    printf("create bad sdk %d\n", d3d != NULL);
    if (d3d)
        printf("bad sdk release %lu\n", IDirect3D8_Release(d3d));
    dx_vm("after first");
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
