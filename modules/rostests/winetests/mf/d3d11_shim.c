/* d3d11.dll is built only at DLL_EXPORT_VERSION >= 0x600, so there is no import library
 * to link the suite's direct D3D11CreateDevice calls against. Resolve it at run time the
 * way mfplat's suite does (pD3D11CreateDevice); without d3d11 the callers' "no device"
 * skip path runs. */
#include <stdarg.h>
#include "windef.h"
#include "winbase.h"
#include "d3d11.h"

HRESULT WINAPI D3D11CreateDevice(IDXGIAdapter *adapter, D3D_DRIVER_TYPE driver_type, HMODULE swrast,
        UINT flags, const D3D_FEATURE_LEVEL *feature_levels, UINT levels, UINT sdk_version,
        ID3D11Device **device, D3D_FEATURE_LEVEL *obtained_feature_level, ID3D11DeviceContext **context)
{
    static HRESULT (WINAPI *create)(IDXGIAdapter *, D3D_DRIVER_TYPE, HMODULE, UINT,
            const D3D_FEATURE_LEVEL *, UINT, UINT, ID3D11Device **, D3D_FEATURE_LEVEL *,
            ID3D11DeviceContext **);
    HMODULE module;

    if (!create && (module = LoadLibraryW(L"d3d11.dll")))
        create = (void *)GetProcAddress(module, "D3D11CreateDevice");
    if (!create)
        return E_NOTIMPL;
    return create(adapter, driver_type, swrast, flags, feature_levels, levels, sdk_version,
            device, obtained_feature_level, context);
}
