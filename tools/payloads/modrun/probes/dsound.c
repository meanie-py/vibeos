/* modrun probe: dsound (dll/directx/wine/dsound -> dsound-rust) against its pre-flip C.
 *
 * Without a device it grades enumeration, GetDeviceID's default-device mapping, the class
 * factory and the uninitialized-object paths, the private property set, and the create calls'
 * failure codes. modrun boots with an AC97 (payload.yaml), and with a wave device present
 * device() also drives buffer creation and its format validation, primary and secondary
 * buffer state, locks, notifications, duplication, the 3D buffer and listener, and a short
 * play of every format so the mixer, resampler and converters run. Playback is real time, so
 * only order-free facts are printed (a cursor moved, a buffer stopped) -- never a position. */
#include "dx_probe.h"
#include <mmsystem.h>
#include <mmreg.h>
#include <setupapi.h>
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

/* {00000003-0000-0010-8000-00aa00389b71}, {00000001-...}: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT/PCM. */
static const GUID sub_float = {3, 0, 0x10, {0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71}};
static const GUID sub_pcm = {1, 0, 0x10, {0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71}};

static void fmt(WAVEFORMATEXTENSIBLE *w, WORD tag, WORD ch, DWORD rate, WORD bits)
{
    memset(w, 0, sizeof(*w));
    w->Format.wFormatTag = tag;
    w->Format.nChannels = ch;
    w->Format.nSamplesPerSec = rate;
    w->Format.wBitsPerSample = bits;
    w->Format.nBlockAlign = ch * bits / 8;
    w->Format.nAvgBytesPerSec = rate * w->Format.nBlockAlign;
    if (tag == WAVE_FORMAT_EXTENSIBLE)
    {
        w->Format.cbSize = sizeof(*w) - sizeof(WAVEFORMATEX);
        w->Samples.wValidBitsPerSample = bits;
        w->dwChannelMask = ch == 1 ? 0x4 : ch == 2 ? 0x3 : 0x3f;
        w->SubFormat = bits == 32 ? sub_float : sub_pcm;
    }
}

static IDirectSoundBuffer *make(IDirectSound8 *ds, const char *tag, DWORD flags, DWORD bytes,
                                WAVEFORMATEXTENSIBLE *w)
{
    DSBUFFERDESC d;
    IDirectSoundBuffer *b = NULL;
    HRESULT hr;
    memset(&d, 0, sizeof(d));
    d.dwSize = sizeof(d);
    d.dwFlags = flags;
    d.dwBufferBytes = bytes;
    d.lpwfxFormat = w ? &w->Format : NULL;
    hr = IDirectSound8_CreateSoundBuffer(ds, &d, &b, NULL);
    printf("%s hr=%08lx\n", tag, (unsigned long)hr);
    return SUCCEEDED(hr) ? b : NULL;
}

static void fill(IDirectSoundBuffer *b, BYTE seed)
{
    void *p1, *p2;
    DWORD n1, n2, i;
    if (SUCCEEDED(IDirectSoundBuffer_Lock(b, 0, 0, &p1, &n1, &p2, &n2, DSBLOCK_ENTIREBUFFER)))
    {
        for (i = 0; i < n1; i++)
            ((BYTE *)p1)[i] = (BYTE)(seed + i * 7);
        IDirectSoundBuffer_Unlock(b, p1, n1, p2, n2);
    }
}

/* Play for a while, then stop: the mixer thread mixes, converts and resamples it. */
static void play(IDirectSoundBuffer *b, const char *tag, DWORD flags)
{
    DWORD play0 = 0, play1 = 0, wr, st = 0;
    HRESULT hr;
    IDirectSoundBuffer_GetCurrentPosition(b, &play0, &wr);
    hr = IDirectSoundBuffer_Play(b, 0, 0, flags);
    IDirectSoundBuffer_GetStatus(b, &st);
    printf("  %s play hr=%08lx playing=%d looping=%d\n", tag, (unsigned long)hr,
           !!(st & DSBSTATUS_PLAYING), !!(st & DSBSTATUS_LOOPING));
    Sleep(300);
    IDirectSoundBuffer_GetCurrentPosition(b, &play1, &wr);
    IDirectSoundBuffer_GetStatus(b, &st);
    /* A non-looping buffer may have finished by now; either way the cursor left 0. */
    printf("  %s moved=%d\n", tag, play1 != play0 || !(st & DSBSTATUS_PLAYING));
    HR("  stop", IDirectSoundBuffer_Stop(b));
    IDirectSoundBuffer_GetStatus(b, &st);
    printf("  %s stopped=%d\n", tag, !(st & DSBSTATUS_PLAYING));
}

static void buffer3d(IDirectSoundBuffer *b)
{
    IDirectSound3DBuffer *b3;
    IDirectSound3DListener *l3 = NULL;
    DS3DBUFFER p;
    D3DVECTOR v;
    D3DVALUE f;
    DWORD a, c;
    LONG o;
    HRESULT hr = IDirectSoundBuffer_QueryInterface(b, &IID_IDirectSound3DBuffer, (void **)&b3);
    HR("  3d qi", hr);
    if (FAILED(hr))
        return;
    memset(&p, 0, sizeof(p));
    p.dwSize = sizeof(p);
    HR("  3d all", IDirectSound3DBuffer_GetAllParameters(b3, &p));
    dx_words("  3d defaults", &p, sizeof(p));
    HR("  3d pos", IDirectSound3DBuffer_SetPosition(b3, 1.5f, -2.0f, 3.25f, DS3D_IMMEDIATE));
    HR("  3d vel", IDirectSound3DBuffer_SetVelocity(b3, 0.5f, 0.0f, -1.0f, DS3D_DEFERRED));
    HR("  3d cone", IDirectSound3DBuffer_SetConeAngles(b3, 30, 90, DS3D_IMMEDIATE));
    HR("  3d cone bad", IDirectSound3DBuffer_SetConeAngles(b3, 400, 90, DS3D_IMMEDIATE));
    HR("  3d cone vol", IDirectSound3DBuffer_SetConeOutsideVolume(b3, -1200, DS3D_IMMEDIATE));
    HR("  3d min", IDirectSound3DBuffer_SetMinDistance(b3, 2.0f, DS3D_IMMEDIATE));
    HR("  3d max", IDirectSound3DBuffer_SetMaxDistance(b3, 50.0f, DS3D_IMMEDIATE));
    HR("  3d mode", IDirectSound3DBuffer_SetMode(b3, DS3DMODE_HEADRELATIVE, DS3D_IMMEDIATE));
    HR("  3d mode bad", IDirectSound3DBuffer_SetMode(b3, 7, DS3D_IMMEDIATE));
    IDirectSound3DBuffer_GetPosition(b3, &v);
    dx_words("  3d pos back", &v, sizeof(v));
    IDirectSound3DBuffer_GetVelocity(b3, &v);
    dx_words("  3d vel deferred", &v, sizeof(v));
    IDirectSound3DBuffer_GetConeAngles(b3, &a, &c);
    IDirectSound3DBuffer_GetConeOutsideVolume(b3, &o);
    IDirectSound3DBuffer_GetMinDistance(b3, &f);
    printf("  3d cone %lu %lu outside %ld min %08lx\n", a, c, o, *(DWORD *)&f);
    IDirectSound3DBuffer_GetMode(b3, &a);
    printf("  3d mode back %lu\n", a);
    if (SUCCEEDED(IDirectSoundBuffer_QueryInterface(b, &IID_IDirectSound3DListener, (void **)&l3)))
    {
        printf("  3d listener from buffer\n");
        IDirectSound3DListener_Release(l3);
    }
    printf("  3d release %lu\n", IDirectSound3DBuffer_Release(b3));
}

static void listener(IDirectSoundBuffer *primary)
{
    IDirectSound3DListener *l;
    DS3DLISTENER p;
    D3DVALUE f;
    HRESULT hr = IDirectSoundBuffer_QueryInterface(primary, &IID_IDirectSound3DListener, (void **)&l);
    HR("listener qi", hr);
    if (FAILED(hr))
        return;
    memset(&p, 0, sizeof(p));
    p.dwSize = sizeof(p);
    HR("  listener all", IDirectSound3DListener_GetAllParameters(l, &p));
    dx_words("  listener defaults", &p, sizeof(p));
    HR("  distance", IDirectSound3DListener_SetDistanceFactor(l, 0.5f, DS3D_IMMEDIATE));
    HR("  distance bad", IDirectSound3DListener_SetDistanceFactor(l, 0.0f, DS3D_IMMEDIATE));
    HR("  rolloff", IDirectSound3DListener_SetRolloffFactor(l, 2.0f, DS3D_DEFERRED));
    HR("  rolloff bad", IDirectSound3DListener_SetRolloffFactor(l, 11.0f, DS3D_IMMEDIATE));
    HR("  doppler", IDirectSound3DListener_SetDopplerFactor(l, 1.5f, DS3D_IMMEDIATE));
    HR("  orient", IDirectSound3DListener_SetOrientation(l, 0, 0, 1, 0, 1, 0, DS3D_IMMEDIATE));
    IDirectSound3DListener_GetRolloffFactor(l, &f);
    printf("  rolloff before commit %08lx\n", *(DWORD *)&f);
    HR("  commit", IDirectSound3DListener_CommitDeferredSettings(l));
    IDirectSound3DListener_GetRolloffFactor(l, &f);
    printf("  rolloff after commit %08lx\n", *(DWORD *)&f);
    printf("  listener release %lu\n", IDirectSound3DListener_Release(l));
}

/* winmm underneath dsound: ReactOS's dsound opens the device with waveOutOpen, and maps its
 * failure to DSERR_GENERIC. Same calls under both variants (winmm is not dsound), so this
 * only localises a create failure; loaded by name so the probe needs no winmm import. */
typedef UINT(WINAPI *NUMDEVS)(void);
typedef MMRESULT(WINAPI *DEVCAPS)(UINT_PTR, WAVEOUTCAPSA *, UINT);
typedef MMRESULT(WINAPI *OPEN)(HWAVEOUT *, UINT, const WAVEFORMATEX *, DWORD_PTR, DWORD_PTR, DWORD);
typedef MMRESULT(WINAPI *CLOSE)(HWAVEOUT);
typedef MMRESULT(WINAPI *GETVOL)(HWAVEOUT, DWORD *);
typedef MMRESULT(WINAPI *SETVOL)(HWAVEOUT, DWORD);
static void winmm(void)
{
    HMODULE m = LoadLibraryA("winmm.dll");
    WAVEFORMATEXTENSIBLE w;
    WAVEOUTCAPSA caps;
    HWAVEOUT h;
    UINT n, i;
    MMRESULT r;
    if (!m)
        return;
    n = ((NUMDEVS)GetProcAddress(m, "waveOutGetNumDevs"))();
    printf("waveout devs %u\n", n);
    for (i = 0; i < n; i++)
    {
        memset(&caps, 0, sizeof(caps));
        r = ((DEVCAPS)GetProcAddress(m, "waveOutGetDevCapsA"))(i, &caps, sizeof(caps));
        printf("  wo %u caps=%u name=%s formats=%lx ch=%u\n", i, r, caps.szPname, caps.dwFormats, caps.wChannels);
    }
    fmt(&w, WAVE_FORMAT_PCM, 2, 44100, 16);
    r = ((OPEN)GetProcAddress(m, "waveOutOpen"))(NULL, 0, &w.Format, 0, 0, WAVE_FORMAT_QUERY);
    printf("  wo query 0 = %u\n", r);
    r = ((OPEN)GetProcAddress(m, "waveOutOpen"))(&h, 0, &w.Format, 0, 0, CALLBACK_NULL);
    printf("  wo open 0 = %u\n", r);
    if (!r)
        ((CLOSE)GetProcAddress(m, "waveOutClose"))(h);
    r = ((OPEN)GetProcAddress(m, "waveOutOpen"))(&h, 0, &w.Format, 0, 0, CALLBACK_NULL | WAVE_MAPPED);
    printf("  wo open 0 mapped = %u\n", r);
    if (!r)
        ((CLOSE)GetProcAddress(m, "waveOutClose"))(h);
    r = ((OPEN)GetProcAddress(m, "waveOutOpen"))(&h, WAVE_MAPPER, &w.Format, 0, 0, CALLBACK_NULL);
    printf("  wo open mapper = %u\n", r);
    if (!r)
        ((CLOSE)GetProcAddress(m, "waveOutClose"))(h);
    /* The primary buffer's volume is the device's: dsound reads it with waveOutGetVolume and,
     * in the C, uses the result even when that call fails. */
    {
        DWORD v = 0x12345678;
        r = ((GETVOL)GetProcAddress(m, "waveOutGetVolume"))((HWAVEOUT)0, &v);
        printf("  wo getvol 0 = %u%s\n", r, r ? "" : " (value not printed: device state)");
        r = ((SETVOL)GetProcAddress(m, "waveOutSetVolume"))((HWAVEOUT)0, 0x80008000);
        printf("  wo setvol 0 = %u\n", r);
        v = 0x12345678;
        r = ((GETVOL)GetProcAddress(m, "waveOutGetVolume"))((HWAVEOUT)0, &v);
        printf("  wo getvol back = %u v=%08lx\n", r, r ? 0 : v);
    }
}

/* Below winmm: the AC97 render filter driven straight through ksuser, the same KsCreatePin
 * mmixer makes. Cuts the stack in two -- if this works and waveOutOpen does not, the fault is
 * in the user layers (mmixer, wdmaud.drv, mmebuddy); if it fails, in ks/portcls/ac97. Local
 * definitions throughout, so the probe needs neither ks.h's GUID macros nor new imports. */
static const GUID cat_render = {0x65e8773e, 0x8f56, 0x11d0, {0xa3, 0xb9, 0x00, 0xa0, 0xc9, 0x22, 0x31, 0x96}};
static const GUID set_pin = {0x8c134960, 0x51ad, 0x11cf, {0x87, 0x8a, 0x94, 0xf8, 0x01, 0xc1, 0x00, 0x00}};
static const GUID if_standard = {0x1a8766a0, 0x62ce, 0x11cf, {0xa5, 0xd6, 0x28, 0xdb, 0x04, 0xc1, 0x00, 0x00}};
static const GUID medium_standard = {0x4747b320, 0x62ce, 0x11cf, {0xa5, 0xd6, 0x28, 0xdb, 0x04, 0xc1, 0x00, 0x00}};
static const GUID type_audio = {0x73647561, 0, 0x10, {0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71}};
static const GUID spec_wfx = {0x05589f81, 0xc356, 0x11ce, {0xbf, 0x01, 0x00, 0xaa, 0x00, 0x55, 0x59, 0x5a}};
struct ksid { GUID set; ULONG id, flags; };
struct ksp_pin { struct ksid prop; ULONG pin, reserved; };
#pragma pack(push, 1)
struct pin_connect {
    struct ksid iface, medium;
    ULONG pin;
    HANDLE to;
    ULONG prio_class, prio_sub;
    struct { ULONG size, flags, sample, reserved; GUID major, sub, spec; } fmt;
    WAVEFORMATEX wfx;
};
#pragma pack(pop)
typedef void *(WINAPI *GETCLASSDEVS)(const GUID *, const WCHAR *, HWND, DWORD);
typedef BOOL(WINAPI *ENUMIFACES)(void *, void *, const GUID *, DWORD, SP_DEVICE_INTERFACE_DATA *);
typedef BOOL(WINAPI *IFACEDETAIL)(void *, SP_DEVICE_INTERFACE_DATA *, SP_DEVICE_INTERFACE_DETAIL_DATA_W *, DWORD, DWORD *, void *);
typedef DWORD(WINAPI *CREATEPIN)(HANDLE, struct pin_connect *, ACCESS_MASK, HANDLE *);
static ULONG pin_prop(HANDLE f, ULONG id, ULONG pin, BOOL *ok)
{
    struct ksp_pin q = {{set_pin, id, 1 /* KSPROPERTY_TYPE_GET */}, pin, 0};
    ULONG v = 0xffffffff;
    DWORD got = 0;
    *ok = DeviceIoControl(f, 0x2f0003 /* IOCTL_KS_PROPERTY */, &q, sizeof(q), &v, sizeof(v), &got, NULL);
    return v;
}
static void ks_direct(void)
{
    HMODULE sa = LoadLibraryA("setupapi.dll"), ku = LoadLibraryA("ksuser.dll");
    union { SP_DEVICE_INTERFACE_DETAIL_DATA_W d; BYTE b[1024]; } u;
    SP_DEVICE_INTERFACE_DATA ifd;
    struct pin_connect pc;
    void *set;
    HANDLE f, pin;
    ULONG pins, i, flow, comm, n;
    BOOL ok, ok2;
    DWORD r;

    if (!sa || !ku)
    {
        printf("ks: setupapi=%d ksuser=%d\n", sa != NULL, ku != NULL);
        return;
    }
    set = ((GETCLASSDEVS)GetProcAddress(sa, "SetupDiGetClassDevsW"))(&cat_render, NULL, NULL, 0x12);
    for (n = 0;; n++)
    {
        memset(&ifd, 0, sizeof(ifd));
        ifd.cbSize = sizeof(ifd);
        if (!((ENUMIFACES)GetProcAddress(sa, "SetupDiEnumDeviceInterfaces"))(set, NULL, &cat_render, n, &ifd))
            break;
        u.d.cbSize = sizeof(u.d);
        if (!((IFACEDETAIL)GetProcAddress(sa, "SetupDiGetDeviceInterfaceDetailW"))(set, &ifd, &u.d, sizeof(u), NULL, NULL))
            continue;
        f = CreateFileW(u.d.DevicePath, GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING, 0, NULL);
        printf("ks render filter %lu wave=%d open=%d err=%lu\n", n, wcsstr(u.d.DevicePath, L"\\Wave") != NULL,
               f != INVALID_HANDLE_VALUE, f == INVALID_HANDLE_VALUE ? GetLastError() : 0);
        if (f == INVALID_HANDLE_VALUE)
            continue;
        pins = pin_prop(f, 1 /* CTYPES */, 0, &ok);
        printf("  pins ok=%d n=%lu\n", ok, pins);
        for (i = 0; ok && i < pins && i < 16; i++)
        {
            flow = pin_prop(f, 2 /* DATAFLOW */, i, &ok2);
            comm = pin_prop(f, 7 /* COMMUNICATION */, i, &ok2);
            memset(&pc, 0, sizeof(pc));
            pc.iface.set = if_standard;
            pc.medium.set = medium_standard;
            pc.pin = i;
            pc.prio_class = 0x40000000;
            pc.prio_sub = 1;
            pc.fmt.size = sizeof(pc.fmt) + sizeof(WAVEFORMATEX);
            pc.fmt.sample = 4;
            pc.fmt.major = type_audio;
            pc.fmt.sub = sub_pcm;
            pc.fmt.spec = spec_wfx;
            pc.wfx.wFormatTag = WAVE_FORMAT_PCM;
            pc.wfx.nChannels = 2;
            pc.wfx.nSamplesPerSec = 44100;
            pc.wfx.wBitsPerSample = 16;
            pc.wfx.nBlockAlign = 4;
            pc.wfx.nAvgBytesPerSec = 44100 * 4;
            pin = NULL;
            r = ((CREATEPIN)GetProcAddress(ku, "KsCreatePin"))(f, &pc, GENERIC_WRITE, &pin);
            printf("  pin %lu flow=%lu comm=%lu create=%08lx\n", i, flow, comm, r);
            if (!r && pin)
                CloseHandle(pin);
        }
        CloseHandle(f);
    }
    printf("ks render filters %lu\n", n);
}

static void device(HWND hwnd)
{
    static const struct { const char *tag; WORD tag_; WORD ch; DWORD rate; WORD bits; } fmts[] = {
        {"pcm8 mono 11k", WAVE_FORMAT_PCM, 1, 11025, 8},
        {"pcm16 stereo 44k", WAVE_FORMAT_PCM, 2, 44100, 16},
        {"pcm16 mono 22k", WAVE_FORMAT_PCM, 1, 22050, 16},
        {"pcm24 stereo 48k", WAVE_FORMAT_PCM, 2, 48000, 24},
        {"pcm32 stereo 96k", WAVE_FORMAT_PCM, 2, 96000, 32},
        {"float stereo 48k", WAVE_FORMAT_IEEE_FLOAT, 2, 48000, 32},
        {"ext float 5.1 48k", WAVE_FORMAT_EXTENSIBLE, 6, 48000, 32},
        {"ext pcm16 stereo", WAVE_FORMAT_EXTENSIBLE, 2, 44100, 16},
        {"pcm12 bad", WAVE_FORMAT_PCM, 2, 44100, 12},
        {"pcm 200k", WAVE_FORMAT_PCM, 2, 200000, 16},
        /* dx_invalid's: the C divides by each (nBlockAlign 0; freqAdjust 0 at 0 Hz) and faults */
        {"pcm 0ch bad", WAVE_FORMAT_PCM, 0, 44100, 16},
        {"adpcm", 2, 1, 22050, 4},
        {"pcm 0hz bad", WAVE_FORMAT_PCM, 2, 0, 16},
    };
    size_t nfmts = sizeof(fmts) / sizeof(fmts[0]), nbad = 3;
    IDirectSound8 *ds = NULL;
    IDirectSoundBuffer *prim, *b, *dup;
    IDirectSoundBuffer8 *b8;
    IDirectSoundNotify *n;
    DSBPOSITIONNOTIFY pos[3];
    WAVEFORMATEXTENSIBLE w, got;
    DSBCAPS bc;
    DSCAPS caps;
    void *p1, *p2;
    DWORD n1, n2, i, sz, speaker, v, align;
    LONG l;
    HRESULT hr;

    winmm();
    hr = ((CREATE8)dx_proc("DirectSoundCreate8"))(NULL, &ds, NULL);
    if (FAILED(hr))
    {
        printf("no device: mixer not graded\n");
        return;
    }
    printf("device present\n");
    memset(&caps, 0, sizeof(caps));
    caps.dwSize = sizeof(caps);
    HR("caps", IDirectSound8_GetCaps(ds, &caps));
    printf("  caps flags=%lx min=%lu max=%lu prim=%lu\n", caps.dwFlags, caps.dwMinSecondarySampleRate,
           caps.dwMaxSecondarySampleRate, caps.dwPrimaryBuffers);
    HR("caps bad size", (caps.dwSize = 4, IDirectSound8_GetCaps(ds, &caps)));
    HR("speaker", IDirectSound8_GetSpeakerConfig(ds, &speaker));
    HR("buffer before coop", make(ds, "  probe", 0, 4096, (fmt(&w, WAVE_FORMAT_PCM, 2, 44100, 16), &w)) ? S_OK : S_FALSE);
    HR("coop bad", IDirectSound8_SetCooperativeLevel(ds, hwnd, 9));
    HR("coop", IDirectSound8_SetCooperativeLevel(ds, hwnd, DSSCL_PRIORITY));

    prim = make(ds, "primary", DSBCAPS_PRIMARYBUFFER | DSBCAPS_CTRL3D | DSBCAPS_CTRLVOLUME, 0, NULL);
    make(ds, "primary with size", DSBCAPS_PRIMARYBUFFER, 4096, NULL);
    make(ds, "primary with format", DSBCAPS_PRIMARYBUFFER, 0, (fmt(&w, WAVE_FORMAT_PCM, 2, 44100, 16), &w));
    if (prim)
    {
        memset(&bc, 0, sizeof(bc));
        bc.dwSize = sizeof(bc);
        HR("  prim caps", IDirectSoundBuffer_GetCaps(prim, &bc));
        printf("  prim flags=%lx\n", bc.dwFlags);
        sz = 0;
        HR("  prim fmt size", IDirectSoundBuffer_GetFormat(prim, NULL, 0, &sz));
        printf("  prim fmt bytes %lu\n", sz);
        fmt(&w, WAVE_FORMAT_PCM, 2, 22050, 16);
        HR("  prim set 22k", IDirectSoundBuffer_SetFormat(prim, &w.Format));
        memset(&got, 0, sizeof(got));
        IDirectSoundBuffer_GetFormat(prim, &got.Format, sizeof(got), NULL);
        printf("  prim fmt tag=%u ch=%u rate=%lu bits=%u align=%u\n", got.Format.wFormatTag,
               got.Format.nChannels, got.Format.nSamplesPerSec, got.Format.wBitsPerSample,
               got.Format.nBlockAlign);
        fmt(&w, WAVE_FORMAT_PCM, 2, 44100, 12);
        HR("  prim set bad", IDirectSoundBuffer_SetFormat(prim, &w.Format));
        HR("  prim lock", IDirectSoundBuffer_Lock(prim, 0, 0, &p1, &n1, &p2, &n2, DSBLOCK_ENTIREBUFFER));
        HR("  prim set vol", IDirectSoundBuffer_SetVolume(prim, -600));
        IDirectSoundBuffer_GetVolume(prim, &l);
        printf("  prim vol %ld\n", l);
        listener(prim);
    }

    for (i = dx_invalid ? nfmts - nbad : 0; i < (dx_invalid ? nfmts : nfmts - nbad); i++)
    {
        fmt(&w, fmts[i].tag_, fmts[i].ch, fmts[i].rate, fmts[i].bits);
        /* An eighth of a second, whole frames; the malformed formats (align 0) get 4096. */
        align = w.Format.nBlockAlign;
        sz = align && w.Format.nAvgBytesPerSec ? w.Format.nAvgBytesPerSec / 8 / align * align : 4096;
        b = make(ds, fmts[i].tag, DSBCAPS_CTRLFREQUENCY | DSBCAPS_CTRLVOLUME | DSBCAPS_CTRLPAN |
                 DSBCAPS_GETCURRENTPOSITION2, sz, &w);
        if (!b)
            continue;
        memset(&got, 0, sizeof(got));
        sz = 0;
        IDirectSoundBuffer_GetFormat(b, &got.Format, sizeof(got), &sz);
        memset(&bc, 0, sizeof(bc));
        bc.dwSize = sizeof(bc);
        IDirectSoundBuffer_GetCaps(b, &bc);
        printf("  fmt bytes=%lu tag=%u align=%u caps=%lx size=%lu\n", sz, got.Format.wFormatTag,
               got.Format.nBlockAlign, bc.dwFlags, bc.dwBufferBytes);
        fill(b, (BYTE)i);
        IDirectSoundBuffer_SetFrequency(b, fmts[i].rate / 2 + 1000);
        IDirectSoundBuffer_SetPan(b, i % 2 ? -2500 : 2500);
        play(b, fmts[i].tag, 0);
        IDirectSoundBuffer_Release(b);
    }

    if (dx_invalid)
        goto out;
    fmt(&w, WAVE_FORMAT_PCM, 2, 44100, 16);
    make(ds, "3d stereo", DSBCAPS_CTRL3D, 8192, &w);
    make(ds, "3d with pan", DSBCAPS_CTRL3D | DSBCAPS_CTRLPAN, 8192, (fmt(&w, WAVE_FORMAT_PCM, 1, 22050, 16), &w));
    make(ds, "tiny", 0, 2, &w);
    make(ds, "huge", 0, 0x10000000, &w);
    make(ds, "fx small", DSBCAPS_CTRLFX, 256, &w);
    make(ds, "hw and sw", DSBCAPS_LOCHARDWARE | DSBCAPS_LOCSOFTWARE, 8192, &w);

    b = make(ds, "main", DSBCAPS_CTRL3D | DSBCAPS_CTRLFREQUENCY | DSBCAPS_CTRLVOLUME |
             DSBCAPS_CTRLPOSITIONNOTIFY | DSBCAPS_GETCURRENTPOSITION2, 22050, &w);
    if (!b)
        goto out;
    HR("  set pos", IDirectSoundBuffer_SetCurrentPosition(b, 1001));
    IDirectSoundBuffer_GetCurrentPosition(b, &n1, &n2);
    printf("  pos back play=%lu\n", n1);
    HR("  set pos past end", IDirectSoundBuffer_SetCurrentPosition(b, 22050));
    HR("  lock wrap", IDirectSoundBuffer_Lock(b, 20000, 4000, &p1, &n1, &p2, &n2, 0));
    printf("  wrap sizes %lu %lu second=%d\n", n1, n2, p2 != NULL);
    IDirectSoundBuffer_Unlock(b, p1, n1, p2, n2);
    HR("  lock no wrap ptr", IDirectSoundBuffer_Lock(b, 20000, 4000, &p1, &n1, NULL, NULL, 0));
    HR("  lock too big", IDirectSoundBuffer_Lock(b, 0, 30000, &p1, &n1, &p2, &n2, 0));
    HR("  lock from write", IDirectSoundBuffer_Lock(b, 0, 100, &p1, &n1, &p2, &n2, DSBLOCK_FROMWRITECURSOR));
    IDirectSoundBuffer_Unlock(b, p1, n1, NULL, 0);
    HR("  unlock bogus", IDirectSoundBuffer_Unlock(b, (BYTE *)p1 + 1, 4, NULL, 0));
    HR("  vol bad", IDirectSoundBuffer_SetVolume(b, 100));
    HR("  vol", IDirectSoundBuffer_SetVolume(b, -1234));
    IDirectSoundBuffer_GetVolume(b, &l);
    printf("  vol back %ld\n", l);
    HR("  pan no ctrl", IDirectSoundBuffer_SetPan(b, 100));
    HR("  freq low", IDirectSoundBuffer_SetFrequency(b, 50));
    HR("  freq", IDirectSoundBuffer_SetFrequency(b, 30000));
    IDirectSoundBuffer_GetFrequency(b, &v);
    printf("  freq back %lu\n", v);
    HR("  freq original", IDirectSoundBuffer_SetFrequency(b, DSBFREQUENCY_ORIGINAL));
    IDirectSoundBuffer_GetFrequency(b, &v);
    printf("  freq original back %lu\n", v);
    buffer3d(b);
    if (SUCCEEDED(IDirectSoundBuffer_QueryInterface(b, &IID_IDirectSoundNotify, (void **)&n)))
    {
        pos[0].dwOffset = 100; pos[0].hEventNotify = CreateEventA(NULL, FALSE, FALSE, NULL);
        pos[1].dwOffset = DSBPN_OFFSETSTOP; pos[1].hEventNotify = CreateEventA(NULL, FALSE, FALSE, NULL);
        pos[2].dwOffset = 30000; pos[2].hEventNotify = pos[0].hEventNotify;
        HR("  notify bad offset", IDirectSoundNotify_SetNotificationPositions(n, 3, pos));
        HR("  notify", IDirectSoundNotify_SetNotificationPositions(n, 2, pos));
        HR("  notify null", IDirectSoundNotify_SetNotificationPositions(n, 1, NULL));
        fill(b, 0x40);
        play(b, "notify", 0);
        printf("  stop event %d\n", WaitForSingleObject(pos[1].hEventNotify, 2000) == WAIT_OBJECT_0);
        printf("  notify release %lu\n", IDirectSoundNotify_Release(n));
        CloseHandle(pos[0].hEventNotify);
        CloseHandle(pos[1].hEventNotify);
    }
    hr = IDirectSound8_DuplicateSoundBuffer(ds, b, &dup);
    HR("  duplicate", hr);
    if (SUCCEEDED(hr))
    {
        IDirectSoundBuffer_GetVolume(dup, &l);
        IDirectSoundBuffer_GetFrequency(dup, &v);
        printf("  dup vol %ld freq %lu\n", l, v);
        play(dup, "dup loop", DSBPLAY_LOOPING);
        printf("  dup release %lu\n", IDirectSoundBuffer_Release(dup));
    }
    if (prim)
        HR("  duplicate primary", IDirectSound8_DuplicateSoundBuffer(ds, prim, &dup));
    if (SUCCEEDED(IDirectSoundBuffer_QueryInterface(b, &IID_IDirectSoundBuffer8, (void **)&b8)))
    {
        DWORD res[2];
        HR("  fx none", IDirectSoundBuffer8_SetFX(b8, 0, NULL, NULL));
        HR("  acquire", IDirectSoundBuffer8_AcquireResources(b8, 0, 1, res));
        IDirectSoundBuffer8_Release(b8);
    }
    printf("  main release %lu\n", IDirectSoundBuffer_Release(b));
out:
    if (prim)
        printf("  primary release %lu\n", IDirectSoundBuffer_Release(prim));
    printf("ds release %lu\n", IDirectSound8_Release(ds));
}

int main(int argc, char **argv)
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
    HWND hwnd;
    HRESULT hr;

    /* `ks`: only the direct KsCreatePin diagnostic (no suite case runs it: a second create on
     * the AC97's render pin hangs in the kernel -- roadmap "Known bugs"). */
    if (argc > 1 && !strcmp(argv[1], "ks"))
    {
        setvbuf(stdout, NULL, _IONBF, 0);
        winmm();
        ks_direct();
        return DONE();
    }
    dx_invalid = argc > 1 && !strcmp(argv[1], "invalid");
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
    hwnd = dx_window();
    device(hwnd);
    DestroyWindow(hwnd);
    HR("can unload", ((CANUNLOAD)dx_proc("DllCanUnloadNow"))());
    return DONE();
}
