/* modrun probe: dsound (dll/directx/wine/dsound -> dsound-rust) against its pre-flip C.
 *
 * The VM has no audio device, so this grades what needs none: enumeration, GetDeviceID's
 * default-device mapping, the class factory and the uninitialized-object paths, the private
 * property set, and the create calls' failure codes. The mixer, format conversion and 3D
 * code need a wave device to be reachable at all -- owed until the VM grows one. */
#include "dx_probe.h"
#include <mmsystem.h>
#include <dsound.h>
#include <dsconf.h>

typedef HRESULT(WINAPI *ENUMA)(LPDSENUMCALLBACKA, void *);
typedef HRESULT(WINAPI *CREATE8)(const GUID *, IDirectSound8 **, IUnknown *);
typedef HRESULT(WINAPI *CREATE)(const GUID *, IDirectSound **, IUnknown *);
typedef HRESULT(WINAPI *CCREATE)(const GUID *, IDirectSoundCapture **, IUnknown *);
typedef HRESULT(WINAPI *DEVID)(const GUID *, GUID *);
typedef HRESULT(WINAPI *GETCLASS)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *CANUNLOAD)(void);

static int seen;
static BOOL CALLBACK enum_cb(GUID *g, const char *desc, const char *mod, void *ctx)
{
    printf("  %s guid=%d desc=%s module=%s\n", (const char *)ctx, g != NULL, desc, mod);
    seen++;
    return TRUE;
}

static void guid(const char *tag, const GUID *g)
{
    printf("%s %08lx-%04x-%04x\n", tag, g->Data1, g->Data2, g->Data3);
}

static const GUID null_guid;

int main(void)
{
    static const GUID *ids[] = {&DSDEVID_DefaultPlayback, &DSDEVID_DefaultCapture,
                                &DSDEVID_DefaultVoicePlayback, &DSDEVID_DefaultVoiceCapture,
                                &null_guid, &IID_IUnknown};
    IDirectSound8 *ds8 = NULL;
    IDirectSound *ds = NULL;
    IDirectSoundCapture *cap = NULL;
    IClassFactory *cf = NULL;
    IKsPropertySet *ps = NULL;
    DSCAPS caps;
    DWORD speaker, i, ret;
    GUID out;
    HRESULT hr;

    dx_load(L"dsound");
    HR("enumerate", ((ENUMA)dx_proc("DirectSoundEnumerateA"))(enum_cb, "play"));
    HR("enumerate capture", ((ENUMA)dx_proc("DirectSoundCaptureEnumerateA"))(enum_cb, "capture"));
    printf("devices %d\n", seen);
    for (i = 0; i < sizeof(ids) / sizeof(ids[0]); i++)
    {
        memset(&out, 0xab, sizeof(out));
        hr = ((DEVID)dx_proc("GetDeviceID"))(ids[i], &out);
        printf("deviceid %lu hr=%08lx\n", i, (unsigned long)hr);
        if (SUCCEEDED(hr))
            guid("  ->", &out);
    }
    HR("deviceid null out", ((DEVID)dx_proc("GetDeviceID"))(&DSDEVID_DefaultPlayback, NULL));

    HR("create8", ((CREATE8)dx_proc("DirectSoundCreate8"))(NULL, &ds8, NULL));
    printf("  create8 object %d\n", ds8 != NULL);
    HR("create8 bogus guid", ((CREATE8)dx_proc("DirectSoundCreate8"))(&IID_IUnknown, &ds8, NULL));
    HR("create", ((CREATE)dx_proc("DirectSoundCreate"))(NULL, &ds, NULL));
    HR("create null out", ((CREATE)dx_proc("DirectSoundCreate"))(NULL, NULL, NULL));
    HR("capture create", ((CCREATE)dx_proc("DirectSoundCaptureCreate"))(NULL, &cap, NULL));
    if (ds8)
        IDirectSound8_Release(ds8);
    if (ds)
        IDirectSound_Release(ds);
    if (cap)
        IDirectSoundCapture_Release(cap);
    ds8 = NULL;

    hr = ((GETCLASS)dx_proc("DllGetClassObject"))(&CLSID_DirectSound8, &IID_IClassFactory, (void **)&cf);
    HR("class object ds8", hr);
    if (SUCCEEDED(hr))
    {
        HR("  bad iid", IClassFactory_CreateInstance(cf, NULL, &IID_IClassFactory, (void **)&ds8));
        HR("  aggregation", IClassFactory_CreateInstance(cf, (IUnknown *)cf, &IID_IDirectSound8, (void **)&ds8));
        hr = IClassFactory_CreateInstance(cf, NULL, &IID_IDirectSound8, (void **)&ds8);
        HR("  instance", hr);
        if (SUCCEEDED(hr))
        {
            memset(&caps, 0, sizeof(caps));
            caps.dwSize = sizeof(caps);
            HR("  caps uninit", IDirectSound8_GetCaps(ds8, &caps));
            HR("  speaker uninit", IDirectSound8_GetSpeakerConfig(ds8, &speaker));
            HR("  coop uninit", IDirectSound8_SetCooperativeLevel(ds8, GetDesktopWindow(), DSSCL_PRIORITY));
            HR("  verify uninit", IDirectSound8_VerifyCertification(ds8, &ret));
            HR("  initialize", IDirectSound8_Initialize(ds8, NULL));
            HR("  initialize twice", IDirectSound8_Initialize(ds8, NULL));
            printf("  instance release %lu\n", IDirectSound8_Release(ds8));
        }
        printf("  factory release %lu\n", IClassFactory_Release(cf));
    }
    HR("class object bogus", ((GETCLASS)dx_proc("DllGetClassObject"))(&IID_IUnknown, &IID_IClassFactory,
                                                                    (void **)&cf));

    hr = ((GETCLASS)dx_proc("DllGetClassObject"))(&CLSID_DirectSoundPrivate, &IID_IClassFactory, (void **)&cf);
    HR("class object private", hr);
    if (SUCCEEDED(hr))
    {
        hr = IClassFactory_CreateInstance(cf, NULL, &IID_IKsPropertySet, (void **)&ps);
        HR("  propset", hr);
        if (SUCCEEDED(hr))
        {
            DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A_DATA map;
            DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A_DATA desc;
            ULONG got = 0, support = 0xffffffff;
            for (i = 0; i < 8; i++)
            {
                hr = IKsPropertySet_QuerySupport(ps, &DSPROPSETID_DirectSoundDevice, i, &support);
                printf("  support %lu hr=%08lx flags=%lx\n", i, (unsigned long)hr, support);
            }
            HR("  bogus set", IKsPropertySet_QuerySupport(ps, &IID_IUnknown, 0, &support));
            memset(&map, 0, sizeof(map));
            map.DeviceName = (char *)"no such device";
            map.DataFlow = DIRECTSOUNDDEVICE_DATAFLOW_RENDER;
            hr = IKsPropertySet_Get(ps, &DSPROPSETID_DirectSoundDevice,
                                    DSPROPERTY_DIRECTSOUNDDEVICE_WAVEDEVICEMAPPING_A, NULL, 0, &map,
                                    sizeof(map), &got);
            printf("  mapping hr=%08lx got=%lu\n", (unsigned long)hr, got);
            memset(&desc, 0, sizeof(desc));
            desc.DeviceId = DSDEVID_DefaultPlayback;
            hr = IKsPropertySet_Get(ps, &DSPROPSETID_DirectSoundDevice,
                                    DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A, NULL, 0, &desc,
                                    sizeof(desc), &got);
            printf("  description hr=%08lx got=%lu\n", (unsigned long)hr, got);
            HR("  set readonly", IKsPropertySet_Set(ps, &DSPROPSETID_DirectSoundDevice,
                                                    DSPROPERTY_DIRECTSOUNDDEVICE_DESCRIPTION_A, NULL,
                                                    0, &desc, sizeof(desc)));
            printf("  propset release %lu\n", IKsPropertySet_Release(ps));
        }
        IClassFactory_Release(cf);
    }
    HR("can unload", ((CANUNLOAD)dx_proc("DllCanUnloadNow"))());
    return DONE();
}
